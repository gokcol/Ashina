//! Sharing copies are rewritten, never appended to the original PDF.
use anyhow::{Context, Result, ensure};
use lopdf::{Document, Object, Stream, dictionary};
#[derive(Clone, Copy)]
pub struct Options {
    pub metadata: bool,
    pub comments: bool,
    pub attachments: bool,
    pub hidden: bool,
    pub dpi: f32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            metadata: true,
            comments: true,
            attachments: true,
            hidden: true,
            dpi: 200.,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Report {
    pub pages: usize,
    pub flattened: bool,
    pub warnings: Vec<String>,
}
pub fn prepare(
    bytes: &[u8],
    options: Options,
    mut progress: impl FnMut(usize, usize) -> bool,
) -> Result<(Vec<u8>, Report)> {
    let mut doc = Document::load_mem(bytes)?;
    ensure!(
        !doc.is_encrypted(),
        "Unlock the PDF before preparing a sharing copy"
    );
    ensure!(
        options.hidden || !crate::document::has_signatures(&doc),
        "A signed PDF needs Remove hidden content enabled; the sharing copy will be unsigned"
    );
    ensure!((72. ..=300.).contains(&options.dpi), "Choose 72–300 dpi");
    let document_layers: std::collections::HashSet<_> = crate::annotations::load(&doc)
        .into_iter()
        .filter(|a| a.object.text.is_some())
        .map(|a| format!("MyPDF-{}", a.id).into_bytes())
        .collect();
    let pages = doc.get_pages();
    ensure!(!pages.is_empty(), "The PDF has no pages");
    for id in pages.values() {
        let annots = doc
            .get_dictionary(*id)?
            .get(b"Annots")
            .ok()
            .and_then(|o| doc.dereference(o).ok())
            .and_then(|(_, o)| o.as_array().ok())
            .cloned();
        if let Some(annots) = annots {
            let keep: Vec<_> = annots
                .into_iter()
                .filter(|a| {
                    if doc
                        .dereference(a)
                        .ok()
                        .and_then(|(_, o)| o.as_dict().ok())
                        .and_then(|d| d.get(b"NM").ok())
                        .and_then(|o| o.as_str().ok())
                        .is_some_and(|name| document_layers.contains(name))
                    {
                        return true;
                    }
                    let subtype = doc
                        .dereference(a)
                        .ok()
                        .and_then(|(_, o)| o.as_dict().ok())
                        .and_then(|d| d.get(b"Subtype").ok())
                        .and_then(|o| o.as_name().ok())
                        .unwrap_or_default();
                    let attachment = matches!(
                        subtype,
                        b"FileAttachment" | b"Sound" | b"Movie" | b"Screen" | b"RichMedia"
                    );
                    !(options.attachments && attachment
                        || options.comments && !matches!(subtype, b"Widget" | b"Link"))
                })
                .collect();
            doc.get_dictionary_mut(*id)?.set("Annots", keep);
        }
    }
    if options.hidden {
        // Render only the visible page, then build a new object graph containing
        // image pixels. No source strings, streams, attachments or revisions survive.
        let input = crate::content::serialize(&mut doc)?;
        let renderer = crate::viewer::PageRenderer::new(&input)?;
        let geometry = crate::annotations::page_geometry(&input)?;
        let mut fresh = Document::with_version("1.7");
        let parent = fresh.new_object_id();
        let mut kids = vec![];
        for (page, (size, _)) in geometry.iter().enumerate() {
            ensure!(progress(page, geometry.len()), "Cancelled");
            ensure!(
                (size[2] * options.dpi / 72.) * (size[3] * options.dpi / 72.) <= 32_000_000.,
                "Page {} is too large at this resolution; choose a lower DPI",
                page + 1
            );
            let ([w, h], rgba) = renderer.render(page, options.dpi / 72.)?;
            let image = image::RgbaImage::from_raw(w as u32, h as u32, rgba)
                .context("Page render failed")?;
            let image_id = crate::content::embed_image(&mut fresh, &image)?;
            let stream = fresh.add_object(Stream::new(
                dictionary! {},
                format!("q {} 0 0 {} 0 0 cm /PageImage Do Q", size[2], size[3]).into_bytes(),
            ));
            let id=fresh.add_object(dictionary!{"Type"=>"Page","Parent"=>parent,"MediaBox"=>vec![0.into(),0.into(),size[2].into(),size[3].into()],"Resources"=>dictionary!{"XObject"=>dictionary!{"PageImage"=>image_id}},"Contents"=>stream});
            kids.push(Object::Reference(id));
        }
        fresh.objects.insert(
            parent,
            Object::Dictionary(
                dictionary! {"Type"=>"Pages","Kids"=>kids,"Count"=>pages.len() as i64},
            ),
        );
        let root = fresh.add_object(dictionary! {"Type"=>"Catalog","Pages"=>parent});
        fresh.trailer.set("Root", root);
        ensure!(progress(pages.len(), pages.len()), "Cancelled");
        return Ok((
            crate::content::serialize(&mut fresh)?,
            Report {
                pages: pages.len(),
                flattened: true,
                warnings: vec![],
            },
        ));
    }
    if options.metadata {
        doc.trailer.remove(b"Info");
        doc.trailer.remove(b"ID");
    }
    fn clean(o: &mut Object, options: Options) {
        match o {
            Object::Dictionary(d) => {
                for key in [b"OpenAction".as_slice(), b"AA", b"JavaScript"] {
                    d.remove(key);
                }
                if options.metadata {
                    for key in [b"Metadata".as_slice(), b"PieceInfo", b"LastModified"] {
                        d.remove(key);
                    }
                }
                if options.attachments {
                    for key in [b"EmbeddedFiles".as_slice(), b"AF", b"Collection"] {
                        d.remove(key);
                    }
                }
                for (_, value) in d.iter_mut() {
                    clean(value, options);
                }
            }
            Object::Stream(s) => {
                let mut d = Object::Dictionary(s.dict.clone());
                clean(&mut d, options);
                s.dict = d.as_dict().unwrap().clone();
            }
            Object::Array(a) => {
                for value in a {
                    clean(value, options);
                }
            }
            _ => {}
        }
    }
    for value in doc.objects.values_mut() {
        clean(value, options);
    }
    ensure!(progress(pages.len(), pages.len()), "Cancelled");
    let intermediate = crate::content::serialize(&mut doc)?;
    let (cleaned, _) = crate::workflows::sharing_copy(
        &intermediate,
        crate::workflows::ShareOptions {
            metadata: options.metadata,
            comments: options.comments,
            attachments: options.attachments,
        },
    )?;
    Ok((cleaned,Report{pages:pages.len(),flattened:false,warnings:vec!["Hidden text, cropped content and optional layers may remain. This copy is not fully sanitized.".into()]}))
}

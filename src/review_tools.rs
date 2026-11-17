//! Local review exchange and printable page-referenced reports.
use crate::{
    annotations::{self, Annotation},
    content,
    pdf_font::PdfFont,
};
use anyhow::{Context, Result, ensure};
use lopdf::{Document, Object, Stream, dictionary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_EXCHANGE_BYTES: u64 = 64 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
struct Exchange {
    format: String,
    version: u32,
    pages: Vec<[f32; 4]>,
    base_digest: String,
    comments: Vec<Annotation>,
}
pub fn is_comment(a: &Annotation) -> bool {
    a.object.text.is_none()
        && a.object.table.is_none()
        && (!matches!(
            a.kind,
            annotations::Kind::Image { .. } | annotations::Kind::Svg { .. }
        ) || a.object.stamp
            || a.asset_id.is_some())
}
fn base_signature(bytes: &[u8]) -> Result<(Vec<[f32; 4]>, String)> {
    let mut doc = Document::load_mem(bytes)?;
    ensure!(
        !doc.is_encrypted(),
        "Unlock the document before exchanging comments"
    );
    let geometry: Vec<_> = annotations::page_geometry(bytes)?
        .into_iter()
        .map(|p| p.0)
        .collect();
    ensure!(
        !geometry.is_empty() && geometry.len() <= 1000,
        "Review exchange supports 1–1000 pages"
    );
    // Hash page content, without review appearances, so another reviewer's PDF
    // matches even after annotation changes, metadata edits or object renumbering.
    let content_ids: std::collections::HashSet<_> = annotations::load(&doc)
        .into_iter()
        .filter(|a| !is_comment(a))
        .map(|a| format!("MyPDF-{}", a.id))
        .collect();
    for (_, page) in doc.get_pages() {
        let retained = doc
            .get_object(page)?
            .as_dict()?
            .get(b"Annots")
            .ok()
            .and_then(|value| doc.dereference(value).ok())
            .and_then(|(_, value)| value.as_array().ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|value| {
                        let Ok((_, object)) = doc.dereference(value) else {
                            return false;
                        };
                        let Ok(dictionary) = object.as_dict() else {
                            return false;
                        };
                        dictionary
                            .get(b"NM")
                            .and_then(Object::as_str)
                            .ok()
                            .is_some_and(|name| {
                                content_ids.contains(String::from_utf8_lossy(name).as_ref())
                            })
                            || dictionary.get(b"Subtype").and_then(Object::as_name).ok()
                                == Some(b"Widget")
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        doc.get_object_mut(page)?
            .as_dict_mut()?
            .set("Annots", retained);
    }
    let base = content::serialize(&mut doc)?;
    let renderer = crate::viewer::PageRenderer::new(&base)?;
    let mut digest = Sha256::new();
    for (i, rect) in geometry.iter().enumerate() {
        for value in rect {
            digest.update(value.to_le_bytes());
        }
        let (size, pixels) = renderer.render(i, 0.25)?;
        for value in size {
            digest.update((value as u64).to_le_bytes());
        }
        digest.update(pixels);
    }
    Ok((geometry, format!("{:x}", digest.finalize())))
}
/// Stop serialization before allocating an oversized exchange file.
struct LimitedOutput(Vec<u8>);
impl std::io::Write for LimitedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_EXCHANGE_BYTES as usize - self.0.len() {
            return Err(std::io::Error::other("Review file exceeds 64 MB"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub fn export_comments(bytes: &[u8], marks: &[Annotation]) -> Result<Vec<u8>> {
    let (pages, base_digest) = base_signature(bytes)?;
    let comments: Vec<_> = marks.iter().filter(|a| is_comment(a)).cloned().collect();
    ensure!(
        comments.len() <= 2000,
        "Exchange up to 2000 comments at a time"
    );
    for a in &comments {
        a.validate()?;
        ensure!(a.page < pages.len(), "Comment page is unavailable");
    }
    let mut output = LimitedOutput(Vec::new());
    serde_json::to_writer_pretty(
        &mut output,
        &Exchange {
            format: "MyPDF review comments".into(),
            version: 1,
            pages,
            base_digest,
            comments,
        },
    )
    .context("Comments exceed the export limit or could not be encoded")?;
    Ok(output.0)
}
pub fn import_comments(bytes: &[u8], input: &[u8]) -> Result<Vec<Annotation>> {
    ensure!(
        input.len() as u64 <= MAX_EXCHANGE_BYTES,
        "Review file exceeds 64 MB"
    );
    let exchange: Exchange =
        serde_json::from_slice(input).context("Choose a Ashina review file (.mypdf-review)")?;
    ensure!(
        exchange.format == "MyPDF review comments" && exchange.version == 1,
        "Unsupported review file version"
    );
    ensure!(
        exchange.comments.len() <= 2000,
        "Review file exceeds 2000 comments"
    );
    let (pages, digest) = base_signature(bytes)?;
    ensure!(
        exchange.pages == pages && exchange.base_digest == digest,
        "These comments belong to a different page layout or document. Open the matching PDF first."
    );
    let mut ids = std::collections::HashSet::new();
    for a in &exchange.comments {
        a.validate()?;
        ensure!(
            a.page < pages.len() && is_comment(a) && ids.insert(&a.id),
            "Invalid or duplicated review comment"
        );
    }
    Ok(exchange.comments)
}
struct DigestWriter<'a>(&'a mut Sha256);
impl std::io::Write for DigestWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
/// Never silently overwrite a local review. Changed IDs are imported as a
/// separate comment; exact duplicates are skipped. The GUI previews the count.
pub fn merge_comments(
    existing: &[Annotation],
    incoming: &[Annotation],
) -> (Vec<Annotation>, usize, usize) {
    let mut merged = existing.to_vec();
    let mut by_id: std::collections::HashMap<String, usize> = merged
        .iter()
        .enumerate()
        .map(|(i, a)| (a.id.clone(), i))
        .collect();
    let (mut added, mut conflicts) = (0, 0);
    for original in incoming {
        let mut a = original.clone();
        let conflict = if let Some(&index) = by_id.get(&a.id) {
            if merged[index] == a {
                continue;
            }
            // Stable conflict identity makes repeated import idempotent, even
            // after preserving a changed version beside the local original.
            let mut digest = Sha256::new();
            digest.update(b"MyPDF imported review v1");
            if serde_json::to_writer(&mut DigestWriter(&mut digest), &a).is_err() {
                continue;
            }
            a.id = format!("{:x}", digest.finalize())[..32].to_owned();
            a.object.name = format!(
                "{} (imported review)",
                a.object.name.chars().take(180).collect::<String>()
            );
            a.object.group = None;
            if by_id.contains_key(&a.id) {
                continue;
            }
            true
        } else {
            false
        };
        by_id.insert(a.id.clone(), merged.len());
        merged.push(a);
        added += 1;
        conflicts += usize::from(conflict);
    }
    (merged, added, conflicts)
}

/// One numbered comment beside a page preview, with continuation pages for
/// long notes/replies. Text is searchable Unicode, not a screenshot of a list.
pub fn pdf_summary(
    bytes: &[u8],
    marks: &[Annotation],
    mut progress: impl FnMut(usize, usize) -> bool,
) -> Result<Vec<u8>> {
    let marks: Vec<_> = marks.iter().filter(|a| is_comment(a)).collect();
    ensure!(
        !marks.is_empty() && marks.len() <= 2000,
        "Choose 1–2000 comments for the summary"
    );
    let geometry = annotations::page_geometry(bytes)?;
    let mut description_bytes = 0usize;
    let descriptions: Vec<String> = marks
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let mut text = format!(
                "Comment {} · Page {}\n{} · {}\n{}\n",
                i + 1,
                a.page + 1,
                a.suggestion.as_ref().map_or(a.kind.label(), |s| s.label()),
                if a.object.review.resolved {
                    "Resolved"
                } else {
                    "Open"
                },
                a.object.review.author
            );
            if let Some(s) = &a.suggestion
                && !s.original.is_empty()
            {
                text.push_str(&format!("Original: {}\nProposed: ", s.original));
            }
            text.push_str(a.kind.text().unwrap_or(&a.object.name));
            if let Some(voice) = &a.voice {
                text.push_str(&format!(
                    "\nVoice recording: {:.1} seconds (audio stays in the PDF).",
                    voice.duration()
                ));
            }
            for reply in &a.object.review.replies {
                text.push_str(&format!("\n\n{}: {}", reply.author, reply.text));
            }
            description_bytes += text.len();
            ensure!(
                description_bytes <= 8 * 1024 * 1024,
                "Summary text exceeds 8 MB"
            );
            Ok(text
                .replace('\t', "    ")
                .chars()
                .filter(|c| !c.is_control() || *c == '\n')
                .collect())
        })
        .collect::<Result<_>>()?;
    let all_text = format!(
        "Ashina Review summary Continued 0123456789{}",
        descriptions.join("\n")
    );
    ensure!(
        all_text.len() <= 8 * 1024 * 1024,
        "Summary text exceeds 8 MB"
    );
    let mut d = Document::with_version("1.7");
    let font = PdfFont::embed(&mut d, &all_text)?;
    let pages = d.new_object_id();
    let mut kids = vec![];
    let renderer = crate::viewer::PageRenderer::new(bytes)?;
    let mut previews = std::collections::HashMap::new();
    for (i, (a, description)) in marks.iter().zip(&descriptions).enumerate() {
        ensure!(progress(i, marks.len()), "Summary cancelled");
        let rect = geometry
            .get(a.page)
            .context("Comment page is unavailable")?
            .0;
        let scale = (260. / rect[2]).min(650. / rect[3]);
        let (w, h) = (rect[2] * scale, rect[3] * scale);
        let image_id = if let Some(&id) = previews.get(&a.page) {
            id
        } else {
            let (size, pixels) = renderer.render(a.page, scale * 2.)?;
            let image = image::RgbaImage::from_raw(size[0] as u32, size[1] as u32, pixels)
                .context("Invalid page preview")?;
            let id = content::embed_image(&mut d, &image)?;
            previews.insert(a.page, id);
            id
        };
        let lines: Vec<_> = description
            .lines()
            .flat_map(|line| crate::convert::wrap(line, |s| font.width(s, 10.) <= 235.))
            .collect();
        for (part, lines) in lines.chunks(48).enumerate() {
            ensure!(kids.len() < 4000, "Summary exceeds 4000 pages");
            let mut body = format!(
                "q {w} 0 0 {h} 28 {} cm /Preview Do Q\nq 0.8 0.82 0.86 RG 0.5 w 28 {} {w} {h} re S Q\n",
                755. - h,
                755. - h
            );
            let x = (28. + a.rect[0] * scale).clamp(28., 276.);
            let y = (755. - a.rect[1] * scale).clamp(105., 755.);
            body.push_str(&format!("q 0.08 0.35 0.8 rg {x} {y} 22 16 re f Q\nBT /F0 10 Tf 1 1 1 rg 1 0 0 1 {} {} Tm <{}> Tj ET\n", x+3.,y+4.,font.hex(&(i+1).to_string())?));
            for (j, line) in lines.iter().enumerate() {
                body.push_str(&format!(
                    "BT /F0 10 Tf 0.12 0.14 0.18 rg 1 0 0 1 322 {} Tm <{}> Tj ET\n",
                    755. - j as f32 * 14.,
                    font.hex(line)?
                ));
            }
            let title = if part == 0 {
                "Ashina Review summary"
            } else {
                "Ashina Review summary · Continued"
            };
            body.push_str(&format!(
                "BT /F0 16 Tf 0.12 0.14 0.18 rg 1 0 0 1 28 798 Tm <{}> Tj ET\n",
                font.hex(title)?
            ));
            let stream = d.add_object(Stream::new(dictionary! {}, body.into_bytes()));
            let page=d.add_object(dictionary!{"Type"=>"Page","Parent"=>pages,"MediaBox"=>vec![0.into(),0.into(),595.276.into(),841.89.into()],"Resources"=>dictionary!{"Font"=>dictionary!{"F0"=>font.id},"XObject"=>dictionary!{"Preview"=>image_id}},"Contents"=>stream});
            kids.push(Object::Reference(page));
        }
    }
    d.objects.insert(
        pages,
        Object::Dictionary(dictionary! {"Type"=>"Pages","Count"=>kids.len() as i64,"Kids"=>kids}),
    );
    let root = d.add_object(dictionary! {"Type"=>"Catalog","Pages"=>pages});
    d.trailer.set("Root", root);
    content::serialize(&mut d)
}

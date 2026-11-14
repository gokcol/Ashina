use lopdf::{Object, Stream, dictionary};
use mypdf_milestone0::{
    annotations::{self, Annotation, Kind, Style},
    sanitize::{self, Options},
};
fn source() -> Vec<u8> {
    let bytes = mypdf_milestone0::content::add_text(
        &mypdf_milestone0::convert::blank(300., 400.).unwrap(),
        1,
        "VISIBLE",
        [20., 30., 250., 70.],
        14.,
        [0.; 3],
    )
    .unwrap();
    let mut note = Annotation::new(
        0,
        [50., 100., 80., 130.],
        Kind::Note {
            text: "PRIVATE COMMENT".into(),
        },
        Style::default(),
    );
    note.voice = Some(mypdf_milestone0::voice::Clip::from_samples(&[123, 456, 789]).unwrap());
    let bytes = annotations::export(&bytes, &[note]).unwrap();
    let mut doc = lopdf::Document::load_mem(&bytes).unwrap();
    let info = doc.add_object(dictionary! {"Author"=>Object::string_literal("PRIVATE AUTHOR")});
    doc.trailer.set("Info", info);
    let xmp = doc.add_object(Stream::new(
        dictionary! {"Type"=>"Metadata","Subtype"=>"XML"},
        b"PRIVATE XMP".to_vec(),
    ));
    let root = doc.trailer.get(b"Root").unwrap().as_reference().unwrap();
    doc.get_dictionary_mut(root).unwrap().set("Metadata", xmp);
    let ghost = doc.add_object(Object::string_literal("PRIVATE UNREFERENCED"));
    let _ = ghost;
    mypdf_milestone0::content::serialize(&mut doc).unwrap()
}
#[test]
fn sharing_copy_has_only_visible_page_pixels_and_never_keeps_source_metadata() {
    let bytes = source();
    let before = bytes.clone();
    let (out, report) = sanitize::prepare(
        &bytes,
        Options {
            dpi: 72.,
            ..Default::default()
        },
        |_, _| true,
    )
    .unwrap();
    assert_eq!(bytes, before);
    assert!(report.flattened);
    assert!(report.warnings.is_empty());
    let doc = lopdf::Document::load_mem(&out).unwrap();
    assert_eq!(doc.get_pages().len(), 1);
    assert!(doc.trailer.get(b"Info").is_err());
    assert!(annotations::load(&doc).is_empty());
    assert!(doc.extract_text(&[1]).unwrap().trim().is_empty());
    for o in doc.objects.values() {
        if let Ok(d) = o.as_dict() {
            assert!(d.get(b"Metadata").is_err());
            assert!(d.get(b"Annots").is_err());
        }
    }
    let geometry = annotations::page_geometry(&out).unwrap();
    assert_eq!(geometry[0].0[2..4], [300., 400.]);
    let mut clean = lopdf::Document::load_mem(&before).unwrap();
    let id = clean.get_pages()[&1];
    clean.get_dictionary_mut(id).unwrap().remove(b"Annots");
    let expected = mypdf_milestone0::content::serialize(&mut clean).unwrap();
    let (_, a) = mypdf_milestone0::viewer::render_page(&expected, 0, 1.).unwrap();
    let (_, b) = mypdf_milestone0::viewer::render_page(&out, 0, 1.).unwrap();
    assert_eq!(a, b);
}
#[test]
fn selective_cleanup_preserves_text_reports_limits_and_can_cancel() {
    let bytes = source();
    let (out, report) = sanitize::prepare(
        &bytes,
        Options {
            hidden: false,
            ..Default::default()
        },
        |_, _| true,
    )
    .unwrap();
    assert!(!report.flattened);
    assert!(!report.warnings.is_empty());
    let doc = lopdf::Document::load_mem(&out).unwrap();
    assert!(doc.extract_text(&[1]).unwrap().contains("VISIBLE"));
    assert!(annotations::load(&doc).is_empty());
    assert!(doc.trailer.get(b"Info").is_err());
    assert!(sanitize::prepare(&bytes, Options::default(), |_, _| false).is_err());
}

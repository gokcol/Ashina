use mypdf_milestone0::{
    annotations::{self, Annotation, Kind, Style},
    voice::Clip,
};
#[test]
fn voice_comment_roundtrips_and_exposes_a_standard_wav_attachment() {
    let source = mypdf_milestone0::convert::blank(595., 842.).unwrap();
    let mut mark = Annotation::new(
        0,
        [30., 30., 58., 58.],
        Kind::Note {
            text: "Inspection note".into(),
        },
        Style::default(),
    );
    mark.voice = Some(Clip::from_samples(&[0, 1000, -1000, 32767, -32768]).unwrap());
    let out = annotations::export(&source, &[mark.clone()]).unwrap();
    let doc = lopdf::Document::load_mem(&out).unwrap();
    assert_eq!(annotations::load(&doc), vec![mark.clone()]);
    let attachment = doc
        .objects
        .values()
        .filter_map(|o| o.as_dict().ok())
        .find(|d| d.get(b"Subtype").and_then(|o| o.as_name()).ok() == Some(b"FileAttachment"))
        .unwrap();
    let file = doc
        .dereference(attachment.get(b"FS").unwrap())
        .unwrap()
        .1
        .as_dict()
        .unwrap();
    let stream = doc
        .dereference(
            file.get(b"EF")
                .unwrap()
                .as_dict()
                .unwrap()
                .get(b"F")
                .unwrap(),
        )
        .unwrap()
        .1
        .as_stream()
        .unwrap();
    assert_eq!(
        stream.decompressed_content().unwrap(),
        *mark.voice.as_ref().unwrap().wav
    );
    let empty = annotations::export(&out, &[]).unwrap();
    let cleaned = lopdf::Document::load_mem(&empty).unwrap();
    assert!(annotations::load(&cleaned).is_empty());
    assert!(
        !cleaned
            .objects
            .values()
            .filter_map(|o| o.as_stream().ok())
            .any(|s| s.dict.get(b"Type").and_then(|v| v.as_name()).ok() == Some(b"EmbeddedFile"))
    );
}
#[test]
fn old_text_comments_still_load_without_voice_payload() {
    let mark = Annotation::new(
        0,
        [30., 30., 58., 58.],
        Kind::Note {
            text: "Old comment".into(),
        },
        Style::default(),
    );
    let json = serde_json::to_string(&mark).unwrap();
    assert!(!json.contains("voice"));
    assert_eq!(serde_json::from_str::<Annotation>(&json).unwrap(), mark);
    let mut invalid = mark;
    invalid.kind = Kind::Rectangle;
    invalid.voice = Some(Clip::default());
    assert!(invalid.validate().is_err());
}

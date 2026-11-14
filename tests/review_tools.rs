use mypdf_milestone0::{
    annotations::{self, Annotation, Kind, Shape, Style},
    convert,
    review::{Reply, Suggestion, SuggestionKind},
    review_tools,
    voice::Clip,
};
fn note() -> Annotation {
    Annotation::new(
        0,
        [40., 80., 64., 104.],
        Kind::Note {
            text: "İnceleme · Größe — approved".into(),
        },
        Style::default(),
    )
}
#[test]
fn review_exchange_preserves_voice_suggestions_and_replies_without_overwriting_local_work() {
    let source = convert::text_to_pdf("Original wording", false).unwrap();
    let mut a = note();
    a.voice = Some(Clip::from_samples(&[1, -2, 3, -4]).unwrap());
    a.object.review.replies.push(Reply {
        author: "Reviewer".into(),
        created: 1,
        text: "Keep this reply".into(),
    });
    let mut b = note();
    b.suggestion = Some(Suggestion {
        kind: SuggestionKind::Replace,
        original: "Original wording".into(),
    });
    let saved = annotations::export(&source, &[a.clone(), b.clone()]).unwrap();
    let bundle = review_tools::export_comments(&saved, &[a.clone(), b.clone()]).unwrap();
    assert_eq!(
        review_tools::import_comments(&source, &bundle).unwrap(),
        vec![a.clone(), b.clone()]
    );
    let (merged, added, conflicts) =
        review_tools::merge_comments(&[a.clone(), b.clone()], &[a.clone(), b.clone()]);
    assert_eq!((merged.len(), added, conflicts), (2, 0, 0));
    let mut changed = a.clone();
    *changed.kind.text_mut().unwrap() = "Another reviewer".into();
    let (merged, added, conflicts) = review_tools::merge_comments(&[a.clone()], &[changed]);
    assert_eq!((added, conflicts), (1, 1));
    assert_eq!(merged[0], a);
    assert_ne!(merged[1].id, a.id);
    let other = convert::text_to_pdf("Unrelated document", false).unwrap();
    assert!(review_tools::import_comments(&other, &bundle).is_err());
    let mut invalid: serde_json::Value = serde_json::from_slice(&bundle).unwrap();
    invalid["comments"][0]["page"] = 100.into();
    assert!(
        review_tools::import_comments(&source, &serde_json::to_vec(&invalid).unwrap()).is_err()
    );
}
#[test]
fn pdf_review_summary_keeps_unicode_long_replies_and_page_previews() {
    let source = convert::text_to_pdf("Original wording", false).unwrap();
    let mut a = note();
    a.object.review.replies.push(Reply {
        author: "Reviewer".into(),
        created: 1,
        text: format!(
            "{}Final reply survives.",
            "Long review sentence. ".repeat(300)
        ),
    });
    let saved = annotations::export(&source, &[a.clone()]).unwrap();
    let out = review_tools::pdf_summary(&saved, &[a.clone()], |_, _| true).unwrap();
    let doc = lopdf::Document::load_mem(&out).unwrap();
    assert!(doc.get_pages().len() > 1);
    let pages: Vec<_> = (0..doc.get_pages().len()).collect();
    let text =
        String::from_utf8(convert::export_text(&out, &pages, convert::Format::Text).unwrap())
            .unwrap();
    assert!(text.contains("İnceleme"));
    assert!(text.contains("Final reply survives."));
    assert!(text.contains("Comment 1"));
    assert!(
        doc.objects
            .values()
            .filter_map(|o| o.as_stream().ok())
            .any(|s| s.dict.get(b"Subtype").and_then(|o| o.as_name()).ok() == Some(b"Image"))
    );
    assert!(review_tools::pdf_summary(&saved, &[a], |_, _| false).is_err());
    std::fs::write("/tmp/mypdf-review-summary-test.pdf", out).unwrap();
}
#[test]
fn cloud_and_suggestion_roundtrip_without_rewriting_page_text() {
    let source = convert::text_to_pdf("Original wording", false).unwrap();
    let mut a = note();
    a.suggestion = Some(Suggestion {
        kind: SuggestionKind::Insert,
        original: String::new(),
    });
    let cloud = Annotation::new(
        0,
        [100., 150., 300., 250.],
        Kind::Shape {
            shape: Shape::Cloud,
        },
        Style::default(),
    );
    cloud.validate().unwrap();
    let saved = annotations::export(&source, &[a.clone(), cloud.clone()]).unwrap();
    let doc = lopdf::Document::load_mem(&saved).unwrap();
    assert_eq!(annotations::load(&doc), vec![a, cloud]);
    assert_eq!(
        convert::export_text(&source, &[0], convert::Format::Text).unwrap(),
        convert::export_text(&saved, &[0], convert::Format::Text).unwrap()
    );
}

#[test]
fn repeated_conflicting_review_imports_are_idempotent() {
    let original = note();
    let mut incoming = original.clone();
    *incoming.kind.text_mut().unwrap() = "Different review".into();
    let (merged, added, conflicts) = review_tools::merge_comments(
        std::slice::from_ref(&original),
        std::slice::from_ref(&incoming),
    );
    assert_eq!((added, conflicts), (1, 1));
    let (again, added, conflicts) = review_tools::merge_comments(&merged, &[incoming]);
    assert_eq!((added, conflicts), (0, 0));
    assert_eq!(again, merged);
}
#[test]
fn reused_renderer_matches_individual_page_rendering() {
    let source =
        convert::text_to_pdf("Rendering stays identical.\nİstanbul · Größe", false).unwrap();
    let renderer = mypdf_milestone0::viewer::PageRenderer::new(&source).unwrap();
    for scale in [0.25, 1., 1.5] {
        assert_eq!(
            renderer.render(0, scale).unwrap(),
            mypdf_milestone0::viewer::render_page(&source, 0, scale).unwrap()
        );
    }
    assert!(renderer.render(0, f32::NAN).is_err());
    assert!(renderer.render(1, 1.).is_err());
}

#[test]
fn exchange_checks_document_text_layers_and_excludes_them_from_comments() {
    let source = convert::blank(595., 842.).unwrap();
    let first = mypdf_milestone0::text_layer::add(
        &source,
        1,
        "First revision",
        [40., 40., 350., 70.],
        14.,
        [0.; 3],
        &Default::default(),
        &[],
    )
    .unwrap();
    let second = mypdf_milestone0::text_layer::add(
        &source,
        1,
        "Different revision",
        [40., 40., 350., 70.],
        14.,
        [0.; 3],
        &Default::default(),
        &[],
    )
    .unwrap();
    let mut marks = annotations::load(&lopdf::Document::load_mem(&first).unwrap());
    marks.push(note());
    let bundle = review_tools::export_comments(&first, &marks).unwrap();
    assert_eq!(
        review_tools::import_comments(&first, &bundle)
            .unwrap()
            .len(),
        1
    );
    assert!(review_tools::import_comments(&second, &bundle).is_err());
}

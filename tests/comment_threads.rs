use lopdf::{Document, Object, dictionary};
use mypdf_milestone0::{
    annotations::{self, Annotation, Kind, Style},
    convert,
    review::{Reply, Review},
};
fn note() -> Annotation {
    let mut a = Annotation::new(
        0,
        [20., 20., 44., 44.],
        Kind::Note {
            text: "Please check this figure.".into(),
        },
        Style::default(),
    );
    a.object.review = Review {
        author: "Orhan Gökçöl".into(),
        created: 1794655800,
        resolved: true,
        replies: vec![
            Reply {
                author: "Ayşe Yılmaz".into(),
                created: 1794656520,
                text: "The correct value is 125. Größe ✓".into(),
            },
            Reply {
                author: "Orhan Gökçöl".into(),
                created: 1794656700,
                text: "Thanks, updated.".into(),
            },
        ],
    };
    a
}
fn list(doc: &Document) -> Vec<Object> {
    doc.get_dictionary(*doc.get_pages().values().next().unwrap())
        .unwrap()
        .get(b"Annots")
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}
#[test]
fn pdf_contains_standard_thread_links_and_roundtrips_without_duplicates() {
    let original = convert::blank(300., 400.).unwrap();
    let a = note();
    let mut bytes = annotations::export(&original, std::slice::from_ref(&a)).unwrap();
    for _ in 0..3 {
        let doc = Document::load_mem(&bytes).unwrap();
        let values = list(&doc);
        assert_eq!(values.len(), 4, "root, two replies, and one review status");
        let root = values[0].as_reference().unwrap();
        let root_dict = doc.get_dictionary(root).unwrap();
        assert_eq!(
            lopdf::decode_text_string(root_dict.get(b"Contents").unwrap()).unwrap(),
            "Please check this figure."
        );
        for (value, reply) in values[1..3].iter().zip(&a.object.review.replies) {
            let d = doc.get_dictionary(value.as_reference().unwrap()).unwrap();
            assert_eq!(d.get(b"IRT").unwrap().as_reference().unwrap(), root);
            assert_eq!(d.get(b"RT").unwrap().as_name().unwrap(), b"R");
            assert_eq!(
                lopdf::decode_text_string(d.get(b"T").unwrap()).unwrap(),
                reply.author
            );
            assert_eq!(
                lopdf::decode_text_string(d.get(b"Contents").unwrap()).unwrap(),
                reply.text
            );
            assert!(d.has(b"CreationDate"));
        }
        let loaded = annotations::load(&doc);
        assert_eq!(loaded, vec![a.clone()]);
        bytes = annotations::export(&bytes, &loaded).unwrap();
    }
    let deleted = annotations::export(&bytes, &[]).unwrap();
    let doc = Document::load_mem(&deleted).unwrap();
    assert!(
        !doc.get_dictionary(*doc.get_pages().values().next().unwrap())
            .unwrap()
            .has(b"Annots")
    );
}
#[test]
fn replies_edited_and_added_by_another_reader_are_loaded_and_resaved() {
    let a = note();
    let original = annotations::export(&convert::blank(300., 400.).unwrap(), &[a]).unwrap();
    let mut doc = Document::load_mem(&original).unwrap();
    let mut values = list(&doc);
    let root = values[0].as_reference().unwrap();
    let reply = values[1].as_reference().unwrap();
    doc.get_dictionary_mut(reply).unwrap().set(
        "Contents",
        annotations::text_string("Changed in another reader"),
    );
    let added = doc.add_object(dictionary! {"Type"=>"Annot", "Subtype"=>"Text", "Rect"=>vec![20.into(),20.into(),44.into(),44.into()],
        "IRT"=>reply, "RT"=>"R", "T"=>annotations::text_string("Reviewer"), "Contents"=>annotations::text_string("A reply to a reply"),
        "CreationDate"=>Object::string_literal("D:20261114160000+03'00'")});
    values.push(added.into());
    let page = *doc.get_pages().values().next().unwrap();
    doc.get_dictionary_mut(page).unwrap().set("Annots", values);
    let mut bytes = vec![];
    doc.save_to(&mut bytes).unwrap();
    let loaded = annotations::load(&doc);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].object.review.replies.len(), 3);
    assert!(
        loaded[0]
            .object
            .review
            .replies
            .iter()
            .any(|r| r.text == "Changed in another reader")
    );
    assert!(
        loaded[0]
            .object
            .review
            .replies
            .iter()
            .any(|r| r.text == "A reply to a reply")
    );
    let saved = annotations::export(&bytes, &loaded).unwrap();
    let saved = Document::load_mem(&saved).unwrap();
    assert_eq!(list(&saved).len(), 5);
    assert_eq!(annotations::load(&saved), loaded);
    assert!(doc.get_dictionary(root).is_ok());
}
#[test]
fn unsupported_reply_data_preserves_the_entire_foreign_thread() {
    let bytes = annotations::export(&convert::blank(300., 400.).unwrap(), &[note()]).unwrap();
    let mut doc = Document::load_mem(&bytes).unwrap();
    let values = list(&doc);
    doc.get_dictionary_mut(values[1].as_reference().unwrap())
        .unwrap()
        .set("RT", "Group");
    assert!(annotations::load(&doc).is_empty());
    let mut bytes = vec![];
    doc.save_to(&mut bytes).unwrap();
    let saved = annotations::export(&bytes, &[]).unwrap();
    assert_eq!(list(&Document::load_mem(&saved).unwrap()).len(), 4);
}

#[test]
fn legacy_threads_and_external_popup_objects_are_preserved_without_orphans() {
    let a = note();
    let bytes = annotations::export(
        &convert::blank(300., 400.).unwrap(),
        std::slice::from_ref(&a),
    )
    .unwrap();
    let mut doc = Document::load_mem(&bytes).unwrap();
    let root = list(&doc)[0].as_reference().unwrap();
    let d = doc.get_dictionary_mut(root).unwrap();
    d.remove(b"MyPDFThreadVersion");
    d.get_mut(b"MyPDFState")
        .unwrap()
        .as_dict_mut()
        .unwrap()
        .remove(b"MyPDFThreadVersion");
    let popup = doc.add_object(dictionary! {"Type"=>"Annot", "Subtype"=>"Popup", "Parent"=>root, "Rect"=>vec![0.into(),0.into(),100.into(),100.into()]});
    let page = *doc.get_pages().values().next().unwrap();
    doc.get_dictionary_mut(page)
        .unwrap()
        .set("Annots", vec![root.into(), popup.into()]);
    let loaded = annotations::load(&doc);
    assert_eq!(loaded, vec![a]);
    let mut bytes = vec![];
    doc.save_to(&mut bytes).unwrap();
    let saved = annotations::export(&bytes, &loaded).unwrap();
    let doc = Document::load_mem(&saved).unwrap();
    assert_eq!(list(&doc).len(), 4);
    assert_eq!(annotations::load(&doc), loaded);
}

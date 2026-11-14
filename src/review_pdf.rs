//! Standard PDF reply annotations. Native editing data is supplementary, never the only copy.
use crate::{
    annotations::{Annotation, text_string},
    review::{Reply, Review},
};
use lopdf::{Document, Object, ObjectId, dictionary};
use std::collections::{HashMap, HashSet};

pub(crate) fn pdf_date(value: i64) -> Option<Object> {
    if value <= 0 {
        return None;
    }
    let t = time::OffsetDateTime::from_unix_timestamp(value).ok()?;
    Some(Object::string_literal(format!(
        "D:{:04}{:02}{:02}{:02}{:02}{:02}Z",
        t.year(),
        t.month() as u8,
        t.day(),
        t.hour(),
        t.minute(),
        t.second()
    )))
}
fn timestamp(d: &lopdf::Dictionary) -> i64 {
    let parse = || -> Option<i64> {
        let value = d
            .get(b"CreationDate")
            .or_else(|_| d.get(b"M"))
            .ok()?
            .as_str()
            .ok()?;
        let s = std::str::from_utf8(value).ok()?.strip_prefix("D:")?;
        if !s.is_ascii() {
            return None;
        }
        let n = |start, end| s.get(start..end)?.parse::<u8>().ok();
        let year = s.get(..4)?.parse().ok()?;
        let date = time::Date::from_calendar_date(
            year,
            n(4, 6).unwrap_or(1).try_into().ok()?,
            n(6, 8).unwrap_or(1),
        )
        .ok()?;
        let t = date
            .with_hms(
                n(8, 10).unwrap_or(0),
                n(10, 12).unwrap_or(0),
                n(12, 14).unwrap_or(0),
            )
            .ok()?;
        let offset = match s.as_bytes().get(14) {
            Some(b'+') | Some(b'-') => {
                let sign = if s.as_bytes()[14] == b'-' { -1 } else { 1 };
                time::UtcOffset::from_hms(
                    n(15, 17)? as i8 * sign,
                    n(18, 20).unwrap_or(0) as i8 * sign,
                    0,
                )
                .ok()?
            }
            _ => time::UtcOffset::UTC,
        };
        Some(t.assume_offset(offset).unix_timestamp())
    };
    parse().unwrap_or(0)
}
pub(crate) fn append(
    doc: &mut Document,
    page: ObjectId,
    parent: ObjectId,
    rect: Object,
    a: &Annotation,
) -> Vec<Object> {
    let mut result = vec![];
    for (i, reply) in a.object.review.replies.iter().enumerate() {
        let mut d = dictionary! {"Type"=>"Annot", "Subtype"=>"Text", "Rect"=>rect.clone(), "P"=>page,
        "IRT"=>parent, "RT"=>"R", "Name"=>"Comment", "F"=> (if a.object.hidden {2} else {4} | if a.object.locked {128} else {0}),
        "T"=>text_string(&reply.author), "Contents"=>text_string(&reply.text),
        "NM"=>Object::string_literal(format!("MyPDF-{}-reply-{i}", a.id))};
        if let Some(date) = pdf_date(reply.created) {
            d.set("CreationDate", date.clone());
            d.set("M", date);
        }
        result.push(Object::Reference(doc.add_object(d)));
    }
    if a.object.review.resolved {
        let mut d = dictionary! {"Type"=>"Annot", "Subtype"=>"Text", "Rect"=>rect, "P"=>page,
        "IRT"=>parent, "RT"=>"R", "F"=>2, "T"=>text_string(&a.object.review.author),
        "StateModel"=>Object::string_literal("Review"), "State"=>Object::string_literal("Completed"),
        "NM"=>Object::string_literal(format!("MyPDF-{}-status", a.id))};
        if let Some(date) = pdf_date(a.object.review.created) {
            d.set("M", date);
        }
        result.push(Object::Reference(doc.add_object(d)));
    }
    result
}

pub(crate) struct Threads<'a> {
    doc: &'a Document,
    children: HashMap<ObjectId, Vec<ObjectId>>,
}
impl<'a> Threads<'a> {
    pub fn new(doc: &'a Document, values: &[Object]) -> Self {
        let mut children: HashMap<_, Vec<_>> = HashMap::new();
        for value in values {
            if let Ok(id) = value.as_reference()
                && let Ok(d) = doc.get_dictionary(id)
                && let Ok(parent) = d
                    .get(b"IRT")
                    .or_else(|error| {
                        if d.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Popup") {
                            d.get(b"Parent")
                        } else {
                            Err(error)
                        }
                    })
                    .and_then(Object::as_reference)
            {
                children.entry(parent).or_default().push(id);
            }
        }
        Self { doc, children }
    }
    /// Only claim a thread when every child is representable. Otherwise preserve
    /// the complete original PDF objects as foreign annotations, without data loss.
    pub fn read(&self, parent: ObjectId, review: &mut Review) -> Option<HashSet<ObjectId>> {
        let mut seen = HashSet::new();
        let mut todo = self.children.get(&parent).cloned().unwrap_or_default();
        let mut replies = vec![];
        let mut resolved = false;
        let mut status_time = i64::MIN;
        while let Some(id) = todo.pop() {
            if id == parent || !seen.insert(id) || seen.len() > 512 {
                return None;
            }
            let d = self.doc.get_dictionary(id).ok()?;
            if d.get(b"Subtype").and_then(Object::as_name).ok() == Some(b"Popup") {
                if d.has(b"IRT") {
                    return None;
                }
                continue;
            }
            if d.get(b"Subtype").and_then(Object::as_name).ok()? != b"Text"
                || d.get(b"RT").is_ok_and(|v| v.as_name().ok() != Some(b"R"))
            {
                return None;
            }
            let text = |key: &[u8]| -> Option<String> {
                match d.get(key) {
                    Ok(v) => {
                        if v.as_str().ok()?.len() > 32_000 {
                            return None;
                        }
                        lopdf::decode_text_string(v).ok()
                    }
                    Err(_) => Some(String::new()),
                }
            };
            if d.has(b"StateModel") || d.has(b"State") {
                if text(b"StateModel")? != "Review" {
                    return None;
                }
                let state = text(b"State")?;
                if !["Completed", "None"].contains(&state.as_str()) {
                    return None;
                }
                let date = timestamp(d);
                if date >= status_time {
                    resolved = state == "Completed";
                    status_time = date;
                }
            } else {
                let author = text(b"T")?;
                let text = text(b"Contents")?;
                if author.chars().count() > 200 || text.len() > 16000 || replies.len() >= 100 {
                    return None;
                }
                replies.push(Reply {
                    author,
                    created: timestamp(d),
                    text,
                });
            }
            if let Some(children) = self.children.get(&id) {
                todo.extend(children.iter().rev());
            }
        }
        replies.reverse();
        let standard = self
            .doc
            .get_dictionary(parent)
            .ok()?
            .has(b"MyPDFThreadVersion");
        if !standard {
            // Older MyPDF files store their previous replies in native metadata.
            // Preserve those while incorporating new standard replies from other readers.
            for reply in &review.replies {
                if !replies.contains(reply) {
                    replies.push(reply.clone());
                }
            }
            if status_time == i64::MIN {
                resolved = review.resolved;
            }
        }
        if replies.len() > 100 {
            return None;
        }
        replies.sort_by_key(|r| r.created);
        review.replies = replies;
        review.resolved = resolved;
        Some(seen)
    }
}

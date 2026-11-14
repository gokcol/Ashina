//! Shared comment identity, timeline and reply composer for the inspector and review window.
use super::*;
use mypdf_milestone0::{
    annotations::Annotation,
    review::{Reply, timestamp},
};

#[derive(Default)]
pub(super) struct State {
    pub author: String,
    pub editing: bool,
    root: PathBuf,
    drafts: HashMap<(u64, String), String>,
    error: Option<String>,
}
pub(super) enum Action {
    Reply(Reply),
    Resolve(bool),
}
impl State {
    pub fn load(root: PathBuf) -> Self {
        let (author, error) = match preferences::load::<String>(&root, "comment-author.json") {
            Ok(name) => (name.chars().take(200).collect(), None),
            Err(e) => (
                String::new(),
                Some(format!("Could not load your comment name: {e}")),
            ),
        };
        Self {
            author,
            root,
            error,
            ..Default::default()
        }
    }
    pub fn posted(&mut self, tab: u64, id: &str) {
        self.drafts.remove(&(tab, id.into()));
    }
    pub fn composer(
        &mut self,
        ui: &mut egui::Ui,
        tab: u64,
        note: &Annotation,
        editable: bool,
    ) -> Option<Action> {
        let mut action = None;
        self.editing = false;
        ui.push_id(("reply_composer", tab, &note.id), |ui| {
            ui.add_enabled_ui(editable, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Reply as").small().color(theme::muted()));
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.author)
                            .desired_width(ui.available_width())
                            .hint_text("Your display name")
                            .char_limit(200),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            editable,
                            "Comment display name",
                        )
                    });
                    self.editing |= response.has_focus();
                    if response.lost_focus() {
                        self.save_author();
                    }
                });
                let draft = self.drafts.entry((tab, note.id.clone())).or_default();
                let response = ui.add(
                    egui::TextEdit::multiline(draft)
                        .id_salt("reply_text")
                        .desired_width(f32::INFINITY)
                        .desired_rows(2)
                        .char_limit(4000)
                        .hint_text("Write a reply…"),
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, editable, "Write a reply")
                });
                self.editing |= response.has_focus();
                let full = note.object.review.replies.len() >= 100;
                ui.horizontal(|ui| {
                    let resolved = note.object.review.resolved;
                    if ui
                        .add(ActionButton::new(
                            Icon::Check,
                            if resolved { "Reopen" } else { "Resolve" },
                        ))
                        .clicked()
                    {
                        action = Some(Action::Resolve(!resolved));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !draft.trim().is_empty() && !self.author.trim().is_empty() && !full,
                                ActionButton::new(Icon::Note, "Post reply").primary(),
                            )
                            .clicked()
                        {
                            action = Some(Action::Reply(Reply {
                                author: self.author.trim().into(),
                                created: timestamp(),
                                text: draft.trim().into(),
                            }));
                        }
                    });
                });
                if full {
                    ui.weak("This thread has reached its 100-reply limit.");
                }
            });
            if !editable {
                ui.weak("Read-only conversation");
            }
            if let Some(error) = &self.error {
                ui.colored_label(theme::danger(), error);
            }
        });
        if matches!(action, Some(Action::Reply(_))) {
            self.save_author();
        }
        action
    }
    fn save_author(&mut self) {
        self.error = preferences::save(&self.root, "comment-author.json", &self.author)
            .err()
            .map(|e| format!("Could not remember your comment name: {e}"));
    }
}

fn display_name(name: &str) -> &str {
    if name.trim().is_empty() {
        "Unknown author"
    } else {
        name.trim()
    }
}
fn initials(name: &str) -> String {
    let parts: Vec<_> = name.split_whitespace().collect();
    let chars = parts
        .first()
        .into_iter()
        .chain(parts.last().filter(|_| parts.len() > 1));
    let value: String = chars
        .filter_map(|s| s.chars().next())
        .flat_map(char::to_uppercase)
        .collect();
    if value.is_empty() { "?".into() } else { value }
}
fn date_label(value: i64) -> String {
    if value <= 0 {
        return "Date unavailable".into();
    }
    let Ok(t) = time::OffsetDateTime::from_unix_timestamp(value) else {
        return "Date unavailable".into();
    };
    // Use an explicit timezone instead of guessing the document author's location.
    format!(
        "{} {} {} · {:02}:{:02} UTC",
        t.day(),
        &t.month().to_string()[..3],
        t.year(),
        t.hour(),
        t.minute()
    )
}
pub(super) fn author_header(ui: &mut egui::Ui, author: &str, created: i64) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(30., 30.), egui::Sense::hover());
        ui.painter()
            .circle_filled(rect.center(), 15., theme::accent_soft());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            initials(author),
            FontId::proportional(11.),
            theme::accent(),
        );
        ui.vertical(|ui| {
            ui.label(RichText::new(display_name(author)).strong());
            ui.label(
                RichText::new(date_label(created))
                    .small()
                    .color(theme::muted()),
            );
        });
    });
}
pub(super) fn replies(ui: &mut egui::Ui, note: &Annotation) {
    let replies = &note.object.review.replies;
    if replies.is_empty() {
        ui.add_space(8.);
        ui.weak("No replies yet. Start the conversation below.");
        return;
    }
    egui::CollapsingHeader::new(format!(
        "{} {}",
        replies.len(),
        if replies.len() == 1 {
            "reply"
        } else {
            "replies"
        }
    ))
    .id_salt(("thread_replies", &note.id))
    .default_open(true)
    .show(ui, |ui| {
        let mut ordered: Vec<_> = replies.iter().collect();
        ordered.sort_by_key(|reply| reply.created);
        for reply in ordered {
            let top = ui.cursor().min;
            let response = egui::Frame::new()
                .inner_margin(egui::Margin {
                    left: 14,
                    right: 0,
                    top: 8,
                    bottom: 12,
                })
                .show(ui, |ui| {
                    author_header(ui, &reply.author, reply.created);
                    ui.add_space(5.);
                    ui.add(egui::Label::new(&reply.text).wrap().selectable(true));
                });
            ui.painter().line_segment(
                [
                    top + vec2(2., 0.),
                    pos2(top.x + 2., response.response.rect.bottom()),
                ],
                Stroke::new(1.5, theme::line()),
            );
        }
    });
}
impl Viewer {
    pub(super) fn apply_comment_action(
        &mut self,
        tab_id: u64,
        note_id: &str,
        action: Action,
    ) -> bool {
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) else {
            return false;
        };
        let Some(session) = &tab.session else {
            return false;
        };
        if !session.editable || self.content.busy || self.organizer.busy {
            return false;
        }
        let Some(mut note) = session
            .annotations
            .iter()
            .find(|a| a.id == note_id && !a.object.locked)
            .cloned()
        else {
            return false;
        };
        let is_reply = matches!(action, Action::Reply(_));
        match action {
            Action::Reply(reply) => {
                if note.object.review.replies.len() >= 100
                    || reply.author.trim().is_empty()
                    || reply.text.trim().is_empty()
                {
                    return false;
                }
                note.object.review.replies.push(reply);
            }
            Action::Resolve(value) => note.object.review.resolved = value,
        }
        if selection::commit(tab, &[note.clone()], None, &mut self.status).is_none() {
            return false;
        }
        tab.session.as_mut().unwrap().name_last_change(if is_reply {
            "Reply to comment"
        } else {
            "Change comment status"
        });
        if self.editor.draft.as_ref().is_some_and(|a| a.id == note_id)
            && self.tabs[self.active].id == tab_id
        {
            self.editor.draft = Some(note);
            self.editor.property_edit = None;
        }
        if is_reply {
            self.editor.comment_thread.posted(tab_id, note_id);
        }
        true
    }
}

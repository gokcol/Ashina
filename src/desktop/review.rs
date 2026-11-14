//! Review threads stay with the annotation and share document Undo and safe saving.
use super::*;
use mypdf_milestone0::annotations::Annotation;
#[derive(Default)]
pub(super) struct State {
    pub open: bool,
    pub selected: Option<String>,
    filter: String,
    status: u8,
    color: Option<[f32; 3]>,
    error: Option<String>,
    job: Option<std::sync::mpsc::Receiver<Result<JobResult, String>>>,
    pending: Option<PendingImport>,
}
enum JobResult {
    Saved,
    Import(PendingImport),
}
struct PendingImport {
    tab: u64,
    revision: u64,
    incoming: Vec<Annotation>,
    added: usize,
    conflicts: usize,
}
impl State {
    pub(super) fn owns_input(&self) -> bool {
        self.pending.is_some()
    }
}
impl Viewer {
    pub(super) fn review_window(&mut self, ctx: &egui::Context) {
        self.review_jobs(ctx);
        if !self.review.open {
            return;
        }
        let Some(t) = self.tabs.get(self.active) else {
            self.review.open = false;
            return;
        };
        let Some(s) = &t.session else { return };
        let id = t.id;
        let marks = &s.annotations;
        let author_filter = self.review.filter.to_lowercase();
        let editable = s.editable && !self.content.busy && !self.organizer.busy;
        let filtered: Vec<&Annotation> = marks
            .iter()
            .filter(|a| {
                if !mypdf_milestone0::review_tools::is_comment(a) {
                    return false;
                }
                let r = &a.object.review;
                (self.review.status == 0
                    || (self.review.status == 1 && !r.resolved)
                    || (self.review.status == 2 && r.resolved))
                    && (self.review.filter.is_empty()
                        || r.author.to_lowercase().contains(&author_filter)
                        || r.replies
                            .iter()
                            .any(|r| r.author.to_lowercase().contains(&author_filter)))
                    && self.review.color.is_none_or(|c| c == a.style.color)
            })
            .collect();
        let mut open = true;
        let mut change = None;
        let mut jump = None;
        let mut action = None;
        egui::Window::new("Review comments")
            .open(&mut open)
            .default_size(vec2(700., 480.))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Author");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.review.filter)
                            .hint_text("All authors")
                            .desired_width(130.),
                    );
                    egui::ComboBox::from_id_salt("review_status")
                        .selected_text(["All", "Open", "Resolved"][self.review.status as usize])
                        .show_ui(ui, |ui| {
                            for (i, label) in ["All", "Open", "Resolved"].iter().enumerate() {
                                ui.selectable_value(&mut self.review.status, i as u8, *label);
                            }
                        });
                    egui::ComboBox::from_id_salt("review_color")
                        .selected_text(if self.review.color.is_some() {
                            "Selected color"
                        } else {
                            "All colors"
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.review.color, None, "All colors");
                            let mut colors = vec![];
                            for a in marks {
                                if !colors.contains(&a.style.color) {
                                    colors.push(a.style.color);
                                }
                            }
                            for c in colors {
                                if ui
                                    .colored_label(
                                        Color32::from_rgb(
                                            (c[0] * 255.) as u8,
                                            (c[1] * 255.) as u8,
                                            (c[2] * 255.) as u8,
                                        ),
                                        "● Select color",
                                    )
                                    .interact(egui::Sense::click())
                                    .clicked()
                                {
                                    self.review.color = Some(c);
                                    ui.close();
                                }
                            }
                        });
                    let actions = ui.add_enabled(
                        self.review.job.is_none() && self.review.pending.is_none(),
                        ActionButton::new(Icon::More, "Review actions").dropdown(),
                    );
                    egui::Popup::menu(&actions).show(|ui| {
                        for (key, label) in [
                            ("pdf", "Save PDF summary…"),
                            ("csv", "Save CSV summary…"),
                            ("export", "Export comments…"),
                            ("import", "Import comments…"),
                        ] {
                            if ui
                                .add_enabled(
                                    (key != "import" || editable)
                                        && (key == "import" || !filtered.is_empty()),
                                    ActionButton::new(
                                        if key == "import" {
                                            Icon::Open
                                        } else {
                                            Icon::Export
                                        },
                                        label,
                                    ),
                                )
                                .clicked()
                            {
                                action = Some(key);
                                ui.close();
                            }
                        }
                        ui.separator();
                        ui.weak("Comment exchange uses MyPDF review files.");
                    });
                    if self.review.job.is_some() {
                        ui.spinner();
                    }
                });
                ui.separator();
                ui.columns(2, |cols| {
                    egui::ScrollArea::vertical()
                        .id_salt("review_list")
                        .max_height(440.)
                        .show(&mut cols[0], |ui| {
                            if filtered.is_empty() {
                                ui.weak("No annotations match these filters.");
                            }
                            for a in &filtered {
                                let title = a
                                    .kind
                                    .text()
                                    .filter(|s| !s.is_empty())
                                    .unwrap_or(a.kind.label());
                                let title: String = title.chars().take(65).collect();
                                let label = format!(
                                    "{}  Page {} · {}",
                                    if a.object.review.resolved {
                                        "✓"
                                    } else {
                                        "○"
                                    },
                                    a.page + 1,
                                    title
                                );
                                if ui
                                    .selectable_label(
                                        self.review.selected.as_ref() == Some(&a.id),
                                        label,
                                    )
                                    .clicked()
                                {
                                    self.review.selected = Some(a.id.clone());
                                    jump = Some(a.page);
                                }
                            }
                        });
                    let ui = &mut cols[1];
                    if let Some(a) = marks
                        .iter()
                        .find(|a| Some(&a.id) == self.review.selected.as_ref())
                    {
                        ui::title(ui, a.kind.label(), 17.);
                        egui::ScrollArea::vertical()
                            .id_salt("review_replies")
                            .max_height(260.)
                            .show(ui, |ui| {
                                comment_thread::author_header(
                                    ui,
                                    &a.object.review.author,
                                    a.object.review.created,
                                );
                                if let Some(text) = a.kind.text() {
                                    ui.label(text);
                                }
                                ui.weak(format!("Page {}", a.page + 1));
                                comment_thread::replies(ui, a);
                            });
                        ui.separator();
                        if let Some(action) = self.editor.comment_thread.composer(
                            ui,
                            id,
                            a,
                            editable && !a.object.locked,
                        ) {
                            change = Some((a.id.clone(), action));
                        }
                    } else {
                        ui.weak("Select an annotation to read or reply.");
                    }
                });
                if let Some(e) = &self.review.error {
                    ui.colored_label(theme::danger(), e);
                }
            });
        let export = action.map(|action| (action, filtered.into_iter().cloned().collect()));
        self.review.open = open;
        if let Some(t) = self.tabs.iter_mut().find(|t| t.id == id)
            && let Some(page) = jump
        {
            t.jump(page);
        }
        if let Some((note, action)) = change {
            // Flush a note editor before applying a reply to its latest revision.
            if self.apply_pending_annotation() {
                self.apply_comment_action(id, &note, action);
            }
        }
        if let Some((action, marks)) = export {
            self.review_action(action, marks);
        }
    }

    fn review_action(&mut self, action: &str, marks: Vec<Annotation>) {
        if !self.apply_pending_annotation() {
            return;
        }
        let Some(tab) = self.tabs.get(self.active) else {
            return;
        };
        if self.content.busy
            || self.organizer.busy
            || self.content.pending_text().is_some()
            || self.pending_form_edits(tab.id)
            || self.table.has_editor(tab.id)
        {
            self.review.error =
                Some("Finish the current edit before exchanging or summarizing comments.".into());
            return;
        }
        let Some(session) = &tab.session else { return };
        let tab_id = tab.id;
        let revision = session.revision();
        let bytes = match session.snapshot_bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                self.review.error = Some(e.to_string());
                return;
            }
        };
        let existing = session.annotations.clone();
        let path = menu::with_file_dialog(self._native_menu.as_ref(), || {
            if action == "import" {
                rfd::FileDialog::new()
                    .set_title("Import MyPDF review comments")
                    .add_filter("MyPDF review", &["mypdf-review"])
                    .pick_file()
            } else {
                let (name, extension) = match action {
                    "pdf" => ("Review summary.pdf", "pdf"),
                    "csv" => ("Review summary.csv", "csv"),
                    _ => ("Review comments.mypdf-review", "mypdf-review"),
                };
                rfd::FileDialog::new()
                    .set_title("Save review copy")
                    .set_file_name(name)
                    .add_filter("Review file", &[extension])
                    .save_file()
            }
        });
        let Some(path) = path else { return };
        let (sender, receiver) = std::sync::mpsc::channel();
        self.review.job = Some(receiver);
        self.review.error = None;
        let action = action.to_owned();
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || -> anyhow::Result<JobResult> {
                    use mypdf_milestone0::{convert, review_tools};
                    if action == "import" {
                        let input = convert::read_bounded(&path, review_tools::MAX_EXCHANGE_BYTES)?;
                        let incoming = review_tools::import_comments(&bytes, &input)?;
                        let (_, added, conflicts) =
                            review_tools::merge_comments(&existing, &incoming);
                        return Ok(JobResult::Import(PendingImport {
                            tab: tab_id,
                            revision,
                            incoming,
                            added,
                            conflicts,
                        }));
                    }
                    let output = match action.as_str() {
                        "pdf" => review_tools::pdf_summary(&bytes, &marks, |_, _| true)?,
                        "csv" => mypdf_milestone0::review::csv(&marks).into_bytes(),
                        _ => review_tools::export_comments(&bytes, &marks)?,
                    };
                    viewer::save_copy(&path, &output)?;
                    Ok(JobResult::Saved)
                },
            ))
            .map_err(|_| "Could not process the review file. The document is unchanged.".to_owned())
            .and_then(|r| r.map_err(|e| e.to_string()));
            let _ = sender.send(result);
        });
    }
    fn review_jobs(&mut self, ctx: &egui::Context) {
        if let Some(job) = &self.review.job {
            match job.try_recv() {
                Ok(result) => {
                    self.review.job = None;
                    match result {
                        Ok(JobResult::Saved) => {
                            self.status = Some(("Review copy saved".into(), false))
                        }
                        Ok(JobResult::Import(pending)) => self.review.pending = Some(pending),
                        Err(e) => {
                            self.review.error = Some(e);
                            self.review.open = true;
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.review.job = None;
                    self.review.error = Some("Review operation did not finish.".into());
                    self.review.open = true;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        let Some(pending) = &self.review.pending else {
            return;
        };
        let mut import = false;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("review_import_preview")).frame(ui::modal_frame()).show(ctx,|ui|{
            ui.set_width(400.);ui.heading("Import review comments");
            ui.label(format!("{} new comments · {} changed versions kept as separate comments",pending.added,pending.conflicts));
            ui.weak("Exact duplicates are skipped. Your existing comments stay unchanged. You can undo the entire import.");
            ui.horizontal(|ui|{cancel=ui.button("Cancel").clicked();import=ui.add_enabled(pending.added>0,ActionButton::new(Icon::Open,"Import comments").primary()).clicked();});
        });
        if import {
            let pending = self.review.pending.take().unwrap();
            let result = (|| -> anyhow::Result<()> {
                let tab = self
                    .tabs
                    .iter_mut()
                    .find(|t| t.id == pending.tab)
                    .ok_or_else(|| anyhow::anyhow!("The document was closed"))?;
                let session = tab
                    .session
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("The document is not ready"))?;
                anyhow::ensure!(
                    session.editable && session.revision() == pending.revision,
                    "The document changed. Import again to compare with the latest edits."
                );
                let (merged, _, _) = mypdf_milestone0::review_tools::merge_comments(
                    &session.annotations,
                    &pending.incoming,
                );
                session.commit(merged)?;
                session.name_last_change("Import review comments");
                Ok(())
            })();
            match result {
                Ok(()) => self.status = Some(("Review comments imported".into(), false)),
                Err(e) => self.review.error = Some(e.to_string()),
            }
        } else if cancel {
            self.review.pending = None;
        }
    }
}

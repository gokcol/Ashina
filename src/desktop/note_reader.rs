//! A PDF comment's icon is its page anchor; its readable contents live here.
use super::*;

pub(in crate::desktop) struct Popover {
    tab: u64,
    note: String,
    position: egui::Pos2,
}

impl Viewer {
    pub(in crate::desktop) fn open_note_reader(&mut self, tab_id: u64, note_id: &str) -> bool {
        let Some(tab) = self.tabs.get(self.active).filter(|tab| tab.id == tab_id) else {
            return false;
        };
        let Some(note) = tab
            .session
            .as_ref()
            .and_then(|session| {
                session.annotations.iter().find(|a| {
                    a.id == note_id && !a.object.hidden && matches!(a.kind, Kind::Note { .. })
                })
            })
            .cloned()
        else {
            return false;
        };
        if self.content.pending_text().is_some() || self.pending_form_edits(tab_id) {
            self.status = Some((
                "Finish the current text or form edit before opening the note.".into(),
                true,
            ));
            return false;
        }
        if self.workbench.crop.is_some() {
            self.status = Some((
                "Finish or cancel the current crop before opening the note.".into(),
                true,
            ));
            return false;
        }
        if !self.apply_pending_annotation() {
            return false;
        }
        let tab = &mut self.tabs[self.active];
        // Applying a pending edit may have just updated this same note.
        let note = tab
            .session
            .as_ref()
            .and_then(|session| session.annotations.iter().find(|a| a.id == note.id))
            .cloned()
            .unwrap_or(note);
        if tab.page != note.page {
            tab.jump(note.page);
        }
        tab.selected_annotation = Some(note.id.clone());
        tab.group.clear();
        tab.signatures.panel = false;
        self.editor.draft = Some(note);
        self.editor.property_edit = None;
        self.editor.text_selection = None;
        self.editor.library_open = false;
        self.editor.tool = Tool::Select;
        if !matches!(self.editor.mode, Mode::Read | Mode::Annotate | Mode::Draw) {
            self.editor.mode = Mode::Annotate;
        }
        self.translation.open = false;
        self.editor.note_popup = None;
        self.inspector = true;
        true
    }

    /// Page hit testing runs after panel layout, so open on the next frame.
    pub(in crate::desktop) fn note_reader_requests(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.pointer.any_down() || i.pointer.any_released()) {
            self.editor.note_pointer_blocked = false;
        }
        if let Some((tab, note)) = self.editor.note_requested.take() {
            if self.zen.is_some() {
                self.toggle_zen(ctx);
            }
            if self.open_note_reader(tab, &note)
                && self.selected_note().is_some_and(|a| a.voice.is_none())
            {
                self.inspector = false;
                self.editor.note_popup = Some(Popover {
                    tab,
                    note,
                    position: ctx
                        .pointer_latest_pos()
                        .unwrap_or(ctx.content_rect().center())
                        + vec2(12., 12.),
                });
            }
        }
        self.note_popover(ctx);
    }

    fn note_popover(&mut self, ctx: &egui::Context) {
        let Some(popover) = &self.editor.note_popup else {
            return;
        };
        let Some(selected) = self.selected_note().filter(|a| {
            a.id == popover.note
                && self.tabs[self.active].id == popover.tab
                && matches!(self.editor.mode, Mode::Read | Mode::Annotate | Mode::Draw)
        }) else {
            self.editor.note_popup = None;
            return;
        };
        let position = popover.position;
        let editable = self.tabs[self.active]
            .session
            .as_ref()
            .is_some_and(|s| s.editable)
            && !selected.object.locked;
        let tab_id = self.tabs[self.active].id;
        if self
            .editor
            .draft
            .as_ref()
            .is_none_or(|a| a.id != selected.id)
        {
            self.editor.draft = Some(selected.clone());
        }
        let mut close = false;
        let mut panel = false;
        let mut review = false;
        let mut focus = None;
        let mut open = true;
        // Popup owns outside clicks. Also block page interaction for its closing
        // frame so dismissing a note never places another annotation underneath.
        self.editor.note_pointer_blocked = true;
        let id = egui::Id::new(("note_popover", tab_id, &selected.id));
        egui::Popup::new(
            id,
            ctx.clone(),
            egui::Rect::from_min_size(position, vec2(1., 1.)),
            egui::LayerId::background(),
        )
        .open_bool(&mut open)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_width(300_f32.min(ctx.content_rect().width() - 32.));
            ui.spacing_mut().item_spacing = vec2(8., 9.);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(selected.suggestion.as_ref().map_or("Note", |s| s.label()))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    close = ui
                        .add(ActionButton::new(Icon::Close, "Close note").compact())
                        .clicked();
                });
            });
            let author = &selected.object.review.author;
            ui.weak(format!(
                "Page {}{}",
                selected.page + 1,
                if author.is_empty() {
                    String::new()
                } else {
                    format!(" · {author}")
                }
            ));
            if let Some(suggestion) = &selected.suggestion {
                if !suggestion.original.is_empty() {
                    ui.collapsing("Original wording", |ui| {
                        ui.label(&suggestion.original);
                    });
                }
                ui.weak("Proposed wording · document text is unchanged");
            }
            let text = self.editor.draft.as_mut().unwrap().kind.text_mut().unwrap();
            egui::ScrollArea::vertical()
                .max_height(180.)
                .show(ui, |ui| {
                    let response = ui.add_enabled(
                        editable,
                        egui::TextEdit::multiline(text)
                            .desired_width(f32::INFINITY)
                            .desired_rows(5)
                            .char_limit(8000)
                            .hint_text("Write a note…"),
                    );
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, editable, "Note text")
                    });
                    if response.has_focus() {
                        focus = Some(response.id);
                    }
                });
            ui.horizontal(|ui| {
                panel = ui
                    .add(ActionButton::new(Icon::Note, "Open in panel"))
                    .clicked();
                review = ui.add(ActionButton::new(Icon::Note, "Replies")).clicked();
            });
            ui.weak(if selected.object.review.resolved {
                "Resolved"
            } else {
                "Edits are kept when you close this note."
            });
        });
        self.editor.text_editing |= focus.is_some();
        let committed = self.commit_note_draft(
            &selected,
            focus,
            ctx.input(|i| i.pointer.any_down() || i.pointer.any_released()),
        );
        if committed && (close || !open || panel || review) {
            self.editor.note_popup = None;
            if panel {
                self.inspector = true;
            } else if review {
                self.review.open = true;
                self.review.selected = Some(selected.id);
            } else {
                selection::set(&mut self.tabs[self.active], &mut self.editor, vec![]);
            }
        }
    }

    pub(super) fn selected_note(&self) -> Option<Annotation> {
        let tab = self.tabs.get(self.active)?;
        if tab.group.len() >= 2 {
            return None;
        }
        let id = tab.selected_annotation.as_ref()?;
        tab.session
            .as_ref()?
            .annotations
            .iter()
            .find(|a| &a.id == id && !a.object.hidden && matches!(a.kind, Kind::Note { .. }))
            .cloned()
    }

    pub(super) fn note_inspector(&mut self, ctx: &egui::Context, selected: Annotation) {
        if selected.voice.is_some() {
            self.voice_inspector(ctx, selected);
            return;
        }
        let tab_id = self.tabs[self.active].id;
        let editable = self.tabs[self.active]
            .session
            .as_ref()
            .is_some_and(|s| s.editable)
            && !selected.object.locked;
        if self
            .editor
            .draft
            .as_ref()
            .is_none_or(|a| a.id != selected.id)
        {
            self.editor.draft = Some(selected.clone());
        }
        if ctx.input(|i| i.pointer.any_pressed()) {
            self.editor.property_edit = None;
        }
        let mut focus = None;
        let mut close = false;
        let mut delete = false;
        egui::SidePanel::right("annotation_inspector")
            .default_width(300.)
            .width_range(250.0..=440.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(theme::paper()).inner_margin(16.))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = vec2(8., 10.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(selected.suggestion.as_ref().map_or("Note", |s| s.label()))
                            .size(18.)
                            .family(ui::semibold()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close = ui
                            .add(ActionButton::new(Icon::Close, "Close note").compact())
                            .on_hover_text("Close note")
                            .clicked();
                    });
                });
                let author = selected.object.review.author.trim();
                let metadata = if author.is_empty() {
                    format!("Page {}", selected.page + 1)
                } else {
                    format!("Page {} · {author}", selected.page + 1)
                };
                ui.label(RichText::new(metadata).small().color(theme::muted()));
                ui.separator();
                if let Some(suggestion) = &selected.suggestion {
                    if !suggestion.original.is_empty() {
                        ui.collapsing("Original wording", |ui| {
                            ui.label(&suggestion.original);
                        });
                    }
                    ui.weak("Suggested wording only. The document text stays unchanged.");
                }
                let note = self.editor.draft.as_mut().unwrap();
                let Kind::Note { text } = &mut note.kind else {
                    return;
                };
                let field_id = egui::Id::new((tab_id, &selected.id, "note_text"));
                // Keep actions visible while long translations scroll inside the body.
                let body_height = (ui.available_height() - 140.).max(80.);
                egui::ScrollArea::vertical()
                    .id_salt((tab_id, &selected.id, "note_body"))
                    .auto_shrink([false, false])
                    .max_height(body_height)
                    .show(ui, |ui| {
                        let width = ui.available_width().max(40.);
                        let response = if editable {
                            ui.add_sized(
                                [width, body_height],
                                egui::TextEdit::multiline(text)
                                    .id(field_id)
                                    .desired_width(width)
                                    .font(egui::TextStyle::Body)
                                    .margin(ui::FIELD_MARGIN)
                                    .hint_text("Write a note…")
                                    .char_limit(8_000),
                            )
                        } else {
                            let mut value = text.as_str();
                            ui.add_sized(
                                [width, body_height],
                                egui::TextEdit::multiline(&mut value)
                                    .id(field_id)
                                    .desired_width(width)
                                    .font(egui::TextStyle::Body)
                                    .margin(ui::FIELD_MARGIN),
                            )
                        };
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, "Note text")
                        });
                        if response.has_focus() {
                            focus = Some(field_id);
                        }
                    });
                ui.horizontal(|ui| {
                    if ui.add(ActionButton::new(Icon::Copy, "Copy note")).clicked() {
                        ctx.copy_text(text.clone());
                    }
                    delete = ui
                        .add_enabled(
                            editable,
                            ActionButton::new(Icon::Trash, "Delete note").compact(),
                        )
                        .on_hover_text("Delete note")
                        .clicked();
                });
                ui.label(
                    RichText::new(if editable {
                        "Edits update immediately. Save the PDF to keep them."
                    } else {
                        "Read-only note · Select or copy the text."
                    })
                    .small()
                    .color(theme::muted()),
                );
                if editable {
                    ui.collapsing("Note color", |ui| {
                        color_palette(ui, &mut note.style.color);
                    });
                }
            });
        self.editor.text_editing |= focus.is_some();
        let committed = self.commit_note_draft(
            &selected,
            focus,
            ctx.input(|i| i.pointer.any_down() || i.pointer.any_released()),
        );
        if !ctx.input(|i| i.pointer.any_down()) && focus.is_none() {
            self.editor.property_edit = None;
        }
        if delete {
            self.editor_delete();
        } else if close && committed {
            selection::set(&mut self.tabs[self.active], &mut self.editor, vec![]);
        }
    }

    pub(super) fn commit_note_draft(
        &mut self,
        selected: &Annotation,
        focus: Option<egui::Id>,
        pointer_active: bool,
    ) -> bool {
        let tab = &mut self.tabs[self.active];
        let Some(session) = &mut tab.session else {
            return false;
        };
        let Some(draft) = self.editor.draft.as_ref().filter(|a| a.id == selected.id) else {
            return false;
        };
        if draft == selected {
            return true;
        }
        if !session.editable || selected.object.locked {
            return false;
        }
        let previous = self
            .editor
            .property_edit
            .as_ref()
            .filter(|(id, annotation, _, previous_focus)| {
                *id == tab.id
                    && *annotation == selected.id
                    && (pointer_active || focus.is_some() && focus == *previous_focus)
            })
            .map(|(_, _, revision, _)| *revision);
        let mut annotations = session.annotations.clone();
        let Some(note) = annotations.iter_mut().find(|a| a.id == selected.id) else {
            return false;
        };
        *note = draft.clone();
        match session.commit_live(annotations, previous) {
            Ok(()) => {
                session.name_last_change("Edit note");
                self.editor.property_edit =
                    Some((tab.id, selected.id.clone(), session.revision(), focus));
                self.status = None;
                true
            }
            Err(error) => {
                self.status = Some((format!("Could not update note: {error:#}"), true));
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mypdf_milestone0::{annotations, convert};

    fn fixture(editable: bool) -> (Viewer, egui::Context, Annotation) {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Viewer::new(&eframe::CreationContext::_new_kittest(ctx.clone()), vec![]);
        let text = format!(
            "Translation · English → Turkish\n\n{}\nSon cümle: Gizlilik korunur. Straße, Größe.",
            "Bu belge çevrimdışı çevrildi. İçerik yalnızca cihazınızda kalır.\n".repeat(12)
        );
        let note = Annotation::new(
            0,
            [100., 100., 124., 124.],
            Kind::Note { text },
            Style::default(),
        );
        let bytes = annotations::export(
            &convert::blank(500., 500.).unwrap(),
            std::slice::from_ref(&note),
        )
        .unwrap();
        // Exercise the saved PDF path: note Contents must survive reload, not just
        // an in-memory draft that happened to retain the translated text.
        app.open_memory(PathBuf::from("Saved note.pdf"), bytes, &ctx);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while app.tabs[0].session.is_none() {
            assert!(std::time::Instant::now() < deadline);
            app.events(&ctx);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        app.tabs[0].session.as_mut().unwrap().editable = editable;
        app.home.visible = false;
        (app, ctx, note)
    }

    fn frame(app: &mut Viewer, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(1200., 820.))),
                events,
                ..Default::default()
            },
            |ctx| {
                app.note_reader_requests(ctx);
                if app.inspector && !app.translation.open && !app.signature_panel_open() {
                    app.annotation_inspector(ctx);
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    let rect = Rect::from_min_size(pos2(40., 40.), vec2(500., 500.));
                    content::smart_select(
                        &mut app.content,
                        &mut app.editor,
                        &mut app.tabs[0],
                        ui,
                        0,
                        rect,
                        1.,
                        &mut app.inspector,
                    );
                    page_interaction(
                        &mut app.editor,
                        &mut app.tabs[0],
                        ui,
                        0,
                        rect,
                        1.,
                        &mut app.status,
                    );
                    content::overlay(
                        &mut app.content,
                        app.editor.mode,
                        &app.tabs[0],
                        ui,
                        0,
                        rect,
                        1.,
                    );
                });
            },
        )
    }

    fn click(app: &mut Viewer, ctx: &egui::Context, point: egui::Pos2) -> egui::FullOutput {
        for pressed in [true, false] {
            frame(
                app,
                ctx,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        frame(app, ctx, vec![])
    }

    fn visible_text(output: &egui::FullOutput, text: &str) -> bool {
        output.shapes.iter().any(|shape| match &shape.shape {
            egui::Shape::Text(t) => t.galley.job.text == text,
            _ => false,
        })
    }

    #[test]
    fn saved_note_click_reveals_full_contents_except_in_document_edit_mode() {
        for editable in [true, false] {
            for mode in [Mode::Read, Mode::Annotate, Mode::Edit] {
                let (mut app, ctx, note) = fixture(editable);
                let before = app.tabs[0].session.as_ref().unwrap().source.clone();
                let revision = app.tabs[0].session.as_ref().unwrap().revision();
                app.editor.mode = mode;
                app.inspector = false;
                app.translation.open = true;
                app.tabs[0].signed = !editable;
                app.tabs[0].signatures.panel = !editable;
                for _ in 0..3 {
                    frame(&mut app, &ctx, vec![]);
                }
                click(&mut app, &ctx, pos2(152., 152.));
                if mode == Mode::Edit {
                    assert!(app.tabs[0].selected_annotation.is_none());
                    assert_eq!(app.editor.mode, Mode::Edit);
                    assert_eq!(app.tabs[0].session.as_ref().unwrap().revision(), revision);
                    continue;
                }
                assert!(
                    app.editor.note_popup.is_some()
                        && !app.inspector
                        && !app.translation.open
                        && !app.signature_panel_open(),
                    "note popover must win panel precedence in {mode:?}"
                );
                assert_eq!(
                    app.tabs[0].selected_annotation.as_deref(),
                    Some(note.id.as_str())
                );
                assert_eq!(
                    app.editor.mode,
                    if mode == Mode::Edit {
                        Mode::Annotate
                    } else {
                        mode
                    }
                );
                // A compact popover scrolls long content; opening the panel
                // must expose the complete saved text without changing it.
                assert!(app.open_note_reader(app.tabs[0].id, &note.id));
                let output = frame(&mut app, &ctx, vec![]);
                let text = note.kind.text().unwrap();
                assert!(
                    visible_text(&output, text),
                    "complete note text must be laid out, including its final line in {mode:?}"
                );
                assert!(
                    output
                        .platform_output
                        .accesskit_update
                        .as_ref()
                        .unwrap()
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some("Note text"))
                );
                let session = app.tabs[0].session.as_ref().unwrap();
                assert_eq!(
                    session.revision(),
                    revision,
                    "reading must not create an Undo entry"
                );
                assert_eq!(session.source.as_ref(), before.as_ref());
                assert_eq!(session.annotations, vec![note.clone()]);
                assert!(
                    app.content.pending_text().is_none(),
                    "note click must not edit underlying page text"
                );
            }
        }
    }

    #[test]
    fn note_body_is_editable_with_grouped_undo_redo_and_readonly_copy() {
        let (mut app, ctx, note) = fixture(true);
        let tab_id = app.tabs[0].id;
        assert!(app.open_note_reader(tab_id, &note.id));
        let field = egui::Id::new((tab_id, &note.id, "note_text"));
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        ctx.memory_mut(|m| m.request_focus(field));
        frame(&mut app, &ctx, vec![]);
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            }],
        );
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text("Updated translation".into())],
        );
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text(" — second sentence.".into())],
        );
        let expected = "Updated translation — second sentence.";
        assert_eq!(
            app.tabs[0].session.as_ref().unwrap().annotations[0]
                .kind
                .text(),
            Some(expected)
        );
        assert_eq!(
            app.tabs[0].session.as_ref().unwrap().history().0,
            1,
            "continuous note typing is one document Undo step"
        );
        assert_eq!(
            app.tabs[0]
                .session
                .as_ref()
                .unwrap()
                .undo_label()
                .as_deref(),
            Some("Edit note")
        );
        app.editor_undo(false);
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(app.editor.draft.as_ref().unwrap(), &note);
        assert!(visible_text(&output, note.kind.text().unwrap()));
        app.editor_undo(true);
        let output = frame(&mut app, &ctx, vec![]);
        assert_eq!(
            app.tabs[0].selected_annotation.as_deref(),
            Some(note.id.as_str())
        );
        assert!(visible_text(&output, expected));

        app.tabs[0].session.as_mut().unwrap().editable = false;
        let revision = app.tabs[0].session.as_ref().unwrap().revision();
        frame(
            &mut app,
            &ctx,
            vec![egui::Event::Text("must not be inserted".into())],
        );
        assert_eq!(app.tabs[0].session.as_ref().unwrap().revision(), revision);
        let output = frame(&mut app, &ctx, vec![]);
        let copy = output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .into_iter()
            .find_map(|(_, node)| {
                (node.label() == Some("Copy note"))
                    .then(|| node.bounds())
                    .flatten()
            })
            .unwrap();
        let point = pos2(
            ((copy.x0 + copy.x1) / 2.) as f32,
            ((copy.y0 + copy.y1) / 2.) as f32,
        );
        let mut copied = false;
        for pressed in [true, false] {
            let output = frame(
                &mut app,
                &ctx,
                vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            copied |= output.platform_output.commands.iter().any(|command| matches!(command, egui::OutputCommand::CopyText(text) if text == expected));
        }
        assert!(copied, "readonly notes must remain fully copyable");
    }

    #[test]
    fn note_reopening_keeps_pending_content_and_rejects_stale_requests() {
        let (mut app, ctx, note) = fixture(true);
        let tab_id = app.tabs[0].id;
        assert!(app.open_note_reader(tab_id, &note.id));
        *app.editor.draft.as_mut().unwrap().kind.text_mut().unwrap() = "Pending note edit".into();
        assert!(app.open_note_reader(tab_id, &note.id));
        assert_eq!(
            app.editor.draft.as_ref().unwrap().kind.text(),
            Some("Pending note edit")
        );
        assert_eq!(
            app.tabs[0].session.as_ref().unwrap().annotations[0]
                .kind
                .text(),
            Some("Pending note edit")
        );
        app.inspector = false;
        app.translation.open = true;
        assert!(!app.open_note_reader(tab_id + 1, &note.id));
        assert!(!app.open_note_reader(tab_id, "deleted note"));
        assert!(!app.inspector && app.translation.open);
        let before = app.tabs[0].session.as_ref().unwrap().source.clone();
        app.content.paste_new_text(
            tab_id,
            0,
            app.tabs[0].pages[0],
            "Unfinished inline edit".into(),
        );
        assert!(!app.open_note_reader(tab_id, &note.id));
        assert_eq!(app.content.pending_text(), Some(tab_id));
        assert_eq!(
            app.tabs[0].session.as_ref().unwrap().source.as_ref(),
            before.as_ref()
        );
        let _ = ctx;
    }

    #[test]
    fn newly_created_note_opens_the_reader_even_with_hidden_inspector() {
        let (mut app, ctx, _) = fixture(true);
        let note = Annotation::new(
            0,
            [200., 100., 224., 124.],
            Kind::Note {
                text: "New comment".into(),
            },
            Style::default(),
        );
        app.inspector = false;
        app.translation.open = true;
        commit_new(
            &mut app.editor,
            &mut app.tabs[0],
            note.clone(),
            false,
            &mut app.status,
        );
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        let output = frame(&mut app, &ctx, vec![]);
        assert!(app.editor.note_popup.is_some() && !app.inspector && !app.translation.open);
        assert_eq!(app.tabs[0].selected_annotation.as_ref(), Some(&note.id));
        assert!(visible_text(&output, "New comment"));
    }
    #[test]
    fn popover_outside_click_only_dismisses_then_second_click_opens_note() {
        let (mut app, ctx, note) = fixture(true);
        app.editor.mode = Mode::Annotate;
        app.inspector = false;
        for _ in 0..3 {
            frame(&mut app, &ctx, vec![]);
        }
        click(&mut app, &ctx, pos2(152., 152.));
        assert!(app.editor.note_popup.is_some());
        let count = app.tabs[0].session.as_ref().unwrap().annotations.len();
        click(&mut app, &ctx, pos2(152., 152.));
        assert!(app.editor.note_popup.is_none());
        assert!(app.tabs[0].selected_annotation.is_none());
        assert_eq!(
            app.tabs[0].session.as_ref().unwrap().annotations.len(),
            count
        );
        click(&mut app, &ctx, pos2(152., 152.));
        assert_eq!(app.tabs[0].selected_annotation.as_ref(), Some(&note.id));
        assert!(app.editor.note_popup.is_some());
    }
    #[test]
    fn recording_commit_failure_keeps_audio_for_retry_without_microphone_access() {
        let (mut app, _, note) = fixture(false);
        let clip = mypdf_milestone0::voice::Clip::from_samples(&[10, -20, 30]).unwrap();
        app.editor.voice.pending_clip = Some((app.tabs[0].id, note.id, clip.clone()));
        assert!(!app.finish_voice_recording());
        assert_eq!(app.editor.voice.pending_clip.as_ref().unwrap().2, clip);
        app.tabs[0].session.as_mut().unwrap().editable = true;
        assert!(app.finish_voice_recording());
        assert!(app.editor.voice.pending_clip.is_none());
        assert_eq!(
            app.tabs[0].session.as_ref().unwrap().annotations[0].voice,
            Some(clip)
        );
    }
}

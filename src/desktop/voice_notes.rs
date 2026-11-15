//! Voice comments share the existing Comment tool and note inspector.
use super::*;
use mypdf_milestone0::voice::Clip;
use std::sync::atomic::Ordering;
fn duration(seconds: f32) -> String {
    format!("{}:{:02}", seconds as u32 / 60, seconds as u32 % 60)
}
impl Viewer {
    pub(in crate::desktop) fn finish_voice_recording(&mut self) -> bool {
        self.editor.voice.pending_permission = false;
        if let Some(recorder) = self.editor.voice.recorder.take() {
            match recorder.finish() {
                Ok(clip) => {
                    if let Some((tab, id)) = self.editor.voice.target.clone() {
                        self.editor.voice.pending_clip = Some((tab, id, clip));
                    }
                }
                Err(e) => {
                    self.editor.voice.error = Some(format!("Recording was not saved: {e}"));
                    return false;
                }
            }
        }
        let Some((tab_id, id, clip)) = self.editor.voice.pending_clip.clone() else {
            return true;
        };
        let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) else {
            self.editor.voice.error =
                Some("Recording retained in memory. Save the audio before closing SkyPDF.".into());
            return false;
        };
        let Some(mut note) = tab
            .session
            .as_ref()
            .and_then(|s| s.annotations.iter().find(|a| a.id == id))
            .cloned()
        else {
            self.editor.voice.error = Some("The note is unavailable. The recording is retained; save its audio before closing SkyPDF.".into());
            return false;
        };
        note.voice = Some(clip);
        if selection::commit(tab, &[note.clone()], None, &mut self.status).is_none() {
            self.editor.voice.error = Some("Could not attach the recording. It is retained in memory; retry or save its audio.".into());
            return false;
        }
        tab.session
            .as_mut()
            .unwrap()
            .name_last_change("Record voice note");
        if self.editor.draft.as_ref().is_some_and(|a| a.id == id) {
            self.editor.draft = Some(note);
        }
        self.editor.voice.pending_clip = None;
        self.editor.voice.error = None;
        true
    }
    pub(in crate::desktop) fn voice_tick(&mut self, ctx: &egui::Context) {
        let active = self
            .tabs
            .get(self.active)
            .and_then(|t| t.selected_annotation.as_ref().map(|id| (t.id, id.clone())));
        let interrupted = self.editor.voice.target.is_some()
            && (active != self.editor.voice.target
                || !self.inspector
                || !matches!(self.editor.mode, Mode::Read | Mode::Annotate | Mode::Draw));
        let shortcut = ctx.input(|i| {
            i.viewport().close_requested()
                || i.modifiers.command
                    && [egui::Key::S, egui::Key::W, egui::Key::Q, egui::Key::Z]
                        .iter()
                        .any(|&key| i.key_pressed(key))
        });
        if interrupted || shortcut {
            self.finish_voice_recording();
            self.editor.voice.player = None;
            self.editor.voice.target = None;
        }
        if self.editor.voice.pending_permission {
            match voice_audio::permission(false) {
                3 => {
                    self.editor.voice.pending_permission = false;
                    self.start_voice_stream();
                }
                1 | 2 => {
                    self.editor.voice.pending_permission = false;
                    self.editor.voice.error = Some("Microphone access is disabled. Allow SkyPDF in System Settings → Privacy & Security → Microphone, then try Record again.".into());
                }
                _ => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
        }
        if let Some(recorder) = &self.editor.voice.recorder {
            let error = recorder.error();
            if recorder.full() || error.is_some() {
                self.finish_voice_recording();
                if let Some(error) = error {
                    self.editor.voice.error = Some(format!("Recording stopped: {error}"));
                }
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if let Some(player) = &self.editor.voice.player {
            if player.done() || player.error().is_some() {
                self.editor.voice.error = player.error();
                self.editor.voice.player = None;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    fn start_voice_stream(&mut self) {
        match voice_audio::Recorder::start() {
            Ok(recorder) => {
                self.editor.voice.recorder = Some(recorder);
                self.editor.voice.error = None;
            }
            Err(e) => {
                self.editor.voice.error = Some(format!(
                    "Could not record: {e}. Check the microphone and its privacy permission."
                ));
            }
        }
    }
    pub(in crate::desktop) fn voice_inspector(
        &mut self,
        ctx: &egui::Context,
        selected: Annotation,
    ) {
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
        let clip = selected.voice.clone().unwrap_or_default();
        let mut thread_action = None;
        let mut close = false;
        let mut delete = false;
        let mut record = false;
        let mut stop = false;
        let mut discard = false;
        let mut play = false;
        let mut export = false;
        let mut focus = None;
        let mut retry = false;
        let mut save_retained = false;
        let mut discard_retained = false;
        egui::SidePanel::right("annotation_inspector")
            .default_width(300.)
            .width_range(250.0..=440.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(theme::paper()).inner_margin(16.))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = vec2(8., 10.);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Voice note").size(18.).family(ui::semibold()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        close = ui
                            .add(ActionButton::new(Icon::Close, "Close voice note").compact())
                            .clicked();
                    });
                });
                ui.weak(format!(
                    "Page {} · {}",
                    selected.page + 1,
                    duration(clip.duration())
                ));
                ui.separator();
                let body_height = (ui.available_height() - 158.).max(80.);
                egui::ScrollArea::vertical()
                    .id_salt(("voice_thread", tab_id, &selected.id))
                    .max_height(body_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        comment_thread::author_header(
                            ui,
                            &selected.object.review.author,
                            selected.object.review.created,
                        );
                        ui.label("Title");
                        let note = self.editor.draft.as_mut().unwrap();
                        let text = note.kind.text_mut().unwrap();
                        let field = ui.add_enabled(
                            editable,
                            egui::TextEdit::singleline(text)
                                .desired_width(f32::INFINITY)
                                .char_limit(200)
                                .hint_text("Optional title"),
                        );
                        if field.has_focus() {
                            focus = Some(field.id);
                        }
                        ui.add_space(8.);
                        if self.editor.voice.pending_permission {
                            ui.spinner();
                            ui.label("Waiting for microphone permission…");
                            discard = ui.button("Cancel").clicked();
                        } else if let Some(recorder) = &self.editor.voice.recorder {
                            ui.colored_label(
                                theme::danger(),
                                format!(
                                    "{}  {} / 3:00",
                                    if recorder.paused() {
                                        "Paused"
                                    } else {
                                        "● Recording"
                                    },
                                    duration(recorder.duration())
                                ),
                            );
                            ui.horizontal(|ui| {
                                if ui
                                    .button(if recorder.paused() { "Resume" } else { "Pause" })
                                    .clicked()
                                {
                                    recorder.toggle_pause();
                                }
                                stop = ui
                                    .add(ActionButton::new(Icon::Check, "Stop & keep").primary())
                                    .clicked();
                            });
                            discard = ui.small_button("Discard recording").clicked();
                        } else {
                            if !clip.wav.is_empty() {
                                let elapsed = self
                                    .editor
                                    .voice
                                    .player
                                    .as_ref()
                                    .map_or(0., |p| p.seconds());
                                ui.add(
                                    egui::ProgressBar::new(elapsed / clip.duration().max(0.01))
                                        .text(format!(
                                            "{} / {}",
                                            duration(elapsed),
                                            duration(clip.duration())
                                        )),
                                );
                                ui.horizontal(|ui| {
                                    if let Some(player) = &self.editor.voice.player {
                                        if ui
                                            .button(if player.paused.load(Ordering::Relaxed) {
                                                "Resume playback"
                                            } else {
                                                "Pause playback"
                                            })
                                            .clicked()
                                        {
                                            player.toggle_pause();
                                        }
                                        if ui.button("Stop").clicked() {
                                            self.editor.voice.player = None;
                                        }
                                    } else {
                                        play = ui
                                            .add(
                                                ActionButton::new(Icon::Right, "Play recording")
                                                    .primary(),
                                            )
                                            .clicked();
                                    }
                                });
                                export = ui
                                    .add(ActionButton::new(Icon::Export, "Save audio…"))
                                    .clicked();
                            } else {
                                ui.weak("Record a comment for this spot on the page.");
                            }
                            record = ui
                                .add_enabled(
                                    editable,
                                    ActionButton::new(
                                        Icon::Microphone,
                                        if clip.wav.is_empty() {
                                            "Record"
                                        } else {
                                            "Record again"
                                        },
                                    ),
                                )
                                .clicked();
                            ui.weak(
                                "Up to 3 minutes. Recording starts only when you press Record.",
                            );
                        }
                        if let Some(error) = &self.editor.voice.error {
                            ui.colored_label(theme::danger(), error);
                        }
                        ui.separator();
                        if self.editor.voice.pending_clip.is_some() {
                            ui.horizontal_wrapped(|ui| {
                                retry = ui.button("Retry attaching").clicked();
                                save_retained = ui.button("Save retained audio…").clicked();
                                discard_retained = ui.button("Discard retained audio").clicked();
                            });
                        }
                        ui.weak(
                            "Audio stays on this computer. Save the PDF to include the recording.",
                        );
                        if editable {
                            ui.collapsing("Note color", |ui| {
                                color_palette(ui, &mut note.style.color)
                            });
                            delete = ui
                                .add_enabled(
                                    self.editor.voice.recorder.is_none()
                                        && !self.editor.voice.pending_permission,
                                    ActionButton::new(Icon::Trash, "Delete note"),
                                )
                                .clicked();
                        }
                        comment_thread::replies(ui, &selected);
                    });
                ui.separator();
                thread_action = self.editor.comment_thread.composer(
                    ui,
                    tab_id,
                    &selected,
                    editable
                        && self.editor.voice.recorder.is_none()
                        && !self.editor.voice.pending_permission,
                );
            });
        self.editor.text_editing |= focus.is_some() || self.editor.comment_thread.editing;
        let committed = self.commit_note_draft(
            &selected,
            focus,
            ctx.input(|i| i.pointer.any_down() || i.pointer.any_released()),
        );
        if committed && let Some(action) = thread_action {
            self.apply_comment_action(tab_id, &selected.id, action);
        }
        if discard {
            self.editor.voice.recorder = None;
            self.editor.voice.pending_permission = false;
        }
        if stop || close {
            self.finish_voice_recording();
        }
        if retry {
            self.finish_voice_recording();
        }
        if save_retained && let Some((_, _, clip)) = self.editor.voice.pending_clip.clone() {
            self.save_voice_audio(clip);
        }
        if discard_retained {
            self.editor.voice.pending_clip = None;
            self.editor.voice.error = None;
        }
        if record && self.finish_voice_recording() {
            self.editor.voice.target = Some((tab_id, selected.id.clone()));
            self.editor.voice.player = None;
            self.editor.voice.error = None;
            match voice_audio::permission(true) {
                3 => self.start_voice_stream(),
                0 => self.editor.voice.pending_permission = true,
                _ => self.editor.voice.error = Some("Microphone access is disabled. Allow SkyPDF in your system privacy settings, then try again.".into()),
            }
        }
        if play {
            self.editor.voice.target = Some((tab_id, selected.id.clone()));
            match voice_audio::Player::start(&clip) {
                Ok(player) => {
                    self.editor.voice.player = Some(player);
                    self.editor.voice.error = None;
                }
                Err(e) => self.editor.voice.error = Some(format!("Could not play: {e}")),
            }
        }
        if export {
            self.save_voice_audio(clip);
        }
        if delete {
            self.editor.voice.player = None;
            self.editor_delete();
        } else if close {
            self.editor.voice.player = None;
            selection::set(&mut self.tabs[self.active], &mut self.editor, vec![]);
        }
    }
    fn save_voice_audio(&mut self, clip: Clip) {
        if let Some(path) = menu::with_file_dialog(self._native_menu.as_ref(), || {
            rfd::FileDialog::new()
                .set_title("Save voice recording")
                .add_filter("WAV audio", &["wav"])
                .set_file_name("Voice note.wav")
                .save_file()
        }) {
            let result = viewer::save_copy(&path, clip.wav.as_slice());
            self.status = Some(match result {
                Ok(()) => ("Voice recording saved".into(), false),
                Err(e) => (format!("Could not save audio: {e}"), true),
            });
        }
    }
}

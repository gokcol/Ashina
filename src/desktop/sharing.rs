use super::*;
use mypdf_milestone0::sanitize::{Options, Report};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
};
enum Update {
    Progress(usize, usize),
    Done(Result<Report, String>),
}
#[derive(Default)]
pub(super) struct State {
    open: bool,
    options: Options,
    bytes: Option<Vec<u8>>,
    source: Option<PathBuf>,
    path: Option<PathBuf>,
    receiver: Option<mpsc::Receiver<Update>>,
    cancel: Option<Arc<AtomicBool>>,
    progress: (usize, usize),
    report: Option<Report>,
    error: Option<String>,
}
impl State {
    pub(super) fn is_open(&self) -> bool {
        self.open
    }
}
fn section(ui: &mut egui::Ui, contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::surface())
        .stroke(egui::Stroke::new(1., theme::line()))
        .corner_radius(12)
        .inner_margin(14.)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            contents(ui);
        });
}
fn mode_card(ui: &mut egui::Ui, title: &str, subtitle: &str, icon: Icon, selected: bool) -> bool {
    let response = egui::Frame::new()
        .fill(if selected {
            theme::accent_soft()
        } else {
            theme::surface()
        })
        .stroke(egui::Stroke::new(
            if selected { 1.5 } else { 1. },
            if selected {
                theme::accent_line()
            } else {
                theme::line()
            },
        ))
        .corner_radius(12)
        .inner_margin(12.)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(64.);
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(22., 22.), egui::Sense::hover());
                ui::draw_icon(
                    ui.painter(),
                    rect,
                    icon,
                    if selected {
                        theme::accent()
                    } else {
                        theme::muted()
                    },
                );
                ui.label(RichText::new(title).strong());
            });
            ui.label(RichText::new(subtitle).small().color(theme::muted()));
        })
        .response
        .interact(egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::RadioButton,
            ui.is_enabled(),
            selected,
            title,
        )
    });
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
}
fn removal_row(ui: &mut egui::Ui, checked: &mut bool, title: &str, detail: &str, forced: bool) {
    ui.horizontal(|ui| {
        let toggle = ui
            .add_enabled(!forced, egui::Checkbox::without_text(checked))
            .on_hover_text(title);
        toggle.widget_info(|| {
            egui::WidgetInfo::selected(egui::WidgetType::Checkbox, !forced, *checked, title)
        });
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 3.;
            ui.label(RichText::new(title).strong());
            ui.label(RichText::new(detail).small().color(theme::muted()));
        });
    });
}
fn sharing_options(ui: &mut egui::Ui, options: &mut Options) {
    ui.label(RichText::new("Choose how to prepare your copy").strong());
    ui.columns(2, |columns| {
        if mode_card(
            &mut columns[0],
            "Clean up",
            "Keep searchable text and interactive pages.",
            Icon::Read,
            !options.hidden,
        ) {
            options.hidden = false;
        }
        if mode_card(
            &mut columns[1],
            "Flatten",
            "Keep only the visible page appearance.",
            Icon::Shield,
            options.hidden,
        ) {
            options.hidden = true;
        }
    });
    ui.add_space(4.);
    section(ui, |ui| {
        ui.label(RichText::new("Remove from this copy").strong());
        ui.add_space(4.);
        removal_row(
            ui,
            &mut options.comments,
            "Comments & drawings",
            "Notes, text annotations, stamps and drawn objects",
            false,
        );
        ui.separator();
        if options.hidden {
            options.metadata = true;
            options.attachments = true;
        }
        removal_row(
            ui,
            &mut options.metadata,
            "Document metadata",
            "Author, title, keywords and embedded metadata",
            options.hidden,
        );
        ui.separator();
        removal_row(
            ui,
            &mut options.attachments,
            "Attachments & recordings",
            "Embedded files and voice-note audio",
            options.hidden,
        );
        if options.hidden {
            ui.add_space(3.);
            ui.weak("Flatten always removes metadata, attachments and hidden content.");
        }
    });
    if options.hidden {
        ui.horizontal(|ui| {
            ui.label("Image quality");
            egui::ComboBox::from_id_salt("sharing_dpi")
                .selected_text(format!("{} dpi", options.dpi))
                .show_ui(ui, |ui| {
                    for (dpi, label) in [
                        (100., "Compact · 100 dpi"),
                        (150., "Standard · 150 dpi"),
                        (200., "Balanced · 200 dpi"),
                        (300., "High quality · 300 dpi"),
                    ] {
                        ui.selectable_value(&mut options.dpi, dpi, label);
                    }
                });
        });
    }
    egui::Frame::new().fill(theme::accent_soft()).corner_radius(10).inner_margin(12.).show(ui,|ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(if options.hidden {"What changes when flattened"} else {"What cleanup preserves"}).strong());
        ui.label(if options.hidden {"Pages become images. Search, text editing, form fields and links stop working. Digital signatures are not preserved."} else {"Text, forms and links stay interactive. Hidden text, cropped content and layers may remain; choose Flatten to remove them."});
    });
    ui.label(RichText::new("Visible sensitive information stays on the page. Redact it first, then review the saved copy before sharing.").small().color(theme::muted()));
}
impl Viewer {
    pub(super) fn request_sharing(&mut self) {
        if self.sharing.receiver.is_some() {
            self.sharing.open = true;
            return;
        }
        if self.content.busy || self.organizer.busy || self.signature_busy {
            self.status = Some((
                "Wait for the current operation before preparing a sharing copy.".into(),
                true,
            ));
            return;
        }
        if !self.apply_pending_annotation() {
            return;
        }
        if self.content.pending_text().is_some() {
            self.status = Some((
                "Finish the text edit before preparing a sharing copy.".into(),
                true,
            ));
            return;
        }
        let Some(tab) = self.tabs.get(self.active) else {
            return;
        };
        if self.pending_form_edits(tab.id) || self.table.has_editor(tab.id) {
            self.status = Some((
                "Finish the form or table edit before preparing a sharing copy.".into(),
                true,
            ));
            return;
        }
        let Some(session) = &tab.session else {
            return;
        };
        match session.snapshot_bytes() {
            Ok(bytes) => {
                self.sharing = State {
                    open: true,
                    bytes: Some(bytes),
                    source: Some(tab.path.clone()),
                    ..Default::default()
                }
            }
            Err(e) => self.status = Some((format!("Could not prepare document: {e}"), true)),
        }
    }
    pub(super) fn sharing_dialog(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.sharing.receiver {
            let mut done = false;
            while let Ok(update) = rx.try_recv() {
                match update {
                    Update::Progress(page, total) => self.sharing.progress = (page, total),
                    Update::Done(result) => {
                        done = true;
                        match result {
                            Ok(report) => self.sharing.report = Some(report),
                            Err(e) => self.sharing.error = Some(e),
                        }
                    }
                }
            }
            if done {
                self.sharing.receiver = None;
                self.sharing.cancel = None;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if !self.sharing.open {
            return;
        }
        let busy = self.sharing.receiver.is_some();
        let mut start = false;
        let mut close = false;
        let mut open = false;
        let modal = egui::Modal::new(egui::Id::new("prepare_sharing")).frame(ui::modal_frame()).show(ctx, |ui| {
            ui.set_width(560_f32.min((ctx.content_rect().width()-64.).max(280.)));
            ui.spacing_mut().item_spacing=vec2(10.,12.);
            ui.horizontal(|ui| {
                let (rect,_)=ui.allocate_exact_size(vec2(42.,42.),egui::Sense::hover());
                ui.painter().rect_filled(rect,12,theme::accent_soft());ui::draw_icon(ui.painter(),rect.shrink(9.),Icon::Shield,theme::accent());
                ui.vertical(|ui| {ui.label(RichText::new("Prepare for sharing").size(22.).family(ui::semibold()));ui.label(RichText::new("A cleaner copy, ready to send").color(theme::muted()));});
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {close=ui.add(ActionButton::new(Icon::Close,"Close sharing preparation").compact()).clicked();});
            });
            ui.separator();
            egui::ScrollArea::vertical().max_height((ctx.content_rect().height()-210.).max(150.)).show(ui,|ui| {
                ui.set_width(ui.available_width());
                if let Some(report)=&self.sharing.report {
                    section(ui,|ui| {
                        ui.label(RichText::new("Your sharing copy is ready").size(18.).color(theme::accent()).strong());
                        ui.label(format!("{} pages prepared",report.pages));
                        if let Some(path)=&self.sharing.path {ui.label(path.file_name().unwrap_or_default().to_string_lossy());}
                        if report.flattened {ui.label("Only visible page images were copied. Source metadata, audio, attachments, hidden content and interactive objects were excluded.");}
                        for warning in &report.warnings {ui.label(warning);}
                    });
                } else if busy {
                    section(ui,|ui| {
                        ui.label(RichText::new("Preparing your sharing copy…").strong());
                        let (page,total)=self.sharing.progress;
                        ui.add(egui::ProgressBar::new(page as f32/total.max(1) as f32).animate(true).text(format!("Page {} of {}",(page+1).min(total),total)));
                        ui.weak("Your original document stays unchanged.");
                    });
                } else {sharing_options(ui,&mut self.sharing.options);}
                if let Some(error)=&self.sharing.error {
                    egui::Frame::new().fill(theme::danger_soft()).corner_radius(10).inner_margin(12.).show(ui,|ui| {ui.colored_label(theme::danger(),error);});
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(RichText::new("Original stays unchanged").small().color(theme::muted()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {
                    if self.sharing.report.is_some() {open=ui.add(ActionButton::new(Icon::Open,"Open sharing copy").primary()).clicked();}
                    else if busy {
                        if ui.button("Cancel preparation").clicked() && let Some(cancel)=&self.sharing.cancel {cancel.store(true,Ordering::Relaxed);}
                    } else {start=ui.add(ActionButton::new(Icon::Export,"Save sharing copy…").primary()).clicked();}
                });
            });
        });
        if close || modal.should_close() {
            if let Some(cancel) = &self.sharing.cancel {
                cancel.store(true, Ordering::Relaxed);
            }
            self.sharing.open = false;
        }
        if open && let Some(path) = self.sharing.path.clone() {
            self.sharing.open = false;
            self.open(path, ctx);
        }
        if start
            && let Some(path) = menu::with_file_dialog(self._native_menu.as_ref(), || {
                rfd::FileDialog::new()
                    .set_title("Save sharing copy")
                    .set_file_name("Sharing copy.pdf")
                    .add_filter("PDF", &["pdf"])
                    .save_file()
            })
        {
            if self.sharing.source.as_ref().is_some_and(|source| {
                source == &path
                    || source
                        .canonicalize()
                        .ok()
                        .zip(path.canonicalize().ok())
                        .is_some_and(|(a, b)| a == b)
            }) {
                self.sharing.error =
                    Some("Choose a different filename to preserve the original.".into());
                return;
            }
            let Some(bytes) = self.sharing.bytes.clone() else {
                return;
            };
            let options = self.sharing.options;
            let cancel = Arc::new(AtomicBool::new(false));
            self.sharing.cancel = Some(cancel.clone());
            let (tx, rx) = mpsc::channel();
            self.sharing.receiver = Some(rx);
            self.sharing.error = None;
            self.sharing.path = Some(path.clone());
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mypdf_milestone0::sanitize::prepare(&bytes, options, |p, t| {
                        let _ = tx.send(Update::Progress(p, t));
                        ctx.request_repaint();
                        !cancel.load(Ordering::Relaxed)
                    })
                    .and_then(|(bytes, report)| {
                        anyhow::ensure!(!cancel.load(Ordering::Relaxed), "Cancelled");
                        viewer::save_copy(&path, &bytes)?;
                        Ok(report)
                    })
                    .map_err(|e| format!("Could not create sharing copy: {e}"))
                }))
                .unwrap_or_else(|_| {
                    Err("Could not process this PDF. The original is unchanged.".into())
                });
                let _ = tx.send(Update::Done(result));
                ctx.request_repaint();
            });
        }
    }
}

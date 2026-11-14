//! Compact, opt-in catalog management; gallery geometry stays unchanged.
use super::*;
use mypdf_milestone0::{annotations::Annotation, assets::Asset};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Action {
    Manage,
    Svg,
    Prompt,
    Import,
    Signature,
}

/// One toolbar slot, with hover and click access for mouse, keyboard and touch.
pub(super) fn actions(ui: &mut egui::Ui) -> Option<Action> {
    let button = ui.add(ActionButton::new(Icon::ManageCollection, "Collection actions").compact());
    let mut action = None;
    egui::Popup::menu(&button)
        .open_memory(ui::menu_hover_open(&button))
        .show(|ui| {
            let add = ui.add(ActionButton::new(Icon::Plus, "Add artwork  ›"));
            egui::containers::menu::SubMenu::new().show(ui, &add, |ui| {
                for (icon, label, value) in [
                    (Icon::Text, "SVG code…", Action::Svg),
                    (Icon::Sparkle, "Create with prompt…", Action::Prompt),
                    (Icon::Import, "Import image / SVG…", Action::Import),
                    (Icon::Sign, "Signature…", Action::Signature),
                ] {
                    if ui.add(ActionButton::new(icon, label)).clicked() {
                        action = Some(value);
                        ui.close_kind(egui::UiKind::Menu);
                    }
                }
            });
            if ui
                .add(ActionButton::new(
                    Icon::ManageCollection,
                    "Manage collection…",
                ))
                .clicked()
            {
                action = Some(Action::Manage);
                ui.close();
            }
        });
    if action.is_some() {
        ui::menu_hover_chosen(&button);
    }
    action
}

const FILE: &str = "asset-collections.json";
const CATEGORIES: [&str; 4] = ["Arrows", "Symbols", "Stamps", "Saved"];
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
struct Layout {
    order: HashMap<String, Vec<String>>,
    visible: HashMap<String, bool>,
}
impl Layout {
    fn visible(&self, item: &Item) -> bool {
        self.visible
            .get(&item.asset.id)
            .copied()
            .unwrap_or(item.default_visible)
    }
    fn rank(&self, category: &str, id: &str) -> usize {
        self.order
            .get(category)
            .and_then(|ids| ids.iter().position(|v| v == id))
            .unwrap_or(usize::MAX)
    }
    fn move_to(&mut self, category: &str, ids: &[String], source: &str, target: usize) {
        let mut ids = ids.to_vec();
        if let Some(from) = ids.iter().position(|id| id == source) {
            let id = ids.remove(from);
            ids.insert(target.min(ids.len()), id);
            self.order.insert(category.into(), ids);
        }
    }
    fn reset(&mut self, category: &str, items: &[Item]) {
        self.order.remove(category);
        for item in items.iter().filter(|item| item.in_category(category)) {
            self.visible.remove(&item.asset.id);
        }
    }
}
#[derive(Clone)]
pub(super) struct Item {
    pub asset: Asset,
    builtin_category: Option<&'static str>,
    default_visible: bool,
}
impl Item {
    fn in_category(&self, category: &str) -> bool {
        match category {
            "Stamps" => self.asset.stamp,
            "Saved" => self.builtin_category.is_none(),
            other => self.builtin_category == Some(other),
        }
    }
}
pub(super) fn catalog(editor: &editing::EditorState) -> Vec<Item> {
    let mut items: Vec<_> = editor
        .builtins
        .iter()
        .map(|b| Item {
            asset: b.asset.clone(),
            builtin_category: Some(b.category),
            default_visible: b.default_visible,
        })
        .collect();
    if let Some(library) = &editor.library {
        for asset in &library.entries {
            if let Some(item) = items.iter_mut().find(|item| item.asset.id == asset.id) {
                item.asset = asset.clone();
            } else {
                items.push(Item {
                    asset: asset.clone(),
                    builtin_category: None,
                    default_visible: true,
                });
            }
        }
    }
    items
}
struct Draft {
    layout: Layout,
    category: String,
    query: String,
    filter: usize,
}
pub(super) struct State {
    layout: Layout,
    draft: Option<Draft>,
    root: PathBuf,
    error: Option<String>,
}
impl State {
    pub fn is_open(&self) -> bool {
        self.draft.is_some()
    }
    pub fn load(root: PathBuf) -> Self {
        let (layout, error) = match preferences::load(&root, FILE) {
            Ok(layout) => (layout, None),
            Err(e) => (
                Layout::default(),
                Some(format!("Could not read collection settings: {e}")),
            ),
        };
        Self {
            layout,
            draft: None,
            root,
            error,
        }
    }
    pub fn open(&mut self, category: &str) {
        self.draft = Some(Draft {
            layout: self.layout.clone(),
            category: if CATEGORIES.contains(&category) {
                category
            } else {
                "Stamps"
            }
            .into(),
            query: String::new(),
            filter: 0,
        });
    }
    pub fn visible(&self, id: &str, default: bool) -> bool {
        self.layout.visible.get(id).copied().unwrap_or(default)
    }
    pub fn rank(&self, category: &str, id: &str) -> usize {
        self.layout.rank(category, id)
    }
}
#[derive(Clone)]
struct DragItem {
    category: String,
    id: String,
}
impl Viewer {
    pub(super) fn collection_manager(&mut self, ctx: &egui::Context) {
        let Some(mut draft) = self.editor.collections.draft.take() else {
            return;
        };
        let items = catalog(&self.editor);
        let mut close = false;
        let mut save = false;
        let width = (ctx.content_rect().width() - 72.).clamp(280., 560.);
        let max_height = (ctx.content_rect().height() - 320.).clamp(80., 460.);
        let modal = egui::Modal::new(egui::Id::new("collection_manager"))
            .frame(ui::modal_frame())
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    ui.heading("Manage collection");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                ActionButton::new(Icon::Close, "Close collection manager")
                                    .compact(),
                            )
                            .clicked()
                        {
                            close = true;
                        }
                    });
                });
                ui.label(
                    RichText::new("Choose what appears. Drag rows or use the arrows to reorder.")
                        .small()
                        .color(theme::muted()),
                );
                ui.add_space(8.);
                let selected = CATEGORIES
                    .iter()
                    .position(|c| *c == draft.category)
                    .unwrap_or(2);
                if let Some(index) = ui::segmented(ui, &CATEGORIES, selected) {
                    draft.category = CATEGORIES[index].into();
                    draft.query.clear();
                }
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut draft.query)
                            .hint_text("Find artwork…")
                            .desired_width((ui.available_width() - 130.).max(120.)),
                    );
                    egui::ComboBox::from_id_salt("collection_filter")
                        .selected_text(["All items", "Visible", "Hidden"][draft.filter])
                        .width(110.)
                        .show_ui(ui, |ui| {
                            for (i, label) in ["All items", "Visible", "Hidden"].iter().enumerate()
                            {
                                ui.selectable_value(&mut draft.filter, i, *label);
                            }
                        });
                });
                let mut entries: Vec<_> = items
                    .iter()
                    .filter(|item| item.in_category(&draft.category))
                    .collect();
                entries.sort_by_key(|item| draft.layout.rank(&draft.category, &item.asset.id));
                let visible = entries
                    .iter()
                    .filter(|item| draft.layout.visible(item))
                    .count();
                ui.horizontal(|ui| {
                    ui.weak(format!(
                        "{visible} visible · {} hidden",
                        entries.len() - visible
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Restore defaults").clicked() {
                            draft.layout.reset(&draft.category, &items);
                        }
                    });
                });
                let ids: Vec<_> = entries.iter().map(|item| item.asset.id.clone()).collect();
                let reorder = draft.query.is_empty() && draft.filter == 0;
                let query = draft.query.to_lowercase();
                let mut movement = None;
                egui::ScrollArea::vertical()
                    .id_salt("collection_rows")
                    .max_height(max_height)
                    .show(ui, |ui| {
                        let mut shown = 0;
                        for (index, item) in entries.iter().enumerate() {
                            let mut visible = draft.layout.visible(item);
                            if !item.asset.name.to_lowercase().contains(&query)
                                || draft.filter == 1 && !visible
                                || draft.filter == 2 && visible
                            {
                                continue;
                            }
                            shown += 1;
                            ui.push_id(&item.asset.id, |ui| {
                                let row = egui::Frame::new()
                                    .fill(if index % 2 == 0 {
                                        theme::canvas()
                                    } else {
                                        theme::surface()
                                    })
                                    .corner_radius(8)
                                    .inner_margin(5)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let check = ui.checkbox(&mut visible, "");
                                            check.widget_info(|| {
                                                egui::WidgetInfo::labeled(
                                                    egui::WidgetType::Checkbox,
                                                    true,
                                                    format!("Show {}", item.asset.name),
                                                )
                                            });
                                            check.clone().on_hover_text("Show in the right panel");
                                            if check.changed() {
                                                draft
                                                    .layout
                                                    .visible
                                                    .insert(item.asset.id.clone(), visible);
                                            }
                                            let (r, _) = ui.allocate_exact_size(
                                                vec2(70., 34.),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().rect_filled(r, 4., Color32::WHITE);
                                            if ui.is_rect_visible(r) {
                                                let base = vec2(200., 200. / item.asset.aspect);
                                                let scale = ((r.width() - 8.) / base.x)
                                                    .min((r.height() - 6.) / base.y);
                                                let page = Rect::from_center_size(
                                                    r.center(),
                                                    base * scale,
                                                );
                                                let mark = Annotation::new(
                                                    0,
                                                    [0., 0., base.x, base.y],
                                                    item.asset.kind.clone(),
                                                    item.asset.style.clone(),
                                                );
                                                editing::paint_annotation(
                                                    &mut self.editor,
                                                    ctx,
                                                    &ui.painter().with_clip_rect(r),
                                                    page,
                                                    scale,
                                                    &mark,
                                                );
                                            }
                                            let label_width = (width - 230.).max(50.);
                                            ui.add_enabled_ui(reorder, |ui| {
                                                ui.dnd_drag_source(
                                                    egui::Id::new((
                                                        "collection_drag",
                                                        &draft.category,
                                                        &item.asset.id,
                                                    )),
                                                    DragItem {
                                                        category: draft.category.clone(),
                                                        id: item.asset.id.clone(),
                                                    },
                                                    |ui| {
                                                        ui.add_sized(
                                                            [label_width, 30.],
                                                            egui::Label::new(&item.asset.name)
                                                                .truncate(),
                                                        );
                                                    },
                                                );
                                            });
                                            for (symbol, label, target, enabled) in [
                                                ("↑", "up", index.saturating_sub(1), index > 0),
                                                ("↓", "down", index + 1, index + 1 < entries.len()),
                                            ] {
                                                let response = ui.add_enabled(
                                                    reorder && enabled,
                                                    egui::Button::new(symbol)
                                                        .min_size(vec2(25., 28.)),
                                                );
                                                response.widget_info(|| {
                                                    egui::WidgetInfo::labeled(
                                                        egui::WidgetType::Button,
                                                        reorder && enabled,
                                                        format!("Move {} {label}", item.asset.name),
                                                    )
                                                });
                                                if response.clicked() {
                                                    movement =
                                                        Some((item.asset.id.clone(), target));
                                                }
                                            }
                                        });
                                    });
                                if reorder
                                    && let Some(payload) =
                                        row.response.dnd_hover_payload::<DragItem>()
                                    && payload.category == draft.category
                                {
                                    ui.painter().rect_stroke(
                                        row.response.rect,
                                        8,
                                        Stroke::new(1.5, theme::accent()),
                                        egui::StrokeKind::Inside,
                                    );
                                }
                                if reorder
                                    && let Some(payload) =
                                        row.response.dnd_release_payload::<DragItem>()
                                    && payload.category == draft.category
                                {
                                    movement = Some((payload.id.clone(), index));
                                }
                            });
                            ui.add_space(3.);
                        }
                        if shown == 0 {
                            ui.weak("No items in this view.");
                        }
                    });
                if let Some((id, index)) = movement {
                    draft.layout.move_to(&draft.category, &ids, &id, index);
                }
                if !reorder {
                    ui.weak("Clear the search and select All items to reorder.");
                }
                if let Some(error) = &self.editor.collections.error {
                    ui.colored_label(theme::danger(), error);
                }
                ui.separator();
                ui.weak("Changes affect the library, not placed artwork.");
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        save = ui
                            .add(ActionButton::new(Icon::Check, "Save changes").primary())
                            .clicked();
                        if ui.button("Cancel").clicked() {
                            close = true;
                        }
                    });
                });
            });
        if modal.should_close() {
            close = true;
        }
        if save {
            match preferences::save(&self.editor.collections.root, FILE, &draft.layout) {
                Ok(()) => {
                    self.editor.collections.layout = draft.layout.clone();
                    self.editor.collections.error = None;
                    close = true;
                }
                Err(e) => {
                    self.editor.collections.error =
                        Some(format!("Could not save collection settings: {e}"));
                    close = false;
                }
            }
        }
        if !close {
            self.editor.collections.draft = Some(draft);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Pos2;

    fn frame(
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        mut draw: impl FnMut(&mut egui::Ui),
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(900., 820.))),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| draw(ui));
            },
        )
    }
    fn position(output: &egui::FullOutput, label: &str) -> Pos2 {
        let n = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some(label))
            .unwrap_or_else(|| panic!("Missing {label}"));
        let r = n.1.bounds().unwrap();
        pos2(((r.x0 + r.x1) / 2.) as f32, ((r.y0 + r.y1) / 2.) as f32)
    }
    fn pointer(pos: Pos2, pressed: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }
    #[test]
    fn collection_hover_menu_exposes_two_actions_and_add_submenu() {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut output = frame(&ctx, vec![], |ui| {
            actions(ui);
        });
        for _ in 0..3 {
            output = frame(&ctx, vec![], |ui| {
                actions(ui);
            });
        }
        let button = position(&output, "Collection actions");
        for _ in 0..3 {
            output = frame(&ctx, vec![egui::Event::PointerMoved(button)], |ui| {
                actions(ui);
            });
        }
        let add = position(&output, "Add artwork  ›");
        position(&output, "Manage collection…");
        assert!(
            !output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .any(|(_, n)| n.label() == Some("SVG code…"))
        );
        for _ in 0..3 {
            output = frame(&ctx, vec![egui::Event::PointerMoved(add)], |ui| {
                actions(ui);
            });
        }
        position(&output, "SVG code…");
        position(&output, "Import image / SVG…");
        let manage = position(&output, "Manage collection…");
        let mut chosen = None;
        for pressed in [true, false] {
            frame(&ctx, pointer(manage, pressed), |ui| {
                chosen = chosen.or(actions(ui));
            });
        }
        assert_eq!(chosen, Some(Action::Manage));
    }
    #[test]
    fn collection_defaults_reorder_visibility_cancel_save_and_reload() {
        let root = tempfile::tempdir().unwrap();
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let mut app = Viewer::new(&eframe::CreationContext::_new_kittest(ctx.clone()), vec![]);
        app.editor.collections = State::load(root.path().into());
        let items = catalog(&app.editor);
        for category in ["Arrows", "Symbols", "Stamps"] {
            assert_eq!(items.iter().filter(|i| i.in_category(category) && !app.editor.collections.layout.visible(i)).count(), 15);
        }
        let ids: Vec<_> = items
            .iter()
            .filter(|i| i.in_category("Stamps"))
            .map(|i| i.asset.id.clone())
            .collect();
        let hidden = ids.last().unwrap().clone();
        let original = app.editor.collections.layout.clone();
        app.editor.collections.open("Stamps");
        let draft = app.editor.collections.draft.as_mut().unwrap();
        draft.layout.visible.insert(hidden.clone(), true);
        draft.layout.move_to("Stamps", &ids, &hidden, 0);
        let mut output = frame(&ctx, vec![], |ui| app.collection_manager(ui.ctx()));
        for _ in 0..3 {
            output = frame(&ctx, vec![], |ui| app.collection_manager(ui.ctx()));
        }
        let cancel = position(&output, "Cancel");
        for pressed in [true, false] {
            frame(&ctx, pointer(cancel, pressed), |ui| {
                app.collection_manager(ui.ctx())
            });
        }
        assert!(app.editor.collections.draft.is_none());
        assert_eq!(app.editor.collections.layout, original);
        assert!(!root.path().join(FILE).exists());
        app.editor.collections.open("Stamps");
        let draft = app.editor.collections.draft.as_mut().unwrap();
        draft.layout.visible.insert(hidden.clone(), true);
        draft.layout.move_to("Stamps", &ids, &hidden, 0);
        for _ in 0..3 {
            output = frame(&ctx, vec![], |ui| app.collection_manager(ui.ctx()));
        }
        let save = position(&output, "Save changes");
        for pressed in [true, false] {
            frame(&ctx, pointer(save, pressed), |ui| {
                app.collection_manager(ui.ctx())
            });
        }
        assert!(
            app.editor.collections.draft.is_none(),
            "save={save:?}, error={:?}",
            app.editor.collections.error
        );
        let mut reloaded = State::load(root.path().into());
        assert_eq!(reloaded.rank("Stamps", &hidden), 0);
        assert!(reloaded.visible(&hidden, false));
        assert_eq!(reloaded.rank("Symbols", &hidden), usize::MAX);
        reloaded.layout.reset("Stamps", &items);
        assert!(!reloaded.visible(&hidden, false));
        assert_eq!(reloaded.rank("Stamps", &hidden), usize::MAX);
    }
}

use eframe::egui;
use std::collections::HashMap;

pub struct GUI {}

impl GUI {
    pub fn new() -> Self {
        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        };

        eframe::run_native(
            "Seamless",
            options,
            Box::new(|cc| Box::<SeamlessUI>::default()),
        )
        .unwrap();

        GUI {}
    }
}

struct SeamlessUI {
    display_items: HashMap<egui::Vec2, String>,
}

impl Default for SeamlessUI {
    fn default() -> Self {
        Self {
            display_items: HashMap::new(),
        }
    }
}

impl eframe::App for SeamlessUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");

            let id_source = "my_drag_and_drop_demo";
            let mut source_col_row = None;
            let mut drop_col = None;
            ui.columns(self.columns.len(), |uis| {
                for (col_idx, column) in self.columns.clone().into_iter().enumerate() {
                    let ui = &mut uis[col_idx];
                    let can_accept_what_is_being_dragged = true; // We accept anything being dragged (for now) ¯\_(ツ)_/¯
                    let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {
                        ui.set_min_size(vec2(64.0, 100.0));
                        for (row_idx, item) in column.iter().enumerate() {
                            let item_id = Id::new(id_source).with(col_idx).with(row_idx);
                            drag_source(ui, item_id, |ui| {
                                let response = ui.add(Label::new(item).sense(Sense::click()));
                                response.context_menu(|ui| {
                                    if ui.button("Remove").clicked() {
                                        self.columns[col_idx].remove(row_idx);
                                        ui.close_menu();
                                    }
                                });
                            });

                            if ui.memory(|mem| mem.is_being_dragged(item_id)) {
                                source_col_row = Some((col_idx, row_idx));
                            }
                        }
                    })
                    .response;

                    let response = response.context_menu(|ui| {
                        if ui.button("New Item").clicked() {
                            self.columns[col_idx].push("New Item".to_owned());
                            ui.close_menu();
                        }
                    });

                    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());
                    if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
                        drop_col = Some(col_idx);
                    }
                }
            });

            if let Some((source_col, source_row)) = source_col_row {
                if let Some(drop_col) = drop_col {
                    if ui.input(|i| i.pointer.any_released()) {
                        // do the drop:
                        let item = self.columns[source_col].remove(source_row);
                        self.columns[drop_col].push(item);
                    }
                }
            }
        });
    }
}

pub fn drop_target<R>(
    ui: &mut egui::Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let is_being_dragged = ui.memory(|mem| mem.is_anything_being_dragged());

    let margin = egui::Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let ret = body(&mut content_ui);
    let outer_rect =
        egui::Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), egui::Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        fill = ui.visuals().gray_out(fill);
        stroke.color = ui.visuals().gray_out(stroke.color);
    }

    ui.painter().set(
        where_to_put_background,
        egui::epaint::RectShape::new(rect, style.rounding, fill, stroke),
    );

    egui::InnerResponse::new(ret, response)
}

pub fn drag_source(ui: &mut egui::Ui, id: egui::Id, body: impl FnOnce(&mut egui::Ui)) {
    let is_being_dragged = ui.memory(|mem| mem.is_being_dragged(id));

    if !is_being_dragged {
        let response = ui.scope(body).response;

        // Check for drags:
        let response = ui.interact(response.rect, id, egui::Sense::drag());
        if response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
    } else {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);

        // Paint the body to a new layer:
        let layer_id = egui::LayerId::new(egui::Order::Tooltip, id);
        let response = ui.with_layer_id(layer_id, body).response;

        // Now we move the visuals of the body to where the mouse is.
        // Normally you need to decide a location for a widget first,
        // because otherwise that widget cannot interact with the mouse.
        // However, a dragged component cannot be interacted with anyway
        // (anything with `Order::Tooltip` always gets an empty [`Response`])
        // So this is fine!

        if let Some(pointer_pos) = ui.ctx().pointer_interact_pos() {
            let delta = pointer_pos - response.rect.center();
            ui.ctx().translate_layer(layer_id, delta);
        }
    }
}

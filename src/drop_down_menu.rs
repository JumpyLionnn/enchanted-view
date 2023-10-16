use egui::NumExt;

pub struct DropDownMenu {
    width: Option<f32>,
    menu_width: Option<f32>,
    text: egui::WidgetText,
    id: egui::Id
}

impl DropDownMenu {
    pub fn new(text: impl Into<egui::WidgetText>, id: impl std::hash::Hash) -> Self {
        Self {
            width: None,
            menu_width: None,
            text: text.into(),
            id: egui::Id::new(id)
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn menu_width(mut self, width: f32) -> Self {
        self.menu_width = Some(width);
        self
    }

    pub fn ui<'a, R>(self, ui: &mut egui::Ui, menu_contents: impl FnOnce(&mut egui::Ui) -> R + 'a) -> egui::InnerResponse<Option<R>> {
        let popup_id = self.id.with("popup");

        let is_popup_open = ui.memory(|m| m.is_popup_open(popup_id));

        let popup_height = 200.0;

        let above_or_below =
            if ui.next_widget_position().y + ui.spacing().interact_size.y + popup_height
                < ui.ctx().screen_rect().bottom()
            {
                egui::AboveOrBelow::Below
            } else {
                egui::AboveOrBelow::Above
            };

        let margin = ui.spacing().button_padding;
        let mut button_response = button_frame(ui, self.id, is_popup_open, egui::Sense::click(), |ui| {
            // We don't want to change width when user selects something new
            let full_minimum_width = {
                // Occupy at least the minimum width assigned to ComboBox.
                let width = self.width.unwrap_or_else(|| ui.spacing().combo_width);
                width - 2.0 * margin.x
            };

            let galley =
                self.text.into_galley(ui, Some(false), f32::INFINITY, egui::TextStyle::Button);

            // The width necessary to contain the whole widget with the currently selected value's text.
            let width = galley.size().x;

            // Case : wrap_enabled : occupy all the available width.
            // Case : !wrap_enabled : occupy at least the minimum width assigned to Slider and ComboBox,
            // increase if the currently selected value needs additional horizontal space to fully display its text (up to wrap_width (f32::INFINITY)).
            let width = width.at_least(full_minimum_width);
            let height = galley.size().y;

            let (_, rect) = ui.allocate_space(egui::Vec2::new(width, height));
            let button_rect = ui.min_rect().expand2(ui.spacing().button_padding);
            let response = ui.interact(button_rect, self.id, egui::Sense::click());

            if ui.is_rect_visible(rect) {
                let visuals = if is_popup_open {
                    &ui.visuals().widgets.open
                } else {
                    ui.style().interact(&response)
                };

                let text_rect = egui::Align2::CENTER_CENTER.align_size_within_rect(galley.size(), rect);
                galley.paint_with_visuals(ui.painter(), text_rect.min, visuals);
            }
        });

        if button_response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        if let Some(menu_width) = self.menu_width {
            // setting the size and position of the popup
            let button_rect = button_response.rect;
            button_response.rect.set_width(menu_width);
            button_response.rect.set_center(button_rect.center());
        }
        let inner = egui::popup::popup_above_or_below_widget(
            ui,
            popup_id,
            &button_response,
            above_or_below,
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(ui.spacing().combo_height)
                    .show(ui, menu_contents)
                    .inner
            },
        );

        egui::InnerResponse {
            inner,
            response: button_response,
        }
    }
}

fn button_frame(
    ui: &mut egui::Ui,
    id: egui::Id,
    is_popup_open: bool,
    sense: egui::Sense,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);

    let margin = ui.spacing().button_padding;
    let interact_size = ui.spacing().interact_size;

    let mut outer_rect = ui.available_rect_before_wrap();
    outer_rect.set_height(outer_rect.height().at_least(interact_size.y));

    let inner_rect = outer_rect.shrink2(margin);
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    add_contents(&mut content_ui);

    let mut outer_rect = content_ui.min_rect().expand2(margin);
    outer_rect.set_height(outer_rect.height().at_least(interact_size.y));

    let response = ui.interact(outer_rect, id, sense);

    if ui.is_rect_visible(outer_rect) {
        let visuals = if is_popup_open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };

        ui.painter().set(
            where_to_put_background,
            egui::epaint::RectShape::new(
                outer_rect.expand(visuals.expansion),
                visuals.rounding,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
            ),
        );
    }

    ui.advance_cursor_after_rect(outer_rect);

    response
}
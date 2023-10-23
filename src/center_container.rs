

pub struct CenterContainer {
    size: egui::Vec2,
    inner_layout: egui::Layout
}

impl CenterContainer {
    pub fn new(size: egui::Vec2) -> Self {
        Self { size, inner_layout: egui::Layout::default() }
    }

    pub fn inner_layout(mut self, layout: egui::Layout) -> Self {
        self.inner_layout = layout;
        self
    }

    pub fn ui<R>(&self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
        let id = ui.next_auto_id();
        let previous_size = ui.data(|data| data.get_temp(id)).unwrap_or(self.size);
        let sides = previous_size / 2.0;
        let available_rect = ui.available_rect_before_wrap();
        let half_size = self.size / 2.0;
        let rect = egui::Rect {
            min: available_rect.min + half_size - sides,
            max: available_rect.min + self.size
        };
        let mut child_ui = ui.child_ui(rect, self.inner_layout);
        let ret = add_contents(&mut child_ui);
        let final_child_rect = child_ui.min_rect();

        ui.advance_cursor_after_rect(final_child_rect);

        ui.data_mut(|writer| writer.insert_temp(id, final_child_rect.size()));
        ret
    }
}
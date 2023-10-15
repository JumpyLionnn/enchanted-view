

pub trait PainterEx {
    fn debug_stroke(&self, rect: egui::Rect);
    fn debug_line(&self, points: [egui::Pos2; 2]);
    fn debug_label(&self, pos: egui::Pos2, text: impl ToString);
}

impl PainterEx for egui::Painter {
    fn debug_stroke(&self, rect: egui::Rect) {
        self.rect_stroke(rect, egui::Rounding::ZERO, egui::Stroke::new(2.0, egui::Color32::GREEN));
    }

    fn debug_line(&self, points: [egui::Pos2; 2]) {
        self.line_segment(points, egui::Stroke::new(4.0, egui::Color32::RED));
    }
    fn debug_label(&self, pos: egui::Pos2, text: impl ToString) {
        // TODO: add a better debug panel capabilities
        self.debug_text(pos, egui::Align2::LEFT_TOP, egui::Color32::WHITE, text);
    }
}
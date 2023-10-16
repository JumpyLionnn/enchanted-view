pub struct ImageButton<'a> {
    tint: egui::Color32,
    disabled_tint: Option<egui::Color32>,
    enabled: bool,
    tooltip: Option<egui::WidgetText>,
    image: egui::ImageSource<'a>
}

impl<'a> ImageButton<'a> {
    pub fn new(image: egui::ImageSource<'a>) -> Self {
        Self { 
            tint: egui::Color32::WHITE, 
            disabled_tint: None, 
            enabled: true, 
            tooltip: None, 
            image 
        }
    }

    pub fn tint(mut self, color: egui::Color32) -> Self {
        self.tint = color;
        self
    }

    pub fn disabled_tint(mut self, color: egui::Color32) -> Self {
        self.disabled_tint = Some(color);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn tooltip(mut self, text: impl Into<egui::WidgetText>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let image = egui::Image::new(self.image)
            .tint(if self.enabled {self.tint} else {self.disabled_tint.unwrap_or(self.tint)});
        let button = egui::ImageButton::new(image);
        let res = ui.add_enabled(self.enabled, button);
        if let Some(tooltip) = self.tooltip {
            res.on_hover_text(tooltip)
        }
        else {
            res
        }
    }
}
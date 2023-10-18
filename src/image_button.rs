pub struct ImageButton<'a> {
    tint: egui::Color32,
    disabled_tint: Option<egui::Color32>,
    enabled: bool,
    selected: bool,
    tooltip: Option<egui::WidgetText>,
    image: egui::ImageSource<'a>,
    max_height: f32,
    maintain_aspect_ratio: bool
}

impl<'a> ImageButton<'a> {
    pub fn new(image: egui::ImageSource<'a>) -> Self {
        Self { 
            tint: egui::Color32::WHITE, 
            disabled_tint: None, 
            enabled: true,
            selected: false,
            tooltip: None, 
            image,
            max_height: f32::INFINITY,
            maintain_aspect_ratio: true 
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

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn maintain_aspect_ratio(mut self, maintain_aspect_ratio: bool) -> Self {
        self.maintain_aspect_ratio = maintain_aspect_ratio;
        self
    }

    pub fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let image = egui::Image::new(self.image)
            .tint(if self.enabled {self.tint} else {self.disabled_tint.unwrap_or(self.tint)})
            .max_height(self.max_height).maintain_aspect_ratio(self.maintain_aspect_ratio);
        let button = egui::ImageButton::new(image).selected(self.selected);
        let res = ui.add_enabled(self.enabled, button);
        if let Some(tooltip) = self.tooltip {
            res.on_hover_text(tooltip)
        }
        else {
            res
        }
    }
}
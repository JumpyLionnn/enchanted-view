use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};


pub struct ImageButtonStyle {
    pub color: egui::Color32,
    pub disabled_color: egui::Color32,
    pub active_bg: egui::Color32,
    pub hover_bg: egui::Color32
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ThemeKind {
    Light,
    Dark
}

impl Default for ThemeKind {
    fn default() -> Self {
        let mode = dark_light::detect();
        match mode {
            dark_light::Mode::Dark => ThemeKind::Dark,
            dark_light::Mode::Light => ThemeKind::Light,
            dark_light::Mode::Default => ThemeKind::Light
        }
    }
}

pub struct Theme {
    visuals: egui::Visuals,
    checkerboard_pattern_colors: [egui::Color32; 2],
    image_button: ImageButtonStyle,
    strong_overlay_background: egui::Color32,
    overlay_text_color: egui::Color32
}

impl Theme {
    pub fn get(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Light => Self::light(),
            ThemeKind::Dark => Self::dark(),
        }
    }

    pub fn light() -> Self {
        let mut visuals = egui::Visuals::light();
        visuals.indent_has_left_vline = false;
        Self { 
            visuals: visuals, 
            checkerboard_pattern_colors: [egui::Color32::WHITE, egui::Color32::LIGHT_GRAY], 
            image_button: ImageButtonStyle { 
                color: egui::Color32::BLACK, 
                disabled_color: egui::Color32::DARK_GRAY,
                active_bg: egui::Color32::from_gray(200),
                hover_bg: egui::Color32::from_gray(230)
            },
            strong_overlay_background: egui::Color32::from_black_alpha(150),
            overlay_text_color: egui::Color32::WHITE
        }
    }
    pub fn dark() -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.indent_has_left_vline = false;
        Self { 
            visuals: visuals, 
            checkerboard_pattern_colors: [egui::Color32::from_gray(60), egui::Color32::DARK_GRAY], 
            image_button: ImageButtonStyle { 
                color: egui::Color32::WHITE, 
                disabled_color: egui::Color32::LIGHT_GRAY,
                active_bg: egui::Color32::from_gray(90),
                hover_bg: egui::Color32::from_gray(60)
            },
            strong_overlay_background: egui::Color32::from_white_alpha(150),
            overlay_text_color: egui::Color32::BLACK
        }
    }

    pub fn text_style(&self) -> BTreeMap<egui::TextStyle, egui::FontId> {
        [
            (egui::TextStyle::Heading, egui::FontId::new(22.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Name("Heading2".into()), egui::FontId::new(19.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Name("ContextHeading".into()), egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(13.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Button, egui::FontId::new(13.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(10.0, egui::FontFamily::Proportional)),
        ].into()
    }

    pub fn visuals(&self) -> &egui::Visuals {
        &self.visuals
    }

    pub fn checkerboard_pattern_colors(&self) -> [egui::Color32; 2] {
        self.checkerboard_pattern_colors
    }

    pub fn strong_overlay_background(&self) -> egui::Color32 {
        self.strong_overlay_background
    }
    pub fn overlay_text_color(&self) -> egui::Color32 {
        self.overlay_text_color
    }

    pub fn image_button(&self) -> &ImageButtonStyle {
        &self.image_button
    }

    pub fn heading2(&self) -> egui::TextStyle {
        egui::TextStyle::Name("Heading2".into())
    }
    
    pub fn heading3(&self) -> egui::TextStyle {
        egui::TextStyle::Name("ContextHeading".into())
    }

    pub fn style_image_button(&self, ui: &mut egui::Ui) {
        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = self.image_button.hover_bg;
        ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
        ui.style_mut().visuals.widgets.active.weak_bg_fill = self.image_button.active_bg;
        ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::TRANSPARENT;
    }
}
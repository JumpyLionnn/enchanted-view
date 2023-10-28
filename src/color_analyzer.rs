
use crate::{button::{close_button, Button}, theme::Theme, settings::Settings, color_name::color_to_name, utilities::num_length};

pub struct ColorAnalyzerOpenState {
    pub picking_color: bool
}

pub struct ColorAnalyzer {
    pub open: Option<ColorAnalyzerOpenState>,
    color: egui::Color32
}

impl ColorAnalyzer {
    pub fn new() -> Self {
        Self { 
            open: None, 
            color: egui::Color32::TRANSPARENT
        }
    }

    pub fn toggle(&mut self) {
        if self.is_open() {
            self.open = None;
        }
        else {
            self.open = Some(ColorAnalyzerOpenState { picking_color: false });
        }
    }

    pub fn toggle_color_picker(&mut self) {
        if let Some(open_state) = self.open.as_mut() {
            open_state.picking_color = !open_state.picking_color;
        } 
        else {
            self.open = Some(ColorAnalyzerOpenState { picking_color: true });
        }
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn is_picking_color(&self) -> bool {
        self.open.as_ref().is_some_and(|opened_state| opened_state.picking_color)
    }

    pub fn set_color(&mut self, color: egui::Color32) {
        self.color = color;
    }
}

impl ColorAnalyzer {
    pub fn ui(&mut self, ui: &mut egui::Ui, enabled: bool, theme: &Theme, settings: &Settings) {
        // Need to keep in mind that the color analyzer will be visible even if self.image_analyzer is none
        // because of the animation.
        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), 50.0), egui::Layout::left_to_right(egui::Align::Min), |ui|{
            if close_button(ui).clicked() {
                self.open = None;
            }
            ui.label("Color Analyzer");
        });
        let image = egui::Image::new(egui::include_image!("../assets/color_picker.png"))
            .tint(if enabled {theme.image_button().color} else {theme.image_button().disabled_color});
        let pick_button = Button::image_and_text(image, "Pick Color")
            .selected(self.is_picking_color())
            .shortcut_text(ui.ctx().format_shortcut(&settings.key_binds.pick_color));
        if ui.add_enabled(enabled, pick_button).clicked() {
            if let Some(open_state) = self.open.as_mut() {
                open_state.picking_color = !open_state.picking_color;
            }
        }

        ui.spacing_mut().slider_width = ui.available_width();
        egui::widgets::color_picker::color_picker_color32(ui, &mut self.color, egui::widgets::color_picker::Alpha::OnlyBlend);
        ui.add_space(5.0);
        color_name_display(ui, self.color);
        rgb_display(ui, &mut self.color);
        hex_display(ui, &mut self.color);
        

        ui.label(egui::RichText::new("Known Bugs").text_style(theme.heading3()));
        ui.label("Picking colors doesn't work with rotation and flipping.");
        ui.label("The hex color picker is pretty bad.");
        ui.label("There are a lot of missing color names.");
    }
}

fn color_name_display(ui: &mut egui::Ui, color: egui::Color32) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("Color name:");
            ui.add_space(ui.spacing().item_spacing.x * 0.5);
            ui.label(color_to_name((color.r(), color.g(), color.b())).unwrap_or("unknown"));
        });
    });
} 

fn rgb_display(ui: &mut egui::Ui, color: &mut egui::Color32) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            let max_value = 255;
            let char_count = num_length(max_value as usize, 10);
            ui.spacing_mut().interact_size = egui::vec2(calc_char_str_width(ui, &digits, char_count, egui::TextStyle::Button), 10.0);
            ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.visuals_mut().widgets.hovered.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.visuals_mut().widgets.active.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.label("RGB");
            ui.add_space(ui.spacing().item_spacing.x * 0.5);
            let speed = 1.5;
            ui.add(egui::DragValue::new(&mut color[0]).clamp_range(0..=max_value).speed(speed));
            ui.add(egui::DragValue::new(&mut color[1]).clamp_range(0..=max_value).speed(speed));
            ui.add(egui::DragValue::new(&mut color[2]).clamp_range(0..=max_value).speed(speed));
        });
    });
}



fn hex_display(ui: &mut egui::Ui, color: &mut egui::Color32) {
    // TODO: make the hex input better
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let previous_spacing = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
            ui.spacing_mut().button_padding = egui::Vec2::ZERO;
            ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.visuals_mut().widgets.hovered.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().drag_value_text_style = egui::TextStyle::Monospace;
            ui.label("HEX");
            ui.add_space(previous_spacing * 0.5);
            ui.label("#");
            let digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
            let width = calc_char_str_width(ui, &digits, 6, egui::TextStyle::Button);
            let id= ui.auto_id_with("_hex");
            let mut hex = ui.data(|reader| reader.get_temp(id))
                .unwrap_or(format!("{:0>2X}{:0>2X}{:0>2X}", color.r(), color.g(), color.b()));
            let output = egui::TextEdit::singleline(&mut hex).char_limit(7).desired_width(width).margin(egui::vec2(0.0, 2.0)).show(ui);
            if output.response.changed() {
                hex = hex.to_uppercase();
                hex.retain(|char| digits.contains(&char));
                hex.truncate(6);
                if let Some((red, green, blue)) = try_parse(&hex) {
                    *color = egui::Color32::from_rgba_premultiplied(red, green, blue, color.a());
                }
                ui.data_mut(|writer| writer.insert_temp(id, hex));
            }
            else if output.response.lost_focus() {
                if let Some((red, green, blue)) = try_parse(&hex) {
                    *color = egui::Color32::from_rgba_premultiplied(red, green, blue, color.a());
                }
                else {
                    ui.data_mut(|writer| writer.remove::<String>(id));
                }
            }
        });
    });
}


fn try_parse(str: &str) -> Option<(u8, u8, u8)> {
    match str.len() {
        3 => {
            let mut color = str.chars().filter_map(|char| {
                u8::from_str_radix(&char.to_string(), 16).ok()
                    .and_then(|num| Some(num * 16 + num))
            });
            Some((color.next()?, color.next()?, color.next()?))
        },
        6 => {
            let red = u8::from_str_radix(&str[0..=1], 16).ok()?;
            let green = u8::from_str_radix(&str[2..=3], 16).ok()?;
            let blue = u8::from_str_radix(&str[4..=5], 16).ok()?;
            Some((red, green, blue))
        }
        _ => None
    }
}

fn calc_char_str_width(ui: &egui::Ui, chars: &[char], count: usize, text_style: egui::TextStyle) -> f32 {
    let font_id = ui.style().text_styles.get(&text_style).expect("This text style must exist");
    let (_glyph_width, char) = ui.fonts(|fonts| {
        chars
            .iter()
            .map(|char| (fonts.glyph_width(font_id, *char), char))
            .max_by(|(a, _), (b, _)| a.total_cmp(b))
        })
        .expect("Max width must be valid.");
    let widget_text = egui::WidgetText::from(char.to_string().repeat(count))
        .into_galley(ui, Some(false), f32::INFINITY, text_style);
    widget_text.size().x
}
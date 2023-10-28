use crate::{theme::{ThemeKind, Theme}, widgets::{RadioValue, select, key_bind_widget, KeyboardShortcutRef, switch}};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyBinds {
    #[serde(with = "KeyboardShortcutRef")]
    pub next_image: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub previous_image: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub zoom_in: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub zoom_out: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub zoom_to_fit: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub zoom_to_original: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub rotate: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub flip_horizontal: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub flip_vertical: egui::KeyboardShortcut,
    #[serde(with = "KeyboardShortcutRef")]
    pub pick_color: egui::KeyboardShortcut
}

impl Default for KeyBinds {
    fn default() -> Self {
        Self { 
            next_image: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowRight), 
            previous_image: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::ArrowLeft), 
            zoom_in: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::PlusEquals), 
            zoom_out: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Minus), 
            zoom_to_fit: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::F), 
            zoom_to_original: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::O), 
            rotate: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::R), 
            flip_horizontal: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::H), 
            flip_vertical: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::V),
            pick_color: egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::P)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub image_filtering: egui::TextureFilter,
    pub theme: ThemeKind,
    pub key_binds: KeyBinds,
    pub experimental_features: bool
}

impl Settings {
    pub fn load(ctx: &egui::Context) -> Self {
        ctx.data_mut(|reader| {
            reader.get_persisted(egui::Id::new("settings"))
                .unwrap_or(Self { 
                    image_filtering: egui::TextureFilter::Nearest,
                    theme: ThemeKind::default(),
                    key_binds: KeyBinds::default(),
                    experimental_features: false
                })
        })
    }

    pub fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|writer| {
            writer.insert_persisted(egui::Id::new("settings"), self.clone()); 
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, theme: &Theme) -> Option<SettingsUpdate> {
        let mut update = None;
        ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.add_space(10.0);
            ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::top_down(egui::Align::Min),|ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.label(egui::RichText::new("Visuals").text_style(theme.heading2()));
                    ui.label("Theme");
                    let theme_changed = select(ui, "theme_select", &mut self.theme, vec![RadioValue::new("Light theme", ThemeKind::Light), RadioValue::new("Dark theme", ThemeKind::Dark)]);
                    if theme_changed {
                        update = Some(SettingsUpdate::ReloadTheme);
                    }
            
                    ui.label("Texture filter");
                    let filter_options = vec![
                        RadioValue::new("Nearest, I want to see the pixels.", egui::TextureFilter::Nearest), 
                        RadioValue::new("Linear, I want a smooth image.", egui::TextureFilter::Linear)
                    ];
                    let filter_changed = select(ui, "texture_filter_select", &mut self.image_filtering, filter_options);
                    if filter_changed {
                        update = Some(SettingsUpdate::ReloadTexture);
                    }
            
                    self.key_binds(ui, theme);
        
                    ui.horizontal(|ui| {
                        let res = ui.add(switch(&mut self.experimental_features));
                        if res.changed() {
                            update = Some(SettingsUpdate::CloseColorAnalyzer);
                        }
                        ui.label("Enable experimental features");
                    });
                    if self.experimental_features {
                        ui.label(egui::RichText::new("*The experimental features aren't done and might have some bugs in them. Be careful.").color(ui.visuals().warn_fg_color).text_style(egui::TextStyle::Small));
                    }
                });
            });
        });
        self.store(ui.ctx());
        update
    }


    fn key_binds(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        ui.label(egui::RichText::new("Key binds").text_style(theme.heading2()));
        egui::Grid::new("key_binds_grid").show(ui, |ui| {
            // changing the size so all key combinations will fit inside
            ui.spacing_mut().interact_size = ui.spacing().interact_size * egui::vec2(2.1, 1.0);
            let default_key_binds = KeyBinds::default();
            key_bind_widget(ui, "Next image", &mut self.key_binds.next_image, default_key_binds.next_image);
            key_bind_widget(ui, "Previous image", &mut self.key_binds.previous_image, default_key_binds.previous_image);
            key_bind_widget(ui, "Zoom in", &mut self.key_binds.zoom_in, default_key_binds.zoom_in);
            key_bind_widget(ui, "Zoom out", &mut self.key_binds.zoom_out, default_key_binds.zoom_out);
            key_bind_widget(ui, "Zoom to fit", &mut self.key_binds.zoom_to_fit, default_key_binds.zoom_to_fit);
            key_bind_widget(ui, "Zoom to original", &mut self.key_binds.zoom_to_original, default_key_binds.zoom_to_original);
            key_bind_widget(ui, "Rotate", &mut self.key_binds.rotate, default_key_binds.rotate);
            key_bind_widget(ui, "Flip horizontal", &mut self.key_binds.flip_horizontal, default_key_binds.flip_horizontal);
            key_bind_widget(ui, "Flip vertical", &mut self.key_binds.flip_vertical, default_key_binds.flip_vertical);
            if self.experimental_features {
                key_bind_widget(ui, "Pick color", &mut self.key_binds.pick_color, default_key_binds.pick_color);
            }
        });
    }
}

pub enum SettingsUpdate {
    ReloadTheme,
    ReloadTexture,
    CloseColorAnalyzer
}
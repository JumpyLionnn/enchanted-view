use crate::theme::ThemeKind;
use serde::{Serialize, Deserialize};
use crate::hotkey::KeyboardShortcutRef;

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
}
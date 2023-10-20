use crate::theme::ThemeKind;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub image_filtering: egui::TextureFilter,
    pub theme: ThemeKind
}

impl Settings {
    pub fn load(ctx: &egui::Context) -> Self {
        ctx.data_mut(|reader| {
            reader.get_persisted(egui::Id::new("settings"))
                .unwrap_or(Self { 
                    image_filtering: egui::TextureFilter::Nearest,
                    theme: ThemeKind::default()
                })
        })
    }

    pub fn store(&self, ctx: &egui::Context) {
        ctx.data_mut(|writer| {
            writer.insert_persisted(egui::Id::new("settings"), self.clone()); 
        });
    }
}
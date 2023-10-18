use std::{path::PathBuf, io, fs};

use egui::{Layout, TextureFilter, TextureOptions, TextureHandle};
use image::{DynamicImage, ImageResult, ImageError};
mod drop_down_menu;
mod egui_extensions;
mod image_button;
mod pan_zoom_image;
use drop_down_menu::DropDownMenu;
use egui_extensions::ContextEx;
use image_button::ImageButton;
use pan_zoom_image::PanZoomImage;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Enchanted View",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(EnchantedView::new(&cc.egui_ctx))
        }),
    )
}

struct EnchantedView {
    image: Result<(DynamicImage, PanZoomImage), String>,
    flip_horizontal: bool,
    flip_vertical: bool,
    rotation: usize
}

impl EnchantedView {
    fn new(context: &egui::Context) -> Self {
        context.style_mut(|style| {
            style.interaction.tooltip_delay = 0.5;
        });
        let texture_path = std::env::args_os().skip(1).next();
        let image = match texture_path {
            Some(path) => {
                // TODO: Add an option to change the magnification texture filter
                let load = context.load_texture_file(PathBuf::from(path.clone()), TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Linear,
                });
                image_or_error(load, PathBuf::from(path))
            },
            None => Err("Unable to find image.".to_string()),
        };
       
        Self {
            image: image,
            flip_horizontal: false,
            flip_vertical: false,
            rotation: 0
        }
    }
    

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        // TODO: center the toolbar horizontally
        let toolbar_size = egui::vec2(ui.available_width(), 30.0);
        let toolbar_layout = egui::Layout::left_to_right(egui::Align::Min)
            .with_main_align(egui::Align::Center)
            .with_cross_align(egui::Align::Center);
        ui.allocate_ui_with_layout(toolbar_size, toolbar_layout, |ui| {
            // TODO: Adjust the styles based on the theme
            ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_gray(230);
            ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::TRANSPARENT;
            ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.active.weak_bg_fill = egui::Color32::from_gray(200);
            ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::TRANSPARENT;

            self.zoom_control(ui);
            self.flip_control(ui);
            self.rotate_control(ui);
        });
    }

    fn zoom_control(&mut self, ui: &mut egui::Ui) {
        let zoom_in_button = ImageButton::new(egui::include_image!("../assets/zoom_in.png"))
            .tint(egui::Color32::BLACK)
            .disabled_tint(egui::Color32::DARK_GRAY)
            .enabled(self.image.as_ref().is_ok_and(|(_, display)| display.can_zoom_in()))
            .tooltip("Zoom in");
        if zoom_in_button.ui(ui).clicked() {
            if let Ok((_, display)) = &mut self.image {
                display.zoom_in();
            }
        }

        ui.allocate_ui_with_layout(
            egui::vec2(60.0, ui.available_height()),
            Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.set_enabled(false);
                let text = format!("{:.2}%", self.image.as_ref().and_then(|(_, display)| Ok(display.scale)).unwrap_or(1.0) * 100.0);
                DropDownMenu::new(text, "zoom_display")
                    .width(ui.available_width())
                    .menu_width(120.0)
                    .ui(ui, |ui| {
                        if ui.button("View actual size").clicked() {
                            if let Ok((_, display)) = &mut self.image {
                                display.zoom_to_fit();
                            }
                        }
                        if ui.button("Zoom to fit").clicked() {
                            if let Ok((_, display)) = &mut self.image {
                                display.zoom_to_fit();
                            }
                        }
                    });
            },
        );

        let zoom_out_button = ImageButton::new(egui::include_image!("../assets/zoom_out.png"))
            .tint(egui::Color32::BLACK)
            .disabled_tint(egui::Color32::DARK_GRAY)
            .enabled(self.image.as_ref().is_ok_and(|(_, display)| display.can_zoom_out()))
            .tooltip("Zoom out");
        if zoom_out_button.ui(ui).clicked() {
            if let Ok((_, display)) = &mut self.image {
                display.zoom_out();
            }
        }
    }

    fn flip_control(&mut self, ui: &mut egui::Ui) {
        let flip_horizontal_button = ImageButton::new(egui::include_image!("../assets/flip_horizontal.png"))
            .tint(egui::Color32::BLACK)
            .selected(self.flip_horizontal)
            .enabled(self.image.is_ok())
            .tooltip("Flip horizontal");
        if flip_horizontal_button.ui(ui).clicked() {
            self.flip_horizontal = !self.flip_horizontal;
        }
        let flip_vertical_button = ImageButton::new(egui::include_image!("../assets/flip_vertical.png"))
            .tint(egui::Color32::BLACK)
            .selected(self.flip_vertical)
            .enabled(self.image.is_ok())
            .tooltip("Flip vertical");
        if flip_vertical_button.ui(ui).clicked() {
            self.flip_vertical = !self.flip_vertical;
        }
    }

    fn rotate_control(&mut self, ui: &mut egui::Ui) {
        let rotate_button = ImageButton::new(egui::include_image!("../assets/rotate.png"))
            .tint(egui::Color32::BLACK)
            .enabled(self.image.is_ok())
            .tooltip("Rotate");
        if rotate_button.ui(ui).clicked() {
            self.rotation = (self.rotation + 1) % 4;
        }
    }
}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.toolbar(ui);
            match &mut self.image {
                Ok((_, display)) => {
                    display.update(ui, self.flip_horizontal, self.flip_vertical, self.rotation);
                },
                Err(error) => {
                    ui.centered_and_justified(|ui| {
                        ui.heading(format!("Couldnt load image: {error}"));
                    });
                },
            }
        });
    }
}


fn image_or_error(res: ImageResult<(TextureHandle, DynamicImage)>, path: PathBuf) -> Result<(DynamicImage, PanZoomImage), String> {
   
    match res {
        Ok((handle, image)) => {
            let image_size = egui::vec2(image.width() as f32, image.height() as f32);
            Ok((
                image,
                PanZoomImage::new(true, true, handle, image_size)
            ))
        },
        Err(error) => {
            match error {
                ImageError::IoError(error) => {
                    match error.kind() {
                        io::ErrorKind::NotFound => {
                            Err("The system could not find the file specified.".to_string())
                        },
                        io::ErrorKind::PermissionDenied => {
                            if let Ok(metadata) = fs::metadata(path) {
                                // On windows this error is returned if the path is a directory
                                if metadata.is_dir() {
                                    Err("Expected a file path, but a directory was provided.".to_string())
                                }
                                else {
                                    Err("Access is denied.".to_string())
                                }
                            }
                            else {
                                Err(error.to_string())
                            }
                        }
                        _ => Err(error.to_string())
                    }
                }
                other => Err(other.to_string())
            }
        },
    }
}
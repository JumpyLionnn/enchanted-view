use std::{path::PathBuf, io, fs::{self, DirEntry}};

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
            Box::new(EnchantedView::new(cc.egui_ctx.clone()))
        }),
    )
}

struct OpenedImage {
    image: DynamicImage,
    display: PanZoomImage
}

enum Direction {
    Next,
    Previous
}

struct EnchantedView {
    image: Result<OpenedImage, String>,
    path: Option<PathBuf>,
    flip_horizontal: bool,
    flip_vertical: bool,
    rotation: usize,
    context: egui::Context
}

impl EnchantedView {
    fn new(context: egui::Context) -> Self {
        context.style_mut(|style| {
            style.interaction.tooltip_delay = 0.5;
        });
        let texture_path = std::env::args_os().skip(1).next();
        let (image, path) = match texture_path {
            Some(path_value) => {
                // TODO: Add an option to change the magnification texture filter
                let load = context.load_texture_file(PathBuf::from(path_value.clone()), TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Linear,
                });
                let path = if load.as_ref().is_err_and(|error| {
                    match error {
                        ImageError::IoError(_) => false,
                        _ => true
                    }
                }) || load.is_ok() {
                    Some(PathBuf::from(path_value.clone()).canonicalize().expect("Couldn't find absolute path"))
                } else { None };
                (image_or_error(load, PathBuf::from(path_value)), path)
            },
            None => (Err("Unable to find image.".to_string()), None),
        };
        
        Self {
            image,
            path,
            flip_horizontal: false,
            flip_vertical: false,
            rotation: 0,
            context
        }
    }
    

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        // TODO: center the toolbar horizontally
        let toolbar_size = egui::vec2(ui.available_width(), 30.0);
        let toolbar_layout = egui::Layout::left_to_right(egui::Align::Min)
            .with_main_align(egui::Align::Center)
            .with_cross_align(egui::Align::Center);
        ui.allocate_ui_with_layout(toolbar_size, toolbar_layout, |ui| {
            
            self.style_image_button(ui);

            self.zoom_control(ui);
            self.flip_control(ui);
            self.rotate_control(ui);
        });
    }

    fn zoom_control(&mut self, ui: &mut egui::Ui) {
        let zoom_in_button = ImageButton::new(egui::include_image!("../assets/zoom_in.png"))
            .tint(egui::Color32::BLACK)
            .disabled_tint(egui::Color32::DARK_GRAY)
            .enabled(self.image.as_ref().is_ok_and(|opened_image| opened_image.display.can_zoom_in()))
            .tooltip("Zoom in");
        if zoom_in_button.ui(ui).clicked() {
            if let Ok(opened_image) = &mut self.image {
                opened_image.display.zoom_in();
            }
        }

        ui.allocate_ui_with_layout(
            egui::vec2(60.0, ui.available_height()),
            Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.set_enabled(self.image.is_ok());
                let text = format!("{:.2}%", self.image.as_ref().and_then(|opened_image| Ok(opened_image.display.scale)).unwrap_or(1.0) * 100.0);
                DropDownMenu::new(text, "zoom_display")
                    .width(ui.available_width())
                    .menu_width(120.0)
                    .ui(ui, |ui| {
                        if ui.button("View actual size").clicked() {
                            if let Ok(opened_image) = &mut self.image {
                                opened_image.display.zoom_to_original();
                            }
                        }
                        if ui.button("Zoom to fit").clicked() {
                            if let Ok(opened_image) = &mut self.image {
                                opened_image.display.zoom_to_fit();
                            }
                        }
                    });
            },
        );

        let zoom_out_button = ImageButton::new(egui::include_image!("../assets/zoom_out.png"))
            .tint(egui::Color32::BLACK)
            .disabled_tint(egui::Color32::DARK_GRAY)
            .enabled(self.image.as_ref().is_ok_and(|opened_image| opened_image.display.can_zoom_out()))
            .tooltip("Zoom out");
        if zoom_out_button.ui(ui).clicked() {
            if let Ok(opened_image) = &mut self.image {
                opened_image.display.zoom_out();
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

    fn switch_image(&mut self, direction: Direction) {
        if let Some(path) = self.path.as_ref() {
            if let Ok(directory) = fs::read_dir(path.parent().expect("Couldn't get parent directory path")) {
                let entries = directory.filter_map(|element| element.ok()).collect::<Vec<DirEntry>>();
                let current_index = entries.iter().position(|entry| entry.path() == *path);
                if let Some(current_index) = current_index {
                    let find_predicate = |entry: &&DirEntry| {
                        image::ImageFormat::from_path(entry.path()).is_ok()
                    };
                    let next_image = match direction {
                        Direction::Next => entries.iter().skip(current_index + 1).find(find_predicate),
                        Direction::Previous => entries.iter().rev().skip(entries.len() - current_index).find(find_predicate),
                    };
                    if let Some(entry) = next_image {
                        // TODO: rotate around and make sure before there are more images to open
                        // TODO: Preload textures to increase performance
                        let load = self.context.load_texture_file(entry.path(), TextureOptions {
                            magnification: TextureFilter::Nearest,
                            minification: TextureFilter::Linear,
                        });
                        self.image = image_or_error(load, entry.path());
                        self.path = Some(entry.path());
                    }
                }
                else {
                    eprintln!("ERROR: couldnt find the current image in the parent folder");
                }
            }
        }
    }

    fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        let bottom_bar_height = 30.0;

        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), bottom_bar_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.style_image_button(ui);
            let res = ImageButton::new(egui::include_image!("../assets/arrow_left.png"))
                .tint(egui::Color32::BLACK)
                .disabled_tint(egui::Color32::DARK_GRAY)
                .enabled(self.path.is_some())
                .tooltip("Previous image")
                .ui(ui);
            if res.clicked() {
                self.previous_image();
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let res = ImageButton::new(egui::include_image!("../assets/arrow_right.png"))
                    .tint(egui::Color32::BLACK)
                    .disabled_tint(egui::Color32::DARK_GRAY)
                    .enabled(self.path.is_some())
                    .tooltip("Next image")
                    .ui(ui);
                if res.clicked() {
                    self.next_image();
                }
                if let Some(path) = &self.path {
                    ui.centered_and_justified(|ui| {
                        ui.label( path.file_name().expect("The file name does not exist.").to_string_lossy());
                    });
                }
                
            });
        });
    }

    fn style_image_button(&self, ui: &mut egui::Ui) {
        // TODO: Adjust the styles based on the theme
        ui.style_mut().visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_gray(230);
        ui.style_mut().visuals.widgets.hovered.bg_fill = egui::Color32::TRANSPARENT;
        ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
        ui.style_mut().visuals.widgets.active.weak_bg_fill = egui::Color32::from_gray(200);
        ui.style_mut().visuals.widgets.active.bg_fill = egui::Color32::TRANSPARENT;
    }

    fn next_image(&mut self) {
        self.switch_image(Direction::Next);
    }

    fn previous_image(&mut self) {
        self.switch_image(Direction::Previous);
    }
}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.toolbar(ui);
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                self.bottom_bar(ui);  
                ui.separator(); 
                match &mut self.image {
                    Ok(opened_image) => {
                        opened_image.display.update(ui, self.flip_horizontal, self.flip_vertical, self.rotation);
                    },
                    Err(error) => {
                        ui.centered_and_justified(|ui| {
                            ui.heading(format!("Couldnt load image: {error}"));
                        });
                    }
                }
            });
        });
    }
}


fn image_or_error(res: ImageResult<(TextureHandle, DynamicImage)>, path: PathBuf) -> Result<OpenedImage, String> {
   
    match res {
        Ok((handle, image)) => {
            let image_size = egui::vec2(image.width() as f32, image.height() as f32);
            Ok(OpenedImage {
                image: image, 
                display: PanZoomImage::new(true, true, handle, image_size)
                                  })
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
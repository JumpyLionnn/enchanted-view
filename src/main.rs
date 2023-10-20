use std::{path::PathBuf, io, fs};

use center_container::CenterContainer;
use egui::{Layout, TextureFilter, TextureOptions, TextureHandle};
use image::{DynamicImage, ImageResult, ImageError};
mod drop_down_menu;
mod egui_extensions;
mod image_button;
mod pan_zoom_image;
mod button;
mod center_container;
use drop_down_menu::DropDownMenu;
use egui_extensions::ContextEx;
use image_button::ImageButton;
use pan_zoom_image::PanZoomImage;
use button::Button;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
        hardware_acceleration: eframe::HardwareAcceleration::Required,
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

struct ImagePathInfo {
    name: String,
    index: usize,
    children: Vec<PathBuf>
}

enum Direction {
    Next,
    Previous
}

struct EnchantedView {
    image: Result<OpenedImage, String>,
    path_info: Option<ImagePathInfo>,
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
                let image_path = PathBuf::from(path_value.clone());
                let load = context.load_texture_file(&image_path, TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Linear,
                });
                let path = if load.as_ref().is_err_and(|error| {
                    match error {
                        ImageError::IoError(_) => false,
                        _ => true
                    }
                }) || load.is_ok() {
                    let path = image_path.canonicalize().expect("Couldn't find absolute path");
                    Some(find_sibling_images(&path))
                } else { None };
                (image_or_error(load, &image_path), path)
            },
            None => (Err("Unable to find image.".to_string()), None),
        };
        
        Self {
            image,
            path_info: path,
            flip_horizontal: false,
            flip_vertical: false,
            rotation: 0,
            context
        }
    }
    

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let toolbar_size = egui::vec2(ui.available_width(), 30.0);
        let toolbar_layout = egui::Layout::left_to_right(egui::Align::Min);
        CenterContainer::new(toolbar_size).inner_layout(toolbar_layout).ui(ui, |ui| {
            
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
            .tooltip("Zoom in (+)");
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
                        if ui.add(Button::new("View actual size").shortcut_text("O")).clicked() {
                            if let Ok(opened_image) = &mut self.image {
                                opened_image.display.zoom_to_original();
                            }
                        }
                        if ui.add(Button::new("Zoom to fit").shortcut_text("F")).clicked() {
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
            .tooltip("Zoom out (-)");
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
            .tooltip("Flip horizontal (H)");
        if flip_horizontal_button.ui(ui).clicked() {
            self.flip_horizontal = !self.flip_horizontal;
        }
        let flip_vertical_button = ImageButton::new(egui::include_image!("../assets/flip_vertical.png"))
            .tint(egui::Color32::BLACK)
            .selected(self.flip_vertical)
            .enabled(self.image.is_ok())
            .tooltip("Flip vertical (V)");
        if flip_vertical_button.ui(ui).clicked() {
            self.flip_vertical = !self.flip_vertical;
        }
    }

    fn rotate_control(&mut self, ui: &mut egui::Ui) {
        let rotate_button = ImageButton::new(egui::include_image!("../assets/rotate.png"))
            .tint(egui::Color32::BLACK)
            .enabled(self.image.is_ok())
            .tooltip("Rotate (R)");
        if rotate_button.ui(ui).clicked() {
            self.rotation = (self.rotation + 1) % 4;
        }
    }

    fn switch_image(&mut self, direction: Direction) {
        if let Some(path_info) = self.path_info.as_mut() {
            match direction {
                Direction::Next => {
                    if path_info.index == path_info.children.len() - 1 {
                        path_info.index = 0;
                    }
                    else {
                        path_info.index += 1;
                    }
                },
                Direction::Previous => {
                    if path_info.index == 0 {
                        path_info.index = path_info.children.len() - 1;
                    }
                    else {
                        path_info.index -= 1;
                    }
                },
            }
            let new_path = &path_info.children[path_info.index];
            let load = self.context.load_texture_file(new_path, TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Linear,
            });
            path_info.name = new_path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string();
            self.image = image_or_error(load, new_path);
        }
    }

    fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        let bottom_bar_height = 30.0;

        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), bottom_bar_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.style_image_button(ui);
            let res = ImageButton::new(egui::include_image!("../assets/arrow_left.png"))
                .tint(egui::Color32::BLACK)
                .disabled_tint(egui::Color32::DARK_GRAY)
                .enabled(self.path_info.is_some())
                .tooltip("Previous image (Left arrow)")
                .ui(ui);
            if res.clicked() {
                self.previous_image();
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let res = ImageButton::new(egui::include_image!("../assets/arrow_right.png"))
                    .tint(egui::Color32::BLACK)
                    .disabled_tint(egui::Color32::DARK_GRAY)
                    .enabled(self.path_info.is_some())
                    .tooltip("Next image (Right arrow)")
                    .ui(ui);
                if res.clicked() {
                    self.next_image();
                }
                if let Some(path_info) = &self.path_info {
                    ui.centered_and_justified(|ui| {
                        ui.label(format!("{} ({}/{})", &path_info.name, path_info.index + 1, path_info.children.len()));
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

    fn hotkeys(&mut self, input: &egui::InputState) {
        if input.key_pressed(egui::Key::ArrowRight) {
            self.next_image();
        }
        if input.key_pressed(egui::Key::ArrowLeft) {
            self.previous_image();
        }
        if let Ok(image) = &mut self.image {
            if input.key_pressed(egui::Key::PlusEquals) {
                image.display.zoom_in();
            }
            if input.key_pressed(egui::Key::Minus) {
                image.display.zoom_out();
            }
            if input.key_pressed(egui::Key::F) {
                image.display.zoom_to_fit();
            }
            if input.key_pressed(egui::Key::O) {
                image.display.zoom_to_original();
            }
            if input.key_pressed(egui::Key::R) {
                self.rotation = (self.rotation + 1) % 4;
            }
            if input.key_pressed(egui::Key::H) {
                self.flip_horizontal = !self.flip_horizontal;
            }
            if input.key_pressed(egui::Key::V) {
                self.flip_vertical = !self.flip_vertical;
            }
        }
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
            ui.input(|input| self.hotkeys(input));
        });
    }
}

fn find_sibling_images(image_path: &PathBuf) -> ImagePathInfo {
    let parent = image_path.parent().expect("There isn't a parent path for the image path");
    let image_entries = fs::read_dir(parent).expect("Couldn't read the parent dir")
        .filter_map(|element| {
            element.ok().and_then(|entry| {
                if image::ImageFormat::from_path(entry.path()).is_ok() {
                    Some(entry.path())
                }
                else {
                    None
                }
            })
        })
        .collect::<Vec<PathBuf>>();
    let current_index = image_entries.iter().position(|path| path == image_path).expect("Couldn't find the current image.");
    ImagePathInfo { 
        name: image_path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string(), 
        index: current_index, 
        children: image_entries 
    }
}


fn image_or_error(res: ImageResult<(TextureHandle, DynamicImage)>, path: &PathBuf) -> Result<OpenedImage, String> {
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
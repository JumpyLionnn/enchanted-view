#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::PathBuf, io, fs};

use center_container::CenterContainer;
use egui::{Layout, TextureFilter, TextureOptions, TextureHandle};
use hotkey::key_bind_widget;
use image::{DynamicImage, ImageResult, ImageError};
mod drop_down_menu;
mod egui_extensions;
mod image_button;
mod pan_zoom_image;
mod button;
mod center_container;
mod select;
mod theme;
mod settings;
mod hotkey;
use select::{select, RadioValue};
use settings::{Settings, KeyBinds};
use drop_down_menu::DropDownMenu;
use egui_extensions::ContextEx;
use image_button::ImageButton;
use pan_zoom_image::PanZoomImage;
use button::Button;
use theme::{Theme, ThemeKind};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
        icon_data: Some(eframe::IconData::try_from_png_bytes(include_bytes!("../assets/icon.png")).expect("The icon should be valid.")),
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
    context: egui::Context,
    theme: Theme,
    settings_screen: bool,
    settings: Settings
}

impl EnchantedView {
    fn new(context: egui::Context) -> Self {
        let settings = Settings::load(&context);
        let theme = Theme::get(settings.theme.clone());
        context.style_mut(|style| {
            style.interaction.tooltip_delay = 0.5;
            style.visuals = theme.visuals().clone();
            style.text_styles = theme.text_style();
        });
        let texture_path = std::env::args_os().skip(1).next();
        let (image, path) = match texture_path {
            Some(path_value) => {
                let image_path = PathBuf::from(path_value.clone());
                let load = context.load_texture_file(&image_path, TextureOptions {
                    magnification: settings.image_filtering,
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
                (image_or_error(load, &image_path, &theme), path)
            },
            None => (Err("Unable to find image.".to_string()), None),
        };
        Self {
            image,
            path_info: path,
            flip_horizontal: false,
            flip_vertical: false,
            rotation: 0,
            context,
            theme,
            settings_screen: false,
            settings
        }
    }
    

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let toolbar_size = egui::vec2(ui.available_width(), 30.0);
        let toolbar_layout = egui::Layout::left_to_right(egui::Align::Min);
        ui.allocate_ui_with_layout(toolbar_size, toolbar_layout, |ui| {
            self.theme.style_image_button(ui);
            CenterContainer::new(toolbar_size).inner_layout(toolbar_layout).ui(ui, |ui| {
                
    
                self.zoom_control(ui);
                self.flip_control(ui);
                self.rotate_control(ui);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let settings_button = ImageButton::new(egui::include_image!("../assets/settings.png"))
                    .tint(self.theme.image_button().color);
                if settings_button.ui(ui).clicked() {
                    self.settings_screen = true;
                }
            });
        });
    }

    fn zoom_control(&mut self, ui: &mut egui::Ui) {
        let zoom_in_button = ImageButton::new(egui::include_image!("../assets/zoom_in.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .enabled(self.image.as_ref().is_ok_and(|opened_image| opened_image.display.can_zoom_in()))
            .tooltip(format!("Zoom in ({})", ui.ctx().format_shortcut(&self.settings.key_binds.zoom_in)));
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
                        if ui.add(Button::new("View actual size").shortcut_text(ui.ctx().format_shortcut(&self.settings.key_binds.zoom_to_original))).clicked() {
                            if let Ok(opened_image) = &mut self.image {
                                opened_image.display.zoom_to_original();
                            }
                        }
                        if ui.add(Button::new("Zoom to fit").shortcut_text(ui.ctx().format_shortcut(&self.settings.key_binds.zoom_to_fit))).clicked() {
                            if let Ok(opened_image) = &mut self.image {
                                opened_image.display.zoom_to_fit();
                            }
                        }
                    });
            },
        );

        let zoom_out_button = ImageButton::new(egui::include_image!("../assets/zoom_out.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .enabled(self.image.as_ref().is_ok_and(|opened_image| opened_image.display.can_zoom_out()))
            .tooltip(format!("Zoom out ({})", ui.ctx().format_shortcut(&self.settings.key_binds.zoom_out)));
        if zoom_out_button.ui(ui).clicked() {
            if let Ok(opened_image) = &mut self.image {
                opened_image.display.zoom_out();
            }
        }
    }

    fn flip_control(&mut self, ui: &mut egui::Ui) {
        let flip_horizontal_button = ImageButton::new(egui::include_image!("../assets/flip_horizontal.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .selected(self.flip_horizontal)
            .enabled(self.image.is_ok())
            .tooltip(format!("Flip horizontal ({})", ui.ctx().format_shortcut(&self.settings.key_binds.flip_horizontal)));
        if flip_horizontal_button.ui(ui).clicked() {
            self.flip_horizontal = !self.flip_horizontal;
        }
        let flip_vertical_button = ImageButton::new(egui::include_image!("../assets/flip_vertical.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .selected(self.flip_vertical)
            .enabled(self.image.is_ok())
            .tooltip(format!("Flip vertical ({})", ui.ctx().format_shortcut(&self.settings.key_binds.flip_vertical)));
        if flip_vertical_button.ui(ui).clicked() {
            self.flip_vertical = !self.flip_vertical;
        }
    }

    fn rotate_control(&mut self, ui: &mut egui::Ui) {
        let rotate_button = ImageButton::new(egui::include_image!("../assets/rotate.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .enabled(self.image.is_ok())
            .tooltip(format!("Rotate ({})", ui.ctx().format_shortcut(&self.settings.key_binds.rotate)));
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
                magnification: self.settings.image_filtering,
                minification: TextureFilter::Linear,
            });
            path_info.name = new_path.file_name().expect("Couldn't extract the file name").to_string_lossy().to_string();
            self.image = image_or_error(load, new_path, &self.theme);
        }
    }

    fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        let bottom_bar_height = 30.0;

        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), bottom_bar_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.theme.style_image_button(ui);
            let res = ImageButton::new(egui::include_image!("../assets/arrow_left.png"))
                .tint(self.theme.image_button().color)
                .disabled_tint(self.theme.image_button().disabled_color)
                .enabled(self.path_info.is_some())
                .tooltip(format!("Previous image ({})", ui.ctx().format_shortcut(&self.settings.key_binds.previous_image)))
                .ui(ui);
            if res.clicked() {
                self.previous_image();
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let res = ImageButton::new(egui::include_image!("../assets/arrow_right.png"))
                    .tint(self.theme.image_button().color)
                    .disabled_tint(self.theme.image_button().disabled_color)
                    .enabled(self.path_info.is_some())
                    .tooltip(format!("Next image ({})", ui.ctx().format_shortcut(&self.settings.key_binds.next_image)))
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

    fn next_image(&mut self) {
        self.switch_image(Direction::Next);
    }

    fn previous_image(&mut self) {
        self.switch_image(Direction::Previous);
    }

    fn hotkeys(&mut self, ui: &mut egui::Ui) {
        if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.next_image)) {
            self.next_image();
        }
        if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.previous_image)) {
            self.previous_image();
        }
        if let Ok(image) = &mut self.image {
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_in)) {
                image.display.zoom_in();
            }
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_out)) {
                image.display.zoom_out();
            }
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_to_fit)) {
                image.display.zoom_to_fit();
            }
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_to_original)) {
                image.display.zoom_to_original();
            }
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.rotate)) {
                self.rotation = (self.rotation + 1) % 4;
            }
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.flip_horizontal)) {
                self.flip_horizontal = !self.flip_horizontal;
            }
            if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.flip_vertical)) {
                self.flip_vertical = !self.flip_vertical;
            }
        }
    }

    

    fn main_screen(&mut self, ui: &mut egui::Ui) {
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
        self.hotkeys(ui);
    }

    fn update_theme(&mut self) {
        self.theme = Theme::get(self.settings.theme.clone());
        self.context.style_mut(|style| {
            style.visuals = self.theme.visuals().clone();
            style.text_styles = self.theme.text_style();
        });
        if let Ok(image) = &mut self.image {
            image.display.change_checkerboard_color(self.theme.checkerboard_pattern_colors());
        }
    }

    fn reload_texture(&mut self) {
        if let Ok(image) = &mut self.image {
            let name = &self.path_info.as_ref().expect("The path info should be valid if the image is").name;
            let options = TextureOptions { magnification: self.settings.image_filtering, minification: TextureFilter::Linear };
            let handle = self.context.load_texture_from_image(&image.image, options, name);
            image.display.texture_handle = handle;
        }
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        let toolbar_size = egui::vec2(ui.available_width(), 30.0);
        ui.allocate_ui_with_layout(toolbar_size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.theme.style_image_button(ui);
            let back_button = ImageButton::new(egui::include_image!("../assets/back_arrow.png"))
                .tint(self.theme.image_button().color);
            if back_button.ui(ui).clicked() {
                self.settings_screen = false;
                self.settings.store(&self.context);
            }
            ui.heading("Settings");
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.label(egui::RichText::new("Visuals").text_style(self.theme.heading2()));
            ui.label("Theme");
            let theme_changed = select(ui, "theme_select", &mut self.settings.theme, vec![RadioValue::new("Light theme", ThemeKind::Light), RadioValue::new("Dark theme", ThemeKind::Dark)]);
            if theme_changed {
                self.update_theme();
            }
    
            ui.label("Texture filter");
            let filter_options = vec![
                RadioValue::new("Nearest, I want to see the pixels.", TextureFilter::Nearest), 
                RadioValue::new("Linear, I want a smooth image.", TextureFilter::Linear)
            ];
            let filter_changed = select(ui, "texture_filter_select", &mut self.settings.image_filtering, filter_options);
            if filter_changed {
                self.reload_texture();
            }
    
            self.key_binds(ui);
        });
    }

    fn key_binds(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Key binds").text_style(self.theme.heading2()));
        egui::Grid::new("key_binds_grid").show(ui, |ui| {
            // changing the size so all key combinations will fit inside
            ui.spacing_mut().interact_size = ui.spacing().interact_size * egui::vec2(2.1, 1.0);
            let default_key_binds = KeyBinds::default();
            key_bind_widget(ui, "Next image", &mut self.settings.key_binds.next_image, default_key_binds.next_image);
            key_bind_widget(ui, "Previous image", &mut self.settings.key_binds.previous_image, default_key_binds.previous_image);
            key_bind_widget(ui, "Zoom in", &mut self.settings.key_binds.zoom_in, default_key_binds.zoom_in);
            key_bind_widget(ui, "Zoom out", &mut self.settings.key_binds.zoom_out, default_key_binds.zoom_out);
            key_bind_widget(ui, "Zoom to fit", &mut self.settings.key_binds.zoom_to_fit, default_key_binds.zoom_to_fit);
            key_bind_widget(ui, "Zoom to original", &mut self.settings.key_binds.zoom_to_original, default_key_binds.zoom_to_original);
            key_bind_widget(ui, "Rotate", &mut self.settings.key_binds.rotate, default_key_binds.rotate);
            key_bind_widget(ui, "Flip horizontal", &mut self.settings.key_binds.flip_horizontal, default_key_binds.flip_horizontal);
            key_bind_widget(ui, "Flip vertical", &mut self.settings.key_binds.flip_vertical, default_key_binds.flip_vertical);
        });
    }
}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.settings_screen {
                self.settings_screen(ui);
            }
            else {
                self.main_screen(ui);
            }
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


fn image_or_error(res: ImageResult<(TextureHandle, DynamicImage)>, path: &PathBuf, theme: &Theme) -> Result<OpenedImage, String> {
    match res {
        Ok((handle, image)) => {
            let image_size = egui::vec2(image.width() as f32, image.height() as f32);
            Ok(OpenedImage {
                image: image, 
                display: PanZoomImage::new(
                    true, 
                    true, 
                    handle, 
                    image_size, 
                    theme.checkerboard_pattern_colors()
                )
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


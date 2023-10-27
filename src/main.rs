#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::PathBuf, io, fs};

use center_container::CenterContainer;
use color_analyzer::ColorAnalyzer;
use egui::{Layout, TextureFilter, TextureOptions, TextureHandle, output};
use file_dialog::{FileDialogHandle, FileDialog};
use hotkey::key_bind_widget;
use image::{DynamicImage, ImageResult, ImageError, ImageFormat, GenericImageView};
mod drop_down_menu;
mod egui_extensions;
mod image_button;
mod pan_zoom_image;
mod button;
mod center_container;
mod select;
mod theme;
mod image_directory;
mod settings;
mod hotkey;
mod file_dialog;
mod checkerboard_pattern;
mod color_name;
mod key_value_match;
mod color_analyzer;
mod switch;
use image_directory::{ImageDirectory, ImageFormatEx, is_image_file};
use select::{select, RadioValue};
use settings::{Settings, KeyBinds};
use drop_down_menu::DropDownMenu;
use egui_extensions::ContextEx;
use image_button::ImageButton;
use pan_zoom_image::PanZoomImage;
use button::Button;
use switch::toggle;
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

struct EnchantedView {
    image: Result<OpenedImage, String>,
    image_directory: Option<ImageDirectory>,
    flip_horizontal: bool,
    flip_vertical: bool,
    rotation: usize,
    context: egui::Context,
    theme: Theme,
    settings_screen: bool,
    settings: Settings,
    file_dialog: Option<FileDialogHandle>,
    color_analyzer: ColorAnalyzer
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
        let (image, directory) = match texture_path {
            Some(path_value) => {
                let image_path = PathBuf::from(path_value.clone());
                let load = context.load_texture_file(&image_path, TextureOptions {
                    magnification: settings.image_filtering,
                    minification: TextureFilter::Linear,
                });
                let directory = if load.as_ref().is_err_and(|error| {
                    match error {
                        ImageError::IoError(_) => false,
                        _ => true
                    }
                }) || load.is_ok() {
                    let path = image_path.canonicalize().expect("Couldn't find absolute path");
                    Some(ImageDirectory::new(&path).expect("Unable to create the image directory"))
                } else { None };
                (image_or_error(load, &image_path, &theme), directory)
            },
            None => (Err("Unable to find image.".to_string()), None),
        };
        Self {
            image,
            image_directory: directory,
            flip_horizontal: false,
            flip_vertical: false,
            rotation: 0,
            context,
            theme,
            settings_screen: false,
            settings,
            file_dialog: None,
            color_analyzer: ColorAnalyzer::new()
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

                if self.settings.experimental_features {
                    self.color_analyzer_control(ui);
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
                    .menu_width(130.0)
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
        let image = match self.rotation {
            0 => egui::include_image!("../assets/rotate.png"),
            1 => egui::include_image!("../assets/rotate_90.png"),
            2 => egui::include_image!("../assets/rotate_180.png"),
            3 => egui::include_image!("../assets/rotate_270.png"),
            other => {eprintln!("Unknown rotation: {other}"); egui::include_image!("../assets/rotate.png")}
        };
        let rotate_button = ImageButton::new(image)
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .enabled(self.image.is_ok())
            .tooltip(format!("Rotate ({})", ui.ctx().format_shortcut(&self.settings.key_binds.rotate)));
        if rotate_button.ui(ui).clicked() {
            self.rotation = (self.rotation + 1) % 4;
        }
    }

    fn color_analyzer_control(&mut self, ui: &mut egui::Ui) {
        let color_analyzer_button = ImageButton::new(egui::include_image!("../assets/color.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .enabled(self.image.is_ok())
            .selected(self.color_analyzer.is_open())
            .tooltip("Color Analyzer");
        if color_analyzer_button.ui(ui).clicked() {
            self.color_analyzer.toggle();
        }
    }


    fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        let bottom_bar_height = 30.0;

        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), bottom_bar_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.theme.style_image_button(ui);
            let res = ImageButton::new(egui::include_image!("../assets/arrow_left.png"))
                .tint(self.theme.image_button().color)
                .disabled_tint(self.theme.image_button().disabled_color)
                .enabled(self.image_directory.is_some())
                .tooltip(format!("Previous image ({})", ui.ctx().format_shortcut(&self.settings.key_binds.previous_image)))
                .ui(ui);
            if res.clicked() {
                self.previous_image();
            }
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let res = ImageButton::new(egui::include_image!("../assets/arrow_right.png"))
                    .tint(self.theme.image_button().color)
                    .disabled_tint(self.theme.image_button().disabled_color)
                    .enabled(self.image_directory.is_some())
                    .tooltip(format!("Next image ({})", ui.ctx().format_shortcut(&self.settings.key_binds.next_image)))
                    .ui(ui);
                if res.clicked() {
                    self.next_image();
                }
                if let Some(directory) = &self.image_directory {
                    ui.centered_and_justified(|ui| {
                        ui.label(format!("{} ({}/{})", directory.image_name(), directory.image_index() + 1, directory.count()));
                    });
                }
                
            });
        });
    }

    fn load_image(&mut self, path: &PathBuf) {
        let load = self.context.load_texture_file(path, TextureOptions {
            magnification: self.settings.image_filtering,
            minification: TextureFilter::Linear,
        });
        self.image = image_or_error(load, path, &self.theme);
    }

    fn load_image_raw(&mut self, bytes: &[u8], path: &PathBuf) {
        let name = path.to_string_lossy().to_string();
        let load = self.context.load_texture_raw(&name, bytes, TextureOptions {
            magnification: self.settings.image_filtering,
            minification: TextureFilter::Linear,
        });
        self.image = image_or_error(load, path, &self.theme);
    }

    fn next_image(&mut self) {
        if let Some(mut directory) = self.image_directory.take() {
            let path = directory.next_image();
            self.load_image(path);
            self.image_directory = Some(directory);
        }
    }

    fn previous_image(&mut self) {
        if let Some(mut directory) = self.image_directory.take() {
            let path = directory.previous_image();
            self.load_image(path);
            self.image_directory = Some(directory);
        }
    }

    fn hotkeys(&mut self, ui: &mut egui::Ui) {
        if ui.output(|output| output.text_cursor_pos.is_none()) {
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
                if self.settings.experimental_features {
                    if ui.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.pick_color)) {
                        self.color_analyzer.toggle_color_picker();
                    }
                }
            }
        }
    }

    fn handle_drop_files(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        let hovered_file = ui.input(|input| {
            input.raw.hovered_files
                .iter()
                .filter(|file| file.path.as_ref().is_some_and(is_image_file))
                .next().cloned()
        });
        if let Some(hovered_file) = hovered_file {
            let path = hovered_file.path.expect("The path must exist");
            ui.painter().rect_filled(rect, egui::Rounding::ZERO, self.theme.strong_overlay_background());
            ui.allocate_ui_at_rect(rect, |ui| {
                CenterContainer::new(rect.size()).inner_layout(egui::Layout::top_down(egui::Align::Center)).ui(ui, |ui| {
                    let text= format!("Open: {}", path.to_string_lossy());
                    ui.label(egui::RichText::new(text).text_style(egui::TextStyle::Heading).color(self.theme.overlay_text_color()));
                });
            });
            ui.ctx().request_repaint();
        }
        let dropped_file = ui.input(|input| {
            input.raw.dropped_files
                .iter()
                .filter(|file| file.path.as_ref().is_some_and(is_image_file))
                .next().cloned()
        });
        if let Some(dropped_file) = dropped_file {
            let path = dropped_file.path.expect("The path must exist.");
            self.image_directory = Some(ImageDirectory::new(&path).expect("Unable to initialize the image directory."));
            if let Some(bytes) = dropped_file.bytes {
                self.load_image_raw(&*bytes, &path);
            }
            else {
                self.load_image(&path);
                
            }
        }
    }

    fn main_screen(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        let previous_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing.y = 0.0;
        self.toolbar(ui);
        ui.separator(); 
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            self.bottom_bar(ui);  
            ui.separator(); 
            ui.spacing_mut().item_spacing = previous_spacing;
            egui::SidePanel::right("color_analyzer")
                .resizable(true)
                .show_separator_line(true)
                .min_width(150.0)
                .default_width(200.0)
                .max_width(300.0)
                .show_animated_inside(ui, self.color_analyzer.is_open(), |ui| {
                    self.color_analyzer.ui(ui, self.image.is_ok(), &self.theme, &self.settings);
                });
            egui::CentralPanel::default().show_inside(ui, |ui| {
                let res =  match &mut self.image {
                    Ok(opened_image) => {
                        let highlight_pixel = self.color_analyzer.is_picking_color();
                        let res = opened_image.display.update(ui, self.flip_horizontal, self.flip_vertical, self.rotation, highlight_pixel);
                        if res.clicked() && highlight_pixel {
                            let hover_pos = res.hover_pos().expect("There must be a hover pos.");
                            let (x, y) = opened_image.display.get_image_pixel_coords(hover_pos.to_vec2());
                            let pixel = opened_image.image.get_pixel(x, y).0;
                            self.color_analyzer.set_color(egui::Color32::from_rgba_unmultiplied(pixel[0], pixel[1], pixel[2], pixel[3]));
                        }
                        res
                    },
                    Err(error) => {
                        CenterContainer::new(ui.available_size()).inner_layout(egui::Layout::top_down(egui::Align::Center)).ui(ui, |ui| {
                            ui.spacing_mut().button_padding = egui::vec2(15.0, 8.0);
                            ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
                            ui.label(egui::RichText::new(format!("Couldn't load image: {error}")).text_style(self.theme.heading2()));
                            if ui.add(Button::new(egui::RichText::new("Open an image").text_style(self.theme.heading3()))).clicked() {
                                let start_dir = self.image_directory.as_ref().and_then(|directory| Some(directory.current_directory_path()));
                                let formats = ImageFormat::iterator().flat_map(|format| format.extensions_str());
                                self.file_dialog = Some(FileDialog::new(frame).title("Choose an image").directory(start_dir).add_filter("Image Formats", &formats.collect::<Vec<&&str>>()).pick_file(ui.ctx()));
                            }
                            ui.label(egui::RichText::new("or").text_style(self.theme.heading3()));
                            ui.label(egui::RichText::new("drag an image to the window.").text_style(self.theme.heading3()));
    
                        }).response
                    }
                };
                
    
                self.handle_drop_files(ui, res.rect);
            });
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
            let name = self.image_directory.as_ref().expect("The path info should be valid if the image is").image_name();
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

            ui.horizontal(|ui| {
                ui.add(toggle(&mut self.settings.experimental_features));
                ui.label("Enable experimental features");
            });
            if self.settings.experimental_features {
                ui.label(egui::RichText::new("*The experimental features aren't done and might have some bugs in them. Be careful.").color(ui.visuals().warn_fg_color).text_style(egui::TextStyle::Small));
                self.color_analyzer.open = None;
            }
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
            if self.settings.experimental_features {
                key_bind_widget(ui, "Pick color", &mut self.settings.key_binds.pick_color, default_key_binds.pick_color);
            }
        });
    }
}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(handle) = self.file_dialog.as_ref() {
            if let Some(path) = handle.file_picked() {
                self.image_directory = Some(ImageDirectory::new(&path).expect("Unable to initialize the image directory."));
                self.load_image(&path);
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(mut directory) = self.image_directory.take() {
                if directory.check_for_changes() {
                    self.load_image(directory.current_image_path());
                }
                self.image_directory = Some(directory);
            }
            if self.settings_screen {
                self.settings_screen(ui);
            }
            else {
                self.main_screen(ui, frame);
            }
        });
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


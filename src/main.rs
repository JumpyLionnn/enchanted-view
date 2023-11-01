#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{path::PathBuf, io, fs};

use color_analyzer::ColorAnalyzer;
use egui::{Layout, TextureFilter, TextureOptions};
use file_dialog::{FileDialogHandle, FileDialog};
use image::{DynamicImage, ImageResult, ImageError, ImageFormat, GenericImageView};
mod widgets;
mod egui_extensions;
mod pan_zoom_image;
mod theme;
mod image_directory;
mod settings;
mod file_dialog;
mod checkerboard_pattern;
mod color_name;
mod key_value_match;
mod color_analyzer;
mod utilities;
use image_directory::{ImageDirectory, ImageFormatEx, is_image_file};
use settings::Settings;
use egui_extensions::{ContextEx, ImageLoadResult};
use pan_zoom_image::PanZoomImage;
use theme::Theme;
use utilities::{format_bytes, format_path};
use widgets::{CenterContainer, Button, ImageButton, close_button, DropDownMenu, ComboBox, PathPickerState, PathPicker};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
        min_window_size: Some(egui::vec2(300.0, 200.0)),
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

#[derive(Default)]
struct ImageMetadata {
    file_size: u128,
    date_created: Option<chrono::DateTime<chrono::offset::Local>>,
    date_modified: Option<chrono::DateTime<chrono::offset::Local>>,
    date_accessed: Option<chrono::DateTime<chrono::offset::Local>>,
    read_only: Option<bool>,
    file_format: Option<image::ImageFormat>,
    inner_format: Option<image::ImageFormat>
}

impl ImageMetadata {
    fn image_format_display(&self) -> String {
        self.inner_format
            .or(self.file_format)
            .and_then(|format| Some(format!("{format:?}").to_uppercase()))
            .unwrap_or_else(|| String::from("--"))
    }

    fn image_format(&self) -> Option<image::ImageFormat> {
        self.inner_format
            .or(self.file_format)
    }

    fn image_format_different(&self) -> bool {
        if let (Some(inner_format), Some(file_format)) = (self.inner_format.as_ref(), self.file_format.as_ref()) {
            inner_format != file_format
        } else { false }
    }

    fn inner_format_display(&self) -> String {
        self.inner_format.as_ref().and_then(|format| {
            let mut inner_format_str = format!("{format:?}");
            inner_format_str.make_ascii_uppercase();
            Some(inner_format_str)
        }).unwrap_or_else(|| String::from("--"))
    }

    fn file_format_display(&self) -> String {
        self.file_format.as_ref().and_then(|format| {
            let mut file_format_str = format!("{format:?}");
            file_format_str.make_ascii_uppercase();
            Some(file_format_str)
        }).unwrap_or_else(|| String::from("--"))
    }

    fn format_ext(&self) -> Option<&str> {
        self.inner_format.and_then(|format| {
            Some(format.extensions_str()[0])
        })
    }
}
struct OpenedImage {
    image: DynamicImage,
    display: PanZoomImage,
    metadata: ImageMetadata
}

struct ImageInfoState {
    rename: Option<String>,
    selected_format: image::ImageFormat,
    format_ext: String,
    path_picker_state: PathPickerState,
    target_path: PathBuf,
    target_name: String
}

struct ErrorWindow {
    title: String,
    description: String
}

struct EnchantedView {
    image: Result<OpenedImage, Option<String>>,
    image_directory: Option<ImageDirectory>,
    flip_horizontal: bool,
    flip_vertical: bool,
    rotation: usize,
    context: egui::Context,
    theme: Theme,
    settings_screen: bool,
    settings: Settings,
    file_dialog: Option<FileDialogHandle>,
    color_analyzer: ColorAnalyzer,
    image_info_panel: Option<ImageInfoState>,
    error: Option<ErrorWindow>
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
            None => (Err(None), None),
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
            color_analyzer: ColorAnalyzer::new(),
            image_info_panel: None,
            error: None
        }
    }
    
    fn load_image(&mut self, path: &PathBuf) {
        let load = self.context.load_texture_file(path, TextureOptions {
            magnification: self.settings.image_filtering,
            minification: TextureFilter::Linear,
        });
        if let Some(info) = self.image_info_panel.as_mut() {
            info.rename = None;
        }
        self.image = image_or_error(load, path, &self.theme);
    }

    fn load_image_raw(&mut self, bytes: &[u8], path: &PathBuf) {
        let load = self.context.load_texture_raw(path, bytes, TextureOptions {
            magnification: self.settings.image_filtering,
            minification: TextureFilter::Linear,
        });
        if let Some(info) = self.image_info_panel.as_mut() {
            info.rename = None;
        }
        self.image = image_or_error(load, path, &self.theme);
    }

    fn reload_texture(&mut self) {
        if let Ok(image) = &mut self.image {
            let name = self.image_directory.as_ref().expect("The path info should be valid if the image is").image_name();
            let options = TextureOptions { magnification: self.settings.image_filtering, minification: TextureFilter::Linear };
            let handle = self.context.load_texture_from_image(&image.image, options, name);
            image.display.texture_handle = handle;
        }
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

    fn hotkeys(&mut self, ctx: &egui::Context) {
        if ctx.output(|output| output.text_cursor_pos.is_none()) {
            if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.next_image)) {
                self.next_image();
            }
            if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.previous_image)) {
                self.previous_image();
            }
            if let Ok(image) = &mut self.image {
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_in)) {
                    image.display.zoom_in();
                }
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_out)) {
                    image.display.zoom_out();
                }
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_to_fit)) {
                    image.display.zoom_to_fit();
                }
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.zoom_to_original)) {
                    image.display.zoom_to_original();
                }
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.rotate)) {
                    self.rotation = (self.rotation + 1) % 4;
                }
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.flip_horizontal)) {
                    self.flip_horizontal = !self.flip_horizontal;
                }
                if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.flip_vertical)) {
                    self.flip_vertical = !self.flip_vertical;
                }
                if self.settings.experimental_features {
                    if ctx.input_mut(|input| input.consume_shortcut(&self.settings.key_binds.pick_color)) {
                        self.color_analyzer.toggle_color_picker();
                    }
                }
            }
        }
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

    fn check_for_image_updates(&mut self) {
        if let Some(handle) = self.file_dialog.as_ref() {
            if let Some(path) = handle.file_picked() {
                self.image_directory = Some(ImageDirectory::new(&path).expect("Unable to initialize the image directory."));
                self.load_image(&path);
            }
        }
        if let Some(mut directory) = self.image_directory.take() {
            if let Some(change) = directory.check_for_changes() {
                match change {
                    image_directory::Change::NewImage => {
                        self.load_image(directory.current_image_path());
                    },
                    image_directory::Change::Rename => {
                        if let Ok(image) = self.image.as_mut() {
                            image.metadata.file_format = image::ImageFormat::from_path(directory.current_image_path()).ok();
                        }
                    },
                }
            }
            self.image_directory = Some(directory);
        }
    }
}

// ui implementation
impl EnchantedView {
    fn main_screen(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        let previous_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing.y = 0.0;
        self.toolbar(ui);
        ui.separator(); 
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            self.bottom_bar(ui);  
            ui.separator(); 
            ui.spacing_mut().item_spacing = previous_spacing;
            egui::SidePanel::left("image_info")
                .resizable(true)
                .show_separator_line(true)
                .width_range(220.0..=400.0)
                .default_width(250.0)
                .show_animated_inside(ui, self.image_info_panel.is_some() && self.image.is_ok(), |ui| {
                    self.image_info_panel(ui, frame);
                });
            egui::SidePanel::right("color_analyzer")
                .resizable(true)
                .show_separator_line(true)
                .width_range(200.0..=400.0)
                .default_width(250.0)
                .show_animated_inside(ui, self.color_analyzer.is_open(), |ui| {
                    self.color_analyzer.ui(ui, self.image.is_ok(), &self.theme, &self.settings);
                });
            egui::CentralPanel::default().frame(egui::Frame::central_panel(ui.style()).inner_margin(0.0)).show_inside(ui, |ui| {
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
                            if let Some(error) = error {
                                ui.label(egui::RichText::new(format!("Couldn't load image: {error}")).text_style(self.theme.heading2()));
                            }
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
        self.hotkeys(ui.ctx());
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
                self.image_info_control(ui);
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

    fn image_info_control(&mut self, ui: &mut egui::Ui) {
        let image_info_button = ImageButton::new(egui::include_image!("../assets/info.png"))
            .tint(self.theme.image_button().color)
            .disabled_tint(self.theme.image_button().disabled_color)
            .enabled(self.image.is_ok())
            .selected(self.image_info_panel.is_some())
            .tooltip("Image Info");
        let res = image_info_button.ui(ui);
        if res.clicked() {
            if let Some(_) = self.image_info_panel {
                self.image_info_panel = None;
            }
            else {
                let current_format = self.image.as_ref().ok().and_then(|image| image.metadata.inner_format);
                let format = image::ImageFormat::iterator().find_map(|format| {
                    if format.can_write() && current_format.is_some_and(|f| f != *format) || current_format.is_none() {
                        Some(*format)
                    } else { None }
                }).expect("There should be atleast one format");
                self.image_info_panel = Some(ImageInfoState { 
                    rename: None, 
                    selected_format: format,
                    format_ext: format.extensions_str()[0].to_owned(),
                    path_picker_state: PathPickerState::default(),
                    target_path: self.image_directory.as_ref().and_then(|directory| Some(directory.current_directory_path())).unwrap_or(PathBuf::default()),
                    target_name: self.image_directory.as_ref().and_then(|directory| Some(directory.image_name_stem())).unwrap_or_default().to_owned()
                });
            }
        }
        if self.image.as_ref().is_ok_and(|image| image.metadata.image_format_different()) {  
            let radius = ui.spacing().icon_width_inner / 2.0;
            let center = res.rect.left_top() + egui::Vec2::splat(radius + ui.spacing().icon_spacing);
            ui.painter().circle_filled(center, radius, ui.visuals().warn_fg_color);
        }
    }

    fn image_info_panel(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), self.theme.heading3().resolve(ui.style()).size), egui::Layout::left_to_right(egui::Align::Min), |ui|{
            ui.label(egui::RichText::new("Image Info").text_style(self.theme.heading3()));
            ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if close_button(ui).clicked() {
                    self.image_info_panel = None;
                }
            });
        });

        ui.add_space(ui.spacing().item_spacing.y);
        self.image_info_name(ui);        
        ui.add_space(ui.spacing().item_spacing.y);

        if let Ok(image) = self.image.as_ref() {
            egui::Grid::new("file_info_grid")
                .num_columns(2)
                .striped(true)
                .spacing(ui.spacing().item_spacing * egui::vec2(2.0, 1.5))
                .show(ui, |ui| {
                    ui.label("Dimensions");
                    ui.label(format!("{}x{}", image.image.width(), image.image.height()));
                    ui.end_row();

                    ui.label("size");
                    ui.label(format_bytes(image.metadata.file_size));
                    ui.end_row();

                    ui.label("Created");
                    if let Some(date) = image.metadata.date_created {
                        ui.label(format!("{}", date.format("%x %X")));
                    }
                    else {
                        ui.label("--");
                    }
                    ui.end_row();

                    ui.label("Last modified");
                    if let Some(date) = image.metadata.date_modified {
                        ui.label(format!("{}", date.format("%x %X")));
                    }
                    else {
                        ui.label("--");
                    }
                    ui.end_row();

                    ui.label("Last accessed");
                    if let Some(date) = image.metadata.date_accessed {
                        ui.label(format!("{}", date.format("%x %X")));
                    }
                    else {
                        ui.label("--");
                    }
                    ui.end_row();

                    ui.label("Permissions");
                    if let Some(read_only) = image.metadata.read_only {
                        if read_only {
                            ui.label("Readonly");
                        }
                        else {
                            ui.label("All");
                        }
                    }
                    else {
                        ui.label("--");
                    }
                    ui.end_row();

                    ui.label("Folder path");
                    if let Some(directory) = self.image_directory.as_ref() {
                        let path = directory.current_directory_path();
                        let res = ui.add(egui::Label::new(egui::RichText::new(format_path(&path)).color(ui.visuals().hyperlink_color)).truncate(true))
                            .interact(egui::Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if res.clicked() {
                            if let Err(error) = open::that_detached(path) {
                                self.error = Some(ErrorWindow { 
                                    title: String::from("File explorer open"), 
                                    description: format!("Failed to open the file explorer\n{}", error.to_string())
                                });
                            }
                        }
                    }
                    else {
                        ui.label("--");
                    }
                    ui.end_row();

                    ui.label("Format");
                    ui.label(image.metadata.image_format_display());
                    ui.end_row();
                });
            if image.metadata.image_format_different() {
                let text = format!("Image formats don't match. File extension is {} but the image format is {}.", image.metadata.file_format_display(), image.metadata.inner_format_display());
                ui.label(egui::RichText::new(text).color(ui.visuals().warn_fg_color));
                if ui.button("Change format").clicked() {
                    if let Some(directory) = self.image_directory.as_ref() {
                        let path = directory.current_image_path();
                        let mut new_path = path.clone();
                        new_path.set_extension(image.metadata.format_ext().expect("The ext must exist."));
                        if let Err(error) = fs::rename(path, new_path) {
                            let from_file_name = path.file_name().expect("The file name doesn't exist").to_string_lossy();
                            let to_file_name = path.file_name().expect("The file name doesn't exist").to_string_lossy();
                            self.error = Some(ErrorWindow { 
                                title: String::from("Rename Failed"), 
                                description: format!("Got an error while trying to rename '{}' to '{}'.\n{}", from_file_name, to_file_name, error.to_string())
                            });
                        }
                    }
                    else {
                        eprintln!("Image directory does not exist.");
                        self.error = Some(ErrorWindow { 
                            title: String::from("Internal Error"), 
                            description: String::from("Image directory does not exist.")
                        });
                    }
                }
            }
        }
        ui.add_space(100.0);
        self.image_info_conversion(ui, frame);
    }

    fn image_info_name(&mut self, ui: &mut egui::Ui) {
        ui.label("File name");
        let mut name = self.image_info_panel.as_ref().and_then(|panel| panel.rename.clone())
            .unwrap_or_else(|| {
                self.image_directory.as_ref()
                    .and_then(|directory| Some(directory.image_name_stem().to_owned()))
                    .unwrap_or(String::from("--"))
            });
        ui.visuals_mut().widgets.inactive.bg_stroke = ui.visuals_mut().widgets.hovered.bg_stroke;
        let res = ui.add_enabled(self.image_directory.is_some(), egui::TextEdit::singleline(&mut name).margin(egui::vec2(8.0, 8.0)).min_size(egui::vec2(ui.available_width(), 2.0)));
        if res.has_focus() {
            if let Some(panel) = self.image_info_panel.as_mut() {
                panel.rename = Some(name.clone());
            }
        }
        let mut submit = res.lost_focus();
        if res.has_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {
            res.surrender_focus();
            submit = true;
        }
        if submit {
            if let Some(panel) = self.image_info_panel.as_mut() {
                panel.rename = None;
            }
        }
        if submit && ui.input(|input| !input.key_pressed(egui::Key::Escape)) {
            if let Some(directory) = self.image_directory.as_ref() {
                let new_name = if let Some(ext) = directory.image_ext() {
                    format!("{name}.{ext}")
                } else { name };
                let mut path = directory.current_image_path().to_owned();
                path.set_file_name(&new_name);
                if let Err(error) = fs::rename(directory.current_image_path(), path) {
                    self.error = Some(ErrorWindow { 
                        title: String::from("Rename Failed"), 
                        description: format!("Got an error while trying to rename '{}' to '{}'.\n{}", directory.image_name(), new_name, error.to_string())
                    });
                }
            }
        }
    }

    fn image_info_conversion(&mut self, ui: &mut egui::Ui, frame: &eframe::Frame) {
        ui.allocate_ui_with_layout( ui.available_size(), egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label("Format Conversion");
            if let Ok(image) = self.image.as_ref() {
                ui.label(image.metadata.inner_format_display());
            }
            let avail_rect = ui.available_rect_before_wrap();
            let origin = avail_rect.center_top();
            let arrow_size = 25.0;
            let rect = egui::Rect::from_min_size(avail_rect.left_top(), egui::vec2(ui.available_width(), arrow_size));
            ui.painter().arrow(origin, egui::vec2(0.0, arrow_size), egui::Stroke::new(2.0, ui.visuals().widgets.inactive.fg_stroke.color));
            ui.advance_cursor_after_rect(rect);

            self.image_info_format_selection(ui);
            if let Some(panel) = self.image_info_panel.as_mut() {
                ui.add(PathPicker::new(&mut panel.target_path, frame, &mut panel.path_picker_state));
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut panel.target_name);
                });
            }
            if ui.button("Convert").clicked() {
                if let (Ok(image), Some(panel)) = (self.image.as_ref(), self.image_info_panel.as_ref()) {
                    let filename = format!("{}.{}", panel.target_name, panel.format_ext);
                    let mut path = panel.target_path.clone();
                    path.extend([filename].iter());
                    if let Err(error) = image.image.save_with_format(path, panel.selected_format) {
                        self.error = Some(ErrorWindow { 
                            title: String::from("Image conversion failed"), 
                            description: error.to_string() 
                        });
                    }
                }
                else {
                    self.error = Some(ErrorWindow { 
                        title: String::from("Internal Error"), 
                        description: String::from("Couldn't convert image, image is err") 
                    });
                    eprintln!("Couldn't convert image, image is err");
                }
            }    
        });    
    }

    fn image_info_format_selection(&mut self, ui: &mut egui::Ui) {
        let selected_text = self.image_info_panel.as_ref()
            .and_then(|info| Some(format!("{:?}", info.selected_format).to_uppercase()))
            .unwrap_or_else(|| String::from("--"));
        ComboBox::from_id_source("target_format_selection")
            .width(150.0)
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                let formats = image::ImageFormat::iterator()
                    .filter_map(|format| {
                        if format.can_write() {
                            Some(format.extensions_str().iter().map(|ext| (format.clone(), *ext)))
                            // Some((*format, format!("{format:?}").to_uppercase()))
                        } else { None }
                    })
                    .flatten()
                    .collect::<Vec<(image::ImageFormat, &str)>>();
                let id = ui.auto_id_with("search");
                let mut text = ui.data(|data| data.get_temp::<String>(id).unwrap_or_default());
                ui.visuals_mut().widgets.inactive.bg_stroke = ui.visuals().widgets.hovered.bg_stroke;
                let res = ui.add(egui::TextEdit::singleline(&mut text).hint_text("Search..."));
                let search = text.to_lowercase();
                let empty = text.is_empty();
                if res.lost_focus() {
                    ui.data_mut(|data| data.remove::<String>(id));
                }
                else {
                    ui.data_mut(|data| data.insert_temp(id, text));
                }
                egui::ScrollArea::vertical()
                    .max_height(ui.spacing().combo_height - res.rect.height())
                    .show(ui, |ui| {
                        for (format, ext) in formats {
                            let search_ext = ext.to_lowercase();
                            let show = empty || (search_ext.contains(&search) || search.contains(&search_ext));
                            if show {
                                let enabled = self.image.as_ref().is_ok_and(|image| image.metadata.image_format() != Some(format)) || self.image.is_err();
                                if ui.add_enabled(enabled, Button::new(ext)).clicked() {
                                    if let Some(state) = self.image_info_panel.as_mut() {
                                        state.selected_format = format;
                                        state.format_ext = ext.to_string();
                                    }
                                }
                            }
                        }
                        ((), res.has_focus())
                    }).inner
    });
    }

    fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        let bottom_bar_height = 30.0;
        ui.add_space(5.0);
        ui.allocate_ui_with_layout(egui::vec2(ui.available_width(), bottom_bar_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.theme.style_image_button(ui);
            ui.add_space(5.0);
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
                ui.add_space(5.0);
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

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        let toolbar_size = egui::vec2(ui.available_width(), 30.0);
        ui.allocate_ui_with_layout(toolbar_size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
            self.theme.style_image_button(ui);
            ui.add_space(5.0);
            let back_button = ImageButton::new(egui::include_image!("../assets/back_arrow.png"))
                .tint(self.theme.image_button().color);
            if back_button.ui(ui).clicked() {
                self.settings_screen = false;
            }
            ui.heading("Settings");
        });
        if let Some(update) = self.settings.show(ui, &self.theme) {
            match update {
                settings::SettingsUpdate::ReloadTheme => self.update_theme(),
                settings::SettingsUpdate::ReloadTexture => self.reload_texture(),
                settings::SettingsUpdate::CloseColorAnalyzer => self.color_analyzer.open = None,
            }
        }
    }

    fn error_popup(&mut self, ctx: &egui::Context) {
        if let Some(error_window) = self.error.as_ref() {
            let mut open = true;
            let center = ctx.screen_rect().center();
            let res = egui::Window::new(egui::RichText::new(&error_window.title).color(self.theme.visuals().error_fg_color))
                .id(egui::Id::new("error_window"))
                .constrain(true)
                .collapsible(false)
                .open(&mut open)
                .resizable(false)
                .fixed_pos(center)
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(&error_window.description).text_style(self.theme.heading3()).color(ui.visuals().error_fg_color));
                    ui.allocate_ui_with_layout(egui::vec2(ui.min_size().x, ui.available_height()), egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.style_mut().spacing.button_padding = (24.0, 4.0).into();
                        if ui.button("ok").clicked() {
                            true
                        } else { false }
                    }).inner
                });
            if !open || res.and_then(|res| res.inner).unwrap_or(false) {
                self.error = None;
            }
        }
    }
}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.check_for_image_updates();
        egui::CentralPanel::default().frame(egui::Frame::central_panel(&*ctx.style()).inner_margin(0.0)).show(ctx, |ui| {
            ui.set_enabled(self.error.is_none());
            
            if self.settings_screen {
                self.settings_screen(ui);
            }
            else {
                self.main_screen(ui, frame);
            }
        });

        self.error_popup(ctx);
    }
}

fn load_metadata(path: &PathBuf, image_load_result: &ImageLoadResult) -> ImageMetadata {
    if let Ok(metadata) = fs::metadata(path) {
        ImageMetadata {
            file_size: metadata.len() as u128,
            date_created: metadata.created().ok().and_then(|date| Some(date.into())),
            date_modified: metadata.modified().ok().and_then(|date| Some(date.into())),
            date_accessed: metadata.accessed().ok().and_then(|date| Some(date.into())),
            read_only: Some(metadata.permissions().readonly()),
            file_format: image_load_result.file_format,
            inner_format: image_load_result.inner_format
        }
    }
    else {
        Default::default()
    }
}

fn image_or_error(res: ImageResult<ImageLoadResult>, path: &PathBuf, theme: &Theme) -> Result<OpenedImage, Option<String>> {
    match res {
        Ok(image_load_result) => {
            let image_size = egui::vec2(image_load_result.image.width() as f32, image_load_result.image.height() as f32);
            let metadata = load_metadata(path, &image_load_result);
            Ok(OpenedImage {
                image: image_load_result.image, 
                display: PanZoomImage::new(
                    true, 
                    true, 
                    image_load_result.handle, 
                    image_size, 
                    theme.checkerboard_pattern_colors()
                ),
                metadata
            })
        },
        Err(error) => {
            match error {
                ImageError::IoError(error) => {
                    match error.kind() {
                        io::ErrorKind::NotFound => {
                            Err(Some("The system could not find the file specified.".to_string()))
                        },
                        io::ErrorKind::PermissionDenied => {
                            if let Ok(metadata) = fs::metadata(path) {
                                // On windows this error is returned if the path is a directory
                                if metadata.is_dir() {
                                    Err(Some("Expected a file path, but a directory was provided.".to_string()))
                                }
                                else {
                                    Err(Some("Access is denied.".to_string()))
                                }
                            }
                            else {
                                Err(Some(error.to_string()))
                            }
                        }
                        _ => Err(Some(error.to_string()))
                    }
                }
                other => Err(Some(other.to_string()))
            }
        },
    }
}


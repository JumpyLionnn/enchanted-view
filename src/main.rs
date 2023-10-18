use egui::{Layout, TextureFilter, TextureOptions};
use image::{DynamicImage, EncodableLayout};
mod drop_down_menu;
mod egui_extensions;
mod image_button;
mod pan_zoom_image;
use drop_down_menu::DropDownMenu;
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
    image: DynamicImage,
    image_display: PanZoomImage,
    flip_horizontal: bool,
    flip_vertical: bool,
    rotation: usize
}

impl EnchantedView {
    fn new(context: &egui::Context) -> Self {
        context.style_mut(|style| {
            style.interaction.tooltip_delay = 0.5;
        });
        let image = image::open("assets/crate.png").unwrap();
        let image = image::open("assets/sample_image.jpg").unwrap();
        let image = image::open("assets/jumpylion.png").unwrap();
        let rgba_image = image.to_rgba8();
        let texture_image = egui::ColorImage::from_rgba_unmultiplied(
            [rgba_image.width() as usize, rgba_image.height() as usize],
            rgba_image.as_bytes(),
        );
        let handle = context.load_texture(
            "main_image",
            texture_image,
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Linear,
            },
        );
        let image_size = egui::vec2(image.width() as f32, image.height() as f32);
        Self {
            image: image,
            image_display: PanZoomImage::new(true, true, handle, image_size),
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
            .enabled(self.image_display.can_zoom_in())
            .tooltip("Zoom in");
        if zoom_in_button.ui(ui).clicked() {
            self.image_display.zoom_in();
        }

        ui.allocate_ui_with_layout(
            egui::vec2(60.0, ui.available_height()),
            Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                let text = format!("{:.2}%", self.image_display.scale * 100.0);
                DropDownMenu::new(text, "zoom_display")
                    .width(ui.available_width())
                    .menu_width(120.0)
                    .ui(ui, |ui| {
                        if ui.button("View actual size").clicked() {
                            self.image_display.zoom_to_original();
                        }
                        if ui.button("Zoom to fit").clicked() {
                            self.image_display.zoom_to_fit();
                        }
                    });
            },
        );

        let zoom_out_button = ImageButton::new(egui::include_image!("../assets/zoom_out.png"))
            .tint(egui::Color32::BLACK)
            .disabled_tint(egui::Color32::DARK_GRAY)
            .enabled(self.image_display.can_zoom_out())
            .tooltip("Zoom out");
        if zoom_out_button.ui(ui).clicked() {
            self.image_display.zoom_out();
        }
    }

    fn flip_control(&mut self, ui: &mut egui::Ui) {
        // Its not working, waiting for #3481 in egui for it to work
        let flip_horizontal_button = ImageButton::new(egui::include_image!("../assets/flip_horizontal.png"))
            .tint(egui::Color32::BLACK)
            .selected(self.flip_horizontal)
            .tooltip("Flip horizontal");
        if flip_horizontal_button.ui(ui).clicked() {
            self.flip_horizontal = !self.flip_horizontal;
        }
        let flip_vertical_button = ImageButton::new(egui::include_image!("../assets/flip_vertical.png"))
            .tint(egui::Color32::BLACK)
            .selected(self.flip_vertical)
            .tooltip("Flip vertical");
        if flip_vertical_button.ui(ui).clicked() {
            self.flip_vertical = !self.flip_vertical;
        }
    }

    fn rotate_control(&mut self, ui: &mut egui::Ui) {
        let rotate_button = ImageButton::new(egui::include_image!("../assets/rotate.png"))
            .tint(egui::Color32::BLACK)
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
            self.image_display.update(ui, self.flip_horizontal, self.flip_vertical, self.rotation);
        });
    }
}

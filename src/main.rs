use egui::{TextureOptions, TextureFilter};
use image::{DynamicImage, EncodableLayout};
mod pan_zoom_image;
mod egui_extensions;
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
    image_display: PanZoomImage
}

impl EnchantedView {
    fn new(context: &egui::Context) -> Self {
        let image = image::open("assets/sample_image.jpg").unwrap();
        let image = image::open("assets/crate.png").unwrap();
        let rgba_image = image.to_rgba8();
        let texture_image = egui::ColorImage::from_rgba_unmultiplied(
            [rgba_image.width() as usize, rgba_image.height() as usize], 
            rgba_image.as_bytes());
        let handle = context.load_texture("main_image", texture_image, TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Linear,
        });
        Self {
            image: image,
            image_display: PanZoomImage::new(true, true, handle)
        }
    }
}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let image_size = egui::vec2(self.image.width() as f32, self.image.height() as f32);
            self.image_display.update(ui, image_size);
        });
    }
}


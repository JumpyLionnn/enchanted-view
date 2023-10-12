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
            Box::<EnchantedView>::default()
        }),
    )
}

#[derive(Default)]
struct EnchantedView {}

impl eframe::App for EnchantedView {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::new([true, true]).show(ui, |ui| {
                ui.image(egui::include_image!("../assets/sample_image.jfif"));
            });
        });
    }
}
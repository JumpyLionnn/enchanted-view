
pub struct RadioValue<T: PartialEq> {
    text: egui::WidgetText,
    value: T
}

impl<T: PartialEq> RadioValue<T> {
    pub fn new(text: impl Into<egui::WidgetText>, value: T) -> Self {
        Self { text: text.into(), value }
    }
}

pub fn select<T: PartialEq>(ui: &mut egui::Ui, id: impl std::hash::Hash, value: &mut T, radios: Vec<RadioValue<T>>) -> bool {
    let mut res = false;
    ui.indent(egui::Id::new(id), |ui| {
        for radio in radios {
            res = ui.radio_value(value, radio.value, radio.text).changed() || res;
        }
    });
    res
}
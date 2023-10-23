use std::path::PathBuf;

use egui::TextureHandle;
use image::{ImageResult, EncodableLayout, DynamicImage};



pub trait PainterEx {
    fn debug_stroke(&self, rect: egui::Rect);
    fn debug_line(&self, points: [egui::Pos2; 2]);
    fn debug_label(&self, pos: egui::Pos2, text: impl ToString);
}

impl PainterEx for egui::Painter {
    fn debug_stroke(&self, rect: egui::Rect) {
        self.rect_stroke(rect, egui::Rounding::ZERO, egui::Stroke::new(2.0, egui::Color32::GREEN));
    }

    fn debug_line(&self, points: [egui::Pos2; 2]) {
        self.line_segment(points, egui::Stroke::new(4.0, egui::Color32::RED));
    }
    fn debug_label(&self, pos: egui::Pos2, text: impl ToString) {
        // TODO: add a better debug panel capabilities
        self.debug_text(pos, egui::Align2::LEFT_TOP, egui::Color32::WHITE, text);
    }
}

pub trait ContextEx {
    fn load_texture_raw(&self, name: &str, bytes: &[u8], options: egui::TextureOptions) -> ImageResult<(egui::TextureHandle, DynamicImage)>;
    fn load_texture_file(&self, path: &PathBuf, options: egui::TextureOptions) -> ImageResult<(egui::TextureHandle, DynamicImage)>;
    fn load_texture_from_image(&self, image: &DynamicImage, options: egui::TextureOptions, name: impl Into<String>) -> TextureHandle;
    fn delta_time(&self) -> f32;
}

impl ContextEx for egui::Context {
    fn load_texture_raw(&self, name: &str, bytes: &[u8], options: egui::TextureOptions) -> ImageResult<(egui::TextureHandle, DynamicImage)> {
        let image = image::load_from_memory(bytes)?;
        let rgba_image = image.to_rgba8();
        let texture_image = egui::ColorImage::from_rgba_unmultiplied(
            [rgba_image.width() as usize, rgba_image.height() as usize], 
            rgba_image.as_bytes());
        let handle = self.load_texture(name, texture_image, options);
        Ok((handle, image))
    }
    fn load_texture_file(&self, path: &PathBuf, options: egui::TextureOptions) -> ImageResult<(egui::TextureHandle, DynamicImage)> {
        let name = path.to_string_lossy().to_string();
        let image = image::open(path)?;
        let handle = self.load_texture_from_image(&image, options, name);
        Ok((handle, image))
    }

    fn load_texture_from_image(&self, image: &DynamicImage, options: egui::TextureOptions, name: impl Into<String>) -> TextureHandle {
        let color_image = match &image {
            DynamicImage::ImageRgb8(image) => {
                // common case optimization
                egui::ColorImage::from_rgb(
                    [image.width() as usize, image.height() as usize],
                    image.as_bytes(),
                )
            },
            other => {
                let image = other.to_rgba8();
                egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize],
                    image.as_bytes(),
                )
            },
        };
        let handle = self.load_texture(name, color_image, options);
        handle
    }

    fn delta_time(&self) -> f32 {
        self.input(|input| input.stable_dt)
    }
}

pub trait Vec2Ex {
    fn rotate90_around(self, anchor: egui::Vec2, amount: usize) -> Self;
}

// amount x y
// 0      + +
// 1      + -
// 2      - -
// 3      - +
// 4      + +
// 5      + -
// 6      - -
impl Vec2Ex for egui::Vec2 {
    fn rotate90_around(self, anchor: egui::Vec2, amount: usize) -> Self {
        let switch = amount % 2 == 1;
        
        let sign = egui::vec2(
            -(((amount as isize / 2) % 2) * 2 - 1) as f32, 
            -((((amount as isize + 1) / 2) % 2) * 2 - 1) as f32
        );
        
        let mut vec = if switch {
            egui::vec2(self.y, self.x)
        }
        else {
            egui::vec2(self.x, self.y)
        } - anchor;
        vec = vec * sign + anchor;

        vec
    }
}

impl Vec2Ex for egui::Pos2 {
    fn rotate90_around(self, anchor: egui::Vec2, amount: usize) -> Self {
        self.to_vec2().rotate90_around(anchor, amount).to_pos2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rotate90_around() {
        // TODO: test the anchor
        let vec = egui::vec2(10.0, 4.0);
        assert_eq!(vec, vec.rotate90_around(egui::vec2(0.0, 0.0), 0));
        assert_eq!(vec.rot90(), vec.rotate90_around(egui::vec2(0.0, 0.0), 1));
        assert_eq!(vec.rot90().rot90(), vec.rotate90_around(egui::vec2(0.0, 0.0), 2));
        assert_eq!(vec.rot90().rot90().rot90(), vec.rotate90_around(egui::vec2(0.0, 0.0), 3));
        assert_eq!(vec.rot90().rot90().rot90().rot90(), vec.rotate90_around(egui::vec2(0.0, 0.0), 4));
        assert_eq!(vec, vec.rotate90_around(egui::vec2(0.0, 0.0), 4));
    }
}
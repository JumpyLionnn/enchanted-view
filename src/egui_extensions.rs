use std::{path::PathBuf, fs};

use egui::TextureHandle;
use image::{ImageResult, EncodableLayout, DynamicImage};



pub trait PainterEx {
    fn rect_stroke_cropped(&self, rect: egui::Rect, bounds: egui::Rect, stroke: egui::Stroke);
    fn debug_stroke(&self, rect: egui::Rect);
    fn debug_line(&self, points: [egui::Pos2; 2]);
    fn debug_label(&self, pos: egui::Pos2, text: impl ToString);
}

impl PainterEx for egui::Painter {
    fn rect_stroke_cropped(&self, rect: egui::Rect, bounds: egui::Rect, stroke: egui::Stroke) {
        let stroke_rect = rect.expand(stroke.width / 2.0);
        let top_stroke_width = rect.top() - (rect.top() - stroke.width).max(bounds.top());
        if top_stroke_width > 0.0 {
            self.line_segment([bounds.clamp(stroke_rect.left_top()), bounds.clamp(stroke_rect.right_top())], egui::Stroke::new(top_stroke_width, stroke.color));
        }

        let right_stroke_width = (stroke.width + rect.right()).min(bounds.right()) - rect.right();
        if right_stroke_width > 0.0 {
            self.line_segment([bounds.clamp(stroke_rect.right_top()), bounds.clamp(stroke_rect.right_bottom())], egui::Stroke::new(right_stroke_width, stroke.color));
        }

        let bottom_stroke_width = (stroke.width + rect.bottom()).min(bounds.bottom()) - rect.bottom();
        if bottom_stroke_width > 0.0 {
            self.line_segment([bounds.clamp(stroke_rect.right_bottom()), bounds.clamp(stroke_rect.left_bottom())], egui::Stroke::new(bottom_stroke_width, stroke.color));
        }

        let left_stroke_width =  rect.left() - (rect.left() - stroke.width).max(bounds.left());
        if left_stroke_width > 0.0 {
            self.line_segment([bounds.clamp(stroke_rect.left_bottom()), bounds.clamp(stroke_rect.left_top())], egui::Stroke::new(left_stroke_width, stroke.color));
        }
    }


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

pub struct ImageLoadResult {
    pub handle: egui::TextureHandle,
    pub image: image::DynamicImage,
    pub inner_format: Option<image::ImageFormat>,
    pub file_format: Option<image::ImageFormat>
}

pub trait ContextEx {
    fn rect_contains_pointer(&self, layer_id: egui::LayerId, rect: egui::Rect) -> bool;
    fn load_texture_file(&self, path: &PathBuf, options: egui::TextureOptions) -> ImageResult<ImageLoadResult>;
    fn load_texture_raw(&self, path: &PathBuf, bytes: &[u8], options: egui::TextureOptions) -> ImageResult<ImageLoadResult>;
    fn load_texture_raw_with_format_hint(&self, name: &str, bytes: &[u8], options: egui::TextureOptions, format_hint: ImageResult<image::ImageFormat>) -> ImageResult<ImageLoadResult>;        
    fn load_texture_from_image(&self, image: &DynamicImage, options: egui::TextureOptions, name: impl Into<String>) -> TextureHandle;
    fn delta_time(&self) -> f32;
}

impl ContextEx for egui::Context {

    fn rect_contains_pointer(&self, layer_id: egui::LayerId, rect: egui::Rect) -> bool {
        rect.is_positive() && {
            let pointer_pos = self.input(|i| i.pointer.interact_pos());
            if let Some(pointer_pos) = pointer_pos {
                rect.contains(pointer_pos) && self.layer_id_at(pointer_pos) == Some(layer_id)
            } else {
                false
            }
        }
    }

    fn load_texture_file(&self, path: &PathBuf, options: egui::TextureOptions) -> ImageResult<ImageLoadResult> {
        let name = path.to_string_lossy();
        let file_image_format = image::ImageFormat::from_path(path);
        let bytes = fs::read(path).map_err(image::ImageError::IoError)?;
        self.load_texture_raw_with_format_hint(&name,  &bytes, options, file_image_format)
    }

    fn load_texture_raw(&self, path: &PathBuf, bytes: &[u8], options: egui::TextureOptions) -> ImageResult<ImageLoadResult> {
        let name = path.to_string_lossy();
        let file_image_format = image::ImageFormat::from_path(path);
        self.load_texture_raw_with_format_hint(&name, bytes, options, file_image_format)
    }
    fn load_texture_raw_with_format_hint(&self, name: &str, bytes: &[u8], options: egui::TextureOptions, format_hint: ImageResult<image::ImageFormat>) -> ImageResult<ImageLoadResult> {
        let guessed_format = image::guess_format(bytes);
        let inner_format = guessed_format.as_ref().ok().cloned();
        let file_format_hint = format_hint.as_ref().ok().cloned();
        let format = guessed_format.or(format_hint)?;
        let image = image::load_from_memory_with_format(bytes, format)?;
        let handle = self.load_texture_from_image(&image, options, name);
        Ok(ImageLoadResult { 
            handle, 
            image, 
            inner_format, 
            file_format: file_format_hint
        })
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
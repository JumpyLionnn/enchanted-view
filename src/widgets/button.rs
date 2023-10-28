use egui::NumExt;

/// Clickable button with text.
///
/// See also [`Ui::button`].
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// # fn do_stuff() {}
///
/// if ui.add(egui::Button::new("Click me")).clicked() {
///     do_stuff();
/// }
///
/// // A greyed-out and non-interactive button:
/// if ui.add_enabled(false, egui::Button::new("Can't click this")).clicked() {
///     unreachable!();
/// }
/// # });
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct Button<'a> {
    image: Option<egui::Image<'a>>,
    text: Option<egui::WidgetText>,
    shortcut_text: egui::WidgetText,
    wrap: Option<bool>,

    /// None means default for interact
    fill: Option<egui::Color32>,
    stroke: Option<egui::Stroke>,
    sense: egui::Sense,
    small: bool,
    frame: Option<bool>,
    min_size: egui::Vec2,
    rounding: Option<egui::Rounding>,
    selected: bool,
}

impl<'a> Button<'a> {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self::opt_image_and_text(None, Some(text.into()))
    }

    /// Creates a button with an image. The size of the image as displayed is defined by the provided size.
    #[allow(clippy::needless_pass_by_value)]
    pub fn image(image: impl Into<egui::Image<'a>>) -> Self {
        Self::opt_image_and_text(Some(image.into()), None)
    }

    /// Creates a button with an image to the left of the text. The size of the image as displayed is defined by the provided size.
    #[allow(clippy::needless_pass_by_value)]
    pub fn image_and_text(image: impl Into<egui::Image<'a>>, text: impl Into<egui::WidgetText>) -> Self {
        Self::opt_image_and_text(Some(image.into()), Some(text.into()))
    }

    pub fn opt_image_and_text(image: Option<egui::Image<'a>>, text: Option<egui::WidgetText>) -> Self {
        Self {
            text,
            image,
            shortcut_text: Default::default(),
            wrap: None,
            fill: None,
            stroke: None,
            sense: egui::Sense::click(),
            small: false,
            frame: None,
            min_size: egui::Vec2::ZERO,
            rounding: None,
            selected: false,
        }
    }

    /// If `true`, the text will wrap to stay within the max width of the [`Ui`].
    ///
    /// By default [`Self::wrap`] will be true in vertical layouts
    /// and horizontal layouts with wrapping,
    /// and false on non-wrapping horizontal layouts.
    ///
    /// Note that any `\n` in the text will always produce a new line.
    #[inline]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    /// Override background fill color. Note that this will override any on-hover effects.
    /// Calling this will also turn on the frame.
    pub fn fill(mut self, fill: impl Into<egui::Color32>) -> Self {
        self.fill = Some(fill.into());
        self.frame = Some(true);
        self
    }

    /// Override button stroke. Note that this will override any on-hover effects.
    /// Calling this will also turn on the frame.
    pub fn stroke(mut self, stroke: impl Into<egui::Stroke>) -> Self {
        self.stroke = Some(stroke.into());
        self.frame = Some(true);
        self
    }

    /// Make this a small button, suitable for embedding into text.
    pub fn small(mut self) -> Self {
        if let Some(text) = self.text {
            self.text = Some(text.text_style(egui::TextStyle::Body));
        }
        self.small = true;
        self
    }

    /// Turn off the frame
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = Some(frame);
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    pub fn sense(mut self, sense: egui::Sense) -> Self {
        self.sense = sense;
        self
    }

    /// Set the minimum size of the button.
    pub fn min_size(mut self, min_size: egui::Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set the rounding of the button.
    pub fn rounding(mut self, rounding: impl Into<egui::Rounding>) -> Self {
        self.rounding = Some(rounding.into());
        self
    }

    /// Show some text on the right side of the button, in weak color.
    ///
    /// Designed for menu buttons, for setting a keyboard shortcut text (e.g. `Ctrl+S`).
    ///
    /// The text can be created with [`Context::format_shortcut`].
    pub fn shortcut_text(mut self, shortcut_text: impl Into<egui::WidgetText>) -> Self {
        self.shortcut_text = shortcut_text.into();
        self
    }

    /// If `true`, mark this button as "selected".
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl egui::Widget for Button<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Button {
            text,
            image,
            shortcut_text,
            wrap,
            fill,
            stroke,
            sense,
            small,
            frame,
            min_size,
            rounding,
            selected,
        } = self;

        let frame = frame.unwrap_or_else(|| ui.visuals().button_frame);

        let mut button_padding = if frame {
            ui.spacing().button_padding
        } else {
            egui::Vec2::ZERO
        };
        if small {
            button_padding.y = 0.0;
        }

        let space_available_for_image = if let Some(text) = &text {
            
            let font_height = ui.fonts(|fonts| {
                // WidgetText font height is private so its done manually
                match text {
                    egui::WidgetText::RichText(rich) => rich.font_height(fonts, ui.style()),
                    egui::WidgetText::LayoutJob(job) => job.font_height(fonts),
                    egui::WidgetText::Galley(galley) => {
                        if let Some(row) = galley.rows.first() {
                            row.height()
                        } else {
                            galley.size().y
                        }
                    },
                }
            });
            egui::Vec2::splat(font_height) // Reasonable?
        } else {
            ui.available_size() - 2.0 * button_padding
        };

        let image_size = if let Some(image) = &image {
            image
                .load_and_calc_size(ui, space_available_for_image)
                .unwrap_or(space_available_for_image)
        } else {
            egui::Vec2::ZERO
        };

        let mut text_wrap_width = ui.available_width() - 2.0 * button_padding.x;
        if image.is_some() {
            text_wrap_width -= image_size.x + ui.spacing().icon_spacing;
        }
        let shortcut_text = (!shortcut_text.is_empty())
            .then(|| shortcut_text.into_galley(ui, Some(false), f32::INFINITY, egui::TextStyle::Button));
        if let Some(text) = &shortcut_text {
            text_wrap_width -= text.size().x;
        }
        // if !shortcut_text.is_empty() {
        //     text_wrap_width -= 60.0; // Some space for the shortcut text (which we never wrap).
        // }

        let text = text.map(|text| text.into_galley(ui, wrap, text_wrap_width, egui::TextStyle::Button));

        let mut desired_size = egui::Vec2::ZERO;
        if image.is_some() {
            desired_size.x += image_size.x;
            desired_size.y = desired_size.y.max(image_size.y);
        }
        if image.is_some() && text.is_some() {
            desired_size.x += ui.spacing().icon_spacing;
        }
        if let Some(text) = &text {
            desired_size.x += text.size().x;
            desired_size.y = desired_size.y.max(text.size().y);
        }
        if let Some(shortcut_text) = &shortcut_text {
            desired_size.x += ui.spacing().item_spacing.x + shortcut_text.size().x;
            desired_size.y = desired_size.y.max(shortcut_text.size().y);
        }
        desired_size += 2.0 * button_padding;
        if !small {
            desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        }
        desired_size = desired_size.at_least(min_size);

        let (rect, response) = ui.allocate_at_least(desired_size, sense);
        response.widget_info(|| {
            if let Some(text) = &text {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, text.text())
            } else {
                egui::WidgetInfo::new(egui::WidgetType::Button)
            }
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            let (frame_expansion, frame_rounding, frame_fill, frame_stroke) = if selected {
                let selection = ui.visuals().selection;
                (
                    egui::Vec2::ZERO,
                    egui::Rounding::ZERO,
                    selection.bg_fill,
                    selection.stroke,
                )
            } else if frame {
                let expansion = egui::Vec2::splat(visuals.expansion);
                (
                    expansion,
                    visuals.rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                )
            } else {
                Default::default()
            };
            let frame_rounding = rounding.unwrap_or(frame_rounding);
            let frame_fill = fill.unwrap_or(frame_fill);
            let frame_stroke = stroke.unwrap_or(frame_stroke);
            ui.painter().rect(
                rect.expand2(frame_expansion),
                frame_rounding,
                frame_fill,
                frame_stroke,
            );

            let mut cursor_x = rect.min.x + button_padding.x;

            if let Some(image) = &image {
                let image_rect = egui::Rect::from_min_size(
                    egui::pos2(cursor_x, rect.center().y - 0.5 - (image_size.y / 2.0)),
                    image_size,
                );
                cursor_x += image_size.x;
                // let tlr = image.load_for_size(ui.ctx(), image_size);
                
                // egui::widgets::image::paint_texture_load_result(
                //     ui,
                //     &tlr,
                //     image_rect,
                //     image.show_loading_spinner,
                //     image.image_options(),
                // );
                image.paint_at(ui, image_rect);
                // response =
                //     egui::widgets::image::texture_load_result_response(image.source(), &tlr, response);
            }

            let visuals = ui.style().interact(&response);

            if image.is_some() && text.is_some() {
                cursor_x += ui.spacing().icon_spacing;
            }

            if let Some(text) = text {
                let text_pos = if image.is_some() || shortcut_text.is_some() {
                    egui::pos2(cursor_x, rect.center().y - 0.5 * text.size().y)
                } else {
                    // Make sure button text is centered if within a centered layout
                    ui.layout()
                        .align_size_within_rect(text.size(), rect.shrink2(button_padding))
                        .min
                };
                text.paint_with_visuals(ui.painter(), text_pos, visuals);
            }

            if let Some(shortcut_text) = shortcut_text {
                let shortcut_text_pos = egui::pos2(
                    rect.max.x - button_padding.x - shortcut_text.size().x,
                    rect.center().y - 0.5 * shortcut_text.size().y,
                );
                shortcut_text.paint_with_fallback_color(
                    ui.painter(),
                    shortcut_text_pos,
                    ui.visuals().weak_text_color(),
                );
            }
        }

        if let Some(cursor) = ui.visuals().interact_cursor {
            if response.hovered {
                ui.ctx().set_cursor_icon(cursor);
            }
        }

        response
    }
}


pub fn close_button(ui: &mut egui::Ui) -> egui::Response {
    let button_size = egui::Vec2::splat(ui.spacing().icon_width);

    let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click());
    let visuals = ui.style().interact(&response);
    let mut rect = rect.shrink(2.0).expand(visuals.expansion);
    rect.set_center(egui::pos2(rect.center().x, ui.available_rect_before_wrap().center().y));
    let stroke = visuals.fg_stroke;
    
    ui.painter() // paints \
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter() // paints /
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);
    response
}
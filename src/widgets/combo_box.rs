use egui::NumExt;

/// A function that paints the [`ComboBox`] icon
pub type IconPainter = Box<dyn FnOnce(&egui::Ui, egui::Rect, &egui::style::WidgetVisuals, bool, egui::AboveOrBelow)>;

/// A drop-down selection menu with a descriptive label.
///
/// ```
/// # #[derive(Debug, PartialEq)]
/// # enum Enum { First, Second, Third }
/// # let mut selected = Enum::First;
/// # egui::__run_test_ui(|ui| {
/// egui::ComboBox::from_label("Select one!")
///     .selected_text(format!("{:?}", selected))
///     .show_ui(ui, |ui| {
///         ui.selectable_value(&mut selected, Enum::First, "First");
///         ui.selectable_value(&mut selected, Enum::Second, "Second");
///         ui.selectable_value(&mut selected, Enum::Third, "Third");
///     }
/// );
/// # });
/// ```
#[must_use = "You should call .show*"]
pub struct ComboBox {
    id_source: egui::Id,
    label: Option<egui::WidgetText>,
    selected_text: egui::WidgetText,
    width: Option<f32>,
    icon: Option<IconPainter>,
    wrap_enabled: bool,
}

impl ComboBox {
    /// Create new [`ComboBox`] with id and label
    pub fn new(id_source: impl std::hash::Hash, label: impl Into<egui::WidgetText>) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            label: Some(label.into()),
            selected_text: Default::default(),
            width: None,
            icon: None,
            wrap_enabled: false,
        }
    }

    /// Label shown next to the combo box
    pub fn from_label(label: impl Into<egui::WidgetText>) -> Self {
        let label = label.into();
        Self {
            id_source: egui::Id::new(label.text()),
            label: Some(label),
            selected_text: Default::default(),
            width: None,
            icon: None,
            wrap_enabled: false,
        }
    }

    /// Without label.
    pub fn from_id_source(id_source: impl std::hash::Hash) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            label: Default::default(),
            selected_text: Default::default(),
            width: None,
            icon: None,
            wrap_enabled: false,
        }
    }

    /// Set the outer width of the button and menu.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// What we show as the currently selected value
    pub fn selected_text(mut self, selected_text: impl Into<egui::WidgetText>) -> Self {
        self.selected_text = selected_text.into();
        self
    }

    /// Use the provided function to render a different [`ComboBox`] icon.
    /// Defaults to a triangle that expands when the cursor is hovering over the [`ComboBox`].
    ///
    /// For example:
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let text = "Selected text";
    /// pub fn filled_triangle(
    ///     ui: &egui::Ui,
    ///     rect: egui::Rect,
    ///     visuals: &egui::style::WidgetVisuals,
    ///     _is_open: bool,
    ///     _above_or_below: egui::AboveOrBelow,
    /// ) {
    ///     let rect = egui::Rect::from_center_size(
    ///         rect.center(),
    ///         egui::vec2(rect.width() * 0.6, rect.height() * 0.4),
    ///     );
    ///     ui.painter().add(egui::Shape::convex_polygon(
    ///         vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
    ///         visuals.fg_stroke.color,
    ///         visuals.fg_stroke,
    ///     ));
    /// }
    ///
    /// egui::ComboBox::from_id_source("my-combobox")
    ///     .selected_text(text)
    ///     .icon(filled_triangle)
    ///     .show_ui(ui, |_ui| {});
    /// # });
    /// ```
    pub fn icon(
        mut self,
        icon_fn: impl FnOnce(&egui::Ui, egui::Rect, &egui::style::WidgetVisuals, bool, egui::AboveOrBelow) + 'static,
    ) -> Self {
        self.icon = Some(Box::new(icon_fn));
        self
    }

    /// Controls whether text wrap is used for the selected text
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap_enabled = wrap;
        self
    }

    /// Show the combo box, with the given ui code for the menu contents.
    ///
    /// Returns `InnerResponse { inner: None }` if the combo box is closed.
    pub fn show_ui<R>(
        self,
        ui: &mut egui::Ui,
        menu_contents: impl FnOnce(&mut egui::Ui) -> (R, bool),
    ) -> egui::InnerResponse<Option<R>> {
        self.show_ui_dyn(ui, Box::new(menu_contents))
    }

    fn show_ui_dyn<'c, R>(
        self,
        ui: &mut egui::Ui,
        menu_contents: Box<dyn FnOnce(&mut egui::Ui) -> (R, bool) + 'c>,
    ) -> egui::InnerResponse<Option<R>> {
        let Self {
            id_source,
            label,
            selected_text,
            width,
            icon,
            wrap_enabled,
        } = self;

        let button_id = ui.make_persistent_id(id_source);

        // ui.horizontal(|ui| {
            let mut ir = combo_box_dyn(
                ui,
                button_id,
                selected_text,
                menu_contents,
                icon,
                wrap_enabled,
                width,
            );
            if let Some(label) = label {
                ir.response
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, label.text()));
                ir.response |= ui.label(label);
            } else {
                ir.response
                    .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, ""));
            }
            ir
        // })
        // .inner
    }

    /// Show a list of items with the given selected index.
    ///
    ///
    /// ```
    /// # #[derive(Debug, PartialEq)]
    /// # enum Enum { First, Second, Third }
    /// # let mut selected = Enum::First;
    /// # egui::__run_test_ui(|ui| {
    /// let alternatives = ["a", "b", "c", "d"];
    /// let mut selected = 2;
    /// egui::ComboBox::from_label("Select one!").show_index(
    ///     ui,
    ///     &mut selected,
    ///     alternatives.len(),
    ///     |i| alternatives[i]
    /// );
    /// # });
    /// ```
    pub fn show_index<Text: Into<egui::WidgetText>>(
        self,
        ui: &mut egui::Ui,
        selected: &mut usize,
        len: usize,
        get: impl Fn(usize) -> Text,
    ) -> egui::Response {
        let slf = self.selected_text(get(*selected));

        let mut changed = false;

        let mut response = slf
            .show_ui(ui, |ui| {
                (for i in 0..len {
                    if ui.selectable_label(i == *selected, get(i)).clicked() {
                        *selected = i;
                        changed = true;
                    }
                }, false)
            })
            .response;

        if changed {
            response.mark_changed();
        }
        response
    }
}

fn combo_box_dyn<'c, R>(
    ui: &mut egui::Ui,
    button_id: egui::Id,
    selected_text: egui::WidgetText,
    menu_contents: Box<dyn FnOnce(&mut egui::Ui) -> (R, bool) + 'c>,
    icon: Option<IconPainter>,
    wrap_enabled: bool,
    width: Option<f32>,
) -> egui::InnerResponse<Option<R>> {
    let popup_id = button_id.with("popup");

    let is_popup_open = ui.memory(|m| m.is_popup_open(popup_id));

    let popup_height = ui.memory(|m| m.area_rect(popup_id).map_or(100.0, |state| state.height()));

    let above_or_below =
        if ui.next_widget_position().y + ui.spacing().interact_size.y + popup_height
            < ui.ctx().screen_rect().bottom()
        {
            egui::AboveOrBelow::Below
        } else {
            egui::AboveOrBelow::Above
        };

    let margin = ui.spacing().button_padding;
    let button_response = button_frame(ui, button_id, is_popup_open, egui::Sense::click(), |ui| {
        let icon_spacing = ui.spacing().icon_spacing;
        // We don't want to change width when user selects something new
        let full_minimum_width = if wrap_enabled {
            // Currently selected value's text will be wrapped if needed, so occupy the available width.
            ui.available_width()
        } else {
            // Occupy at least the minimum width assigned to ComboBox.
            let width = width.unwrap_or_else(|| ui.spacing().combo_width);
            width - 2.0 * margin.x
        };
        let icon_size = egui::Vec2::splat(ui.spacing().icon_width);
        let wrap_width = if wrap_enabled {
            // Use the available width, currently selected value's text will be wrapped if exceeds this value.
            ui.available_width() - icon_spacing - icon_size.x
        } else {
            // Use all the width necessary to display the currently selected value's text.
            f32::INFINITY
        };

        let galley =
            selected_text.into_galley(ui, Some(wrap_enabled), wrap_width, egui::TextStyle::Button);

        // The width necessary to contain the whole widget with the currently selected value's text.
        let width = if wrap_enabled {
            full_minimum_width
        } else {
            // Occupy at least the minimum width needed to contain the widget with the currently selected value's text.
            galley.size().x + icon_spacing + icon_size.x
        };

        // Case : wrap_enabled : occupy all the available width.
        // Case : !wrap_enabled : occupy at least the minimum width assigned to Slider and ComboBox,
        // increase if the currently selected value needs additional horizontal space to fully display its text (up to wrap_width (f32::INFINITY)).
        let width = width.at_least(full_minimum_width);
        let height = galley.size().y.max(icon_size.y);

        let (_, rect) = ui.allocate_space(egui::Vec2::new(width, height));
        let button_rect = rect.expand2(ui.spacing().button_padding);
        let response = ui.interact(button_rect, button_id, egui::Sense::click());
        // response.active |= is_popup_open;

        if ui.is_rect_visible(rect) {
            let icon_rect = egui::Align2::RIGHT_CENTER.align_size_within_rect(icon_size, rect);
            let visuals = if is_popup_open {
                &ui.visuals().widgets.open
            } else {
                ui.style().interact(&response)
            };

            if let Some(icon) = icon {
                icon(
                    ui,
                    icon_rect.expand(visuals.expansion),
                    visuals,
                    is_popup_open,
                    above_or_below,
                );
            } else {
                paint_default_icon(
                    ui.painter(),
                    icon_rect.expand(visuals.expansion),
                    visuals,
                    above_or_below,
                );
            }

            let text_rect = egui::Align2::LEFT_CENTER.align_size_within_rect(galley.size(), rect);
            galley.paint_with_visuals(ui.painter(), text_rect.min, visuals);
        }
        button_rect
    });

    if button_response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
    }
    let inner = popup_above_or_below_widget(
        ui,
        popup_id,
        &button_response,
        above_or_below,
        menu_contents
    );

    egui::InnerResponse {
        inner,
        response: button_response,
    }
}

fn button_frame(
    ui: &mut egui::Ui,
    id: egui::Id,
    is_popup_open: bool,
    sense: egui::Sense,
    add_contents: impl FnOnce(&mut egui::Ui) -> egui::Rect,
) -> egui::Response {
    let where_to_put_background = ui.painter().add(egui::Shape::Noop);

    let margin = ui.spacing().button_padding;
    let interact_size = ui.spacing().interact_size;

    let mut outer_rect = ui.available_rect_before_wrap();
    outer_rect.set_height(outer_rect.height().at_least(interact_size.y));
    let inner_rect = outer_rect.shrink2(margin);
    
    let mut content_ui = ui.child_ui(inner_rect, *ui.layout());
    let content_rect = add_contents(&mut content_ui);

    let mut outer_rect = content_rect.expand2(margin);
    outer_rect.set_height(outer_rect.height().at_least(interact_size.y));

    let response = ui.interact(outer_rect, id, sense);

    if ui.is_rect_visible(outer_rect) {
        let visuals = if is_popup_open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };
        ui.painter().set(
            where_to_put_background,
            egui::epaint::RectShape::new(
                outer_rect.expand(visuals.expansion),
                visuals.rounding,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
            ),
        );
    }

    ui.advance_cursor_after_rect(outer_rect);
    response
}

fn paint_default_icon(
    painter: &egui::Painter,
    rect: egui::Rect,
    visuals: &egui::style::WidgetVisuals,
    above_or_below: egui::AboveOrBelow,
) {
    let rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width() * 0.7, rect.height() * 0.45),
    );

    match above_or_below {
        egui::AboveOrBelow::Above => {
            // Upward pointing triangle
            painter.add(egui::Shape::convex_polygon(
                vec![rect.left_bottom(), rect.right_bottom(), rect.center_top()],
                visuals.fg_stroke.color,
                egui::Stroke::NONE,
            ));
        }
        egui::AboveOrBelow::Below => {
            // Downward pointing triangle
            painter.add(egui::Shape::convex_polygon(
                vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
                visuals.fg_stroke.color,
                egui::Stroke::NONE,
            ));
        }
    }
}


/// Shows a popup above or below another widget.
///
/// Useful for drop-down menus (combo boxes) or suggestion menus under text fields.
///
/// The opened popup will have the same width as the parent.
///
/// You must open the popup with [`Memory::open_popup`] or  [`Memory::toggle_popup`].
///
/// Returns `None` if the popup is not open.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// let response = ui.button("Open popup");
/// let popup_id = ui.make_persistent_id("my_unique_id");
/// if response.clicked() {
///     ui.memory_mut(|mem| mem.toggle_popup(popup_id));
/// }
/// let below = egui::AboveOrBelow::Below;
/// egui::popup::popup_above_or_below_widget(ui, popup_id, &response, below, |ui| {
///     ui.set_min_width(200.0); // if you want to control the size
///     ui.label("Some more info, or things you can select:");
///     ui.label("…");
/// });
/// # });
/// ```
pub fn popup_above_or_below_widget<R>(
    ui: &egui::Ui,
    popup_id: egui::Id,
    widget_response: &egui::Response,
    above_or_below: egui::AboveOrBelow,
    add_contents: impl FnOnce(&mut egui::Ui) -> (R, bool),
) -> Option<R> {
    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let (pos, pivot) = match above_or_below {
            egui::AboveOrBelow::Above => (widget_response.rect.left_top(), egui::Align2::LEFT_BOTTOM),
            egui::AboveOrBelow::Below => (widget_response.rect.left_bottom(), egui::Align2::LEFT_TOP),
        };

        let res = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .constrain(true)
            .fixed_pos(pos)
            .pivot(pivot)
            .show(ui.ctx(), |ui| {
                // Note: we use a separate clip-rect for this area, so the popup can be outside the parent.
                // See https://github.com/emilk/egui/issues/825
                let frame = egui::Frame::popup(ui.style());
                let frame_margin = frame.total_margin();
                frame
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.set_width(widget_response.rect.width() - frame_margin.sum().x);
                            add_contents(ui)
                        })
                        .inner
                    })
                    .inner
            });
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) 
            || (widget_response.clicked_elsewhere() && !res.inner.1) {
            ui.memory_mut(|mem| mem.close_popup());
        }
        Some(res.inner.0)
    } else {
        None
    }
}

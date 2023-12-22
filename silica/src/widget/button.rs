use std::{cell::Cell, rc::Rc};

use taffy::{geometry::Point, prelude::*};

use crate::{
    define_widget, model::Model, signal, widget::Label, GraphicsContext, Gui, HorizontalAlign,
    PointerState, Signals, ThemeColor, VerticalAlign, VisualStyle, WidgetData, WidgetObject,
};

pub struct BaseButtonData {
    was_pressed: Cell<bool>,
}

impl BaseButtonData {
    fn new() -> Self {
        BaseButtonData {
            was_pressed: Cell::new(false),
        }
    }
    fn set_pointer_state<F>(&self, state: PointerState, on_activate: F) -> VisualStyle
    where
        F: FnOnce(),
    {
        if state == PointerState::Over && self.was_pressed.get() {
            on_activate();
        }
        self.was_pressed.set(state == PointerState::Press);

        let mut visual = VisualStyle::BUTTON;
        match state {
            PointerState::None => visual.background = Some(ThemeColor::ButtonNormal),
            PointerState::Over => visual.background = Some(ThemeColor::ButtonOver),
            PointerState::Press => visual.background = Some(ThemeColor::ButtonPress),
        }
        visual
    }
}

pub struct ButtonData {
    base: BaseButtonData,
    label: Label,
    toggle: Option<Rc<dyn Model<bool>>>,
    signals: Signals<Button>,
}

impl WidgetObject for ButtonData {
    fn draw(data: &WidgetData<Self>, context: &mut dyn GraphicsContext, size: Size<f32>) {
        let mut border_width = 1.0;
        if let Some(model) = data.object.toggle.as_ref() {
            if model.get() {
                border_width = 3.0;
            }
        }
        context.draw_border(size, Rect::points(border_width));
    }
    fn set_pointer_state(data: Rc<WidgetData<Self>>, state: PointerState) {
        let button = Button(data);
        let visual = button.object.base.set_pointer_state(state, || {
            if let Some(model) = button.object.toggle.as_ref() {
                model.clone().set(!model.get());
            }
            button.object.signals.emit(button.clone(), signal::Activate);
        });
        button.set_visual(Some(visual));
    }
}

define_widget!(Button, ButtonData);

impl Button {
    fn new(gui: Rc<Gui>, label_text: String, toggle: Option<Rc<dyn Model<bool>>>) -> Self {
        let label = Label::with_text(gui.clone(), label_text);
        label.set_halign(HorizontalAlign::Center);
        label.set_valign(VerticalAlign::Center);
        label.set_layout(Style {
            flex_grow: 1.0,
            ..Default::default()
        });

        let style = Style {
            min_size: Size {
                width: Dimension::Points(128.),
                height: Dimension::Points(32.),
            },
            align_items: Some(AlignItems::Stretch),
            justify_items: Some(JustifyItems::Stretch),
            ..Default::default()
        };
        let button = Button(WidgetData::with_style(
            gui,
            style,
            Some(VisualStyle::BUTTON),
            ButtonData {
                base: BaseButtonData::new(),
                label: label.clone(),
                toggle,
                signals: Signals::new(),
            },
        ));
        button.add_child(label);
        button
    }
    pub fn with_label(gui: Rc<Gui>, label_text: String) -> Self {
        Self::new(gui, label_text, None)
    }
    pub fn with_label_toggle(
        gui: Rc<Gui>,
        label_text: String,
        toggle: Rc<dyn Model<bool>>,
    ) -> Self {
        Self::new(gui, label_text, Some(toggle))
    }

    pub fn label(&self) -> Label {
        self.object.label.clone()
    }

    pub fn connect_activate<F>(&self, handler: F)
    where
        F: FnMut(Button, signal::Activate) + 'static,
    {
        self.object.signals.connect(handler);
    }
}

pub struct CheckboxData {
    base: BaseButtonData,
    model: Rc<dyn Model<bool>>,
    rocker: bool,
}

impl WidgetObject for CheckboxData {
    fn draw(data: &WidgetData<Self>, context: &mut dyn GraphicsContext, size: Size<f32>) {
        context.draw_border(size, Rect::points(1.0));
        if data.object.model.get() {
            if data.object.rocker {
                let point = Point {
                    x: (size.width * 0.75) - (size.height / 4.0),
                    y: size.height / 4.0,
                };
                context.draw_rect(
                    point,
                    Size {
                        width: size.height / 2.0,
                        height: size.height / 2.0,
                    },
                );
            } else {
                let point = Point {
                    x: size.width / 4.0,
                    y: size.height / 4.0,
                };
                context.draw_rect(point, size.map(|x| x / 2.0));
            }
        } else if data.object.rocker {
            let point = Point {
                x: (size.width * 0.25) - (size.height / 4.0),
                y: size.height / 4.0,
            };
            context.draw_rect(
                point,
                Size {
                    width: size.height / 2.0,
                    height: size.height / 2.0,
                },
            );
        }
    }
    fn set_pointer_state(data: Rc<WidgetData<Self>>, state: PointerState) {
        let checkbox = Checkbox(data);
        let visual = checkbox.object.base.set_pointer_state(state, || {
            let model = &checkbox.object.model;
            model.clone().set(!model.get());
        });
        checkbox.set_visual(Some(visual));
    }
}

define_widget!(Checkbox, CheckboxData);

impl Checkbox {
    pub fn new(gui: Rc<Gui>, model: Rc<dyn Model<bool>>) -> Self {
        let style = Style {
            min_size: Size {
                width: Dimension::Points(24.),
                height: Dimension::Points(24.),
            },
            ..Default::default()
        };
        Checkbox(WidgetData::with_style(
            gui,
            style,
            Some(VisualStyle::BUTTON),
            CheckboxData {
                base: BaseButtonData::new(),
                model,
                rocker: false,
            },
        ))
    }
    pub fn new_rocker(gui: Rc<Gui>, model: Rc<dyn Model<bool>>) -> Self {
        let style = Style {
            min_size: Size {
                width: Dimension::Points(48.),
                height: Dimension::Points(24.),
            },
            ..Default::default()
        };
        Checkbox(WidgetData::with_style(
            gui,
            style,
            Some(VisualStyle::BUTTON),
            CheckboxData {
                base: BaseButtonData::new(),
                model,
                rocker: true,
            },
        ))
    }
    pub fn value(&self) -> bool {
        self.object.model.get()
    }
}

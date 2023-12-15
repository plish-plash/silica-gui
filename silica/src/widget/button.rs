use std::{cell::Cell, rc::Rc};

use taffy::prelude::*;

use crate::{
    define_widget, signal, widget::Label, GraphicsContext, Gui, HorizontalAlign, PointerState,
    Signals, ThemeColor, VerticalAlign, VisualStyle, WidgetData, WidgetObject,
};

pub struct ButtonData {
    label: Label,
    was_pressed: Cell<bool>,
    signals: Signals<Button>,
}

impl WidgetObject for ButtonData {
    fn draw(_data: &WidgetData<Self>, _context: &mut dyn GraphicsContext, _size: Size<f32>) {}
    fn set_pointer_state(data: Rc<WidgetData<Self>>, state: PointerState) {
        let button = Button(data);
        let mut visual = VisualStyle::BUTTON;
        match state {
            PointerState::None => visual.background = Some(ThemeColor::ButtonNormal),
            PointerState::Over => visual.background = Some(ThemeColor::ButtonOver),
            PointerState::Press => visual.background = Some(ThemeColor::ButtonPress),
        }
        button.set_visual(Some(visual));
        if state == PointerState::Over && button.object.was_pressed.get() {
            button.object.signals.emit(button.clone(), signal::Activate);
        }
        button.object.was_pressed.set(state == PointerState::Press);
    }
}

define_widget!(Button, ButtonData);

impl Button {
    pub fn with_label(gui: Rc<Gui>, label_text: String) -> Self {
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
            border: Rect::points(2.0),
            align_items: Some(AlignItems::Stretch),
            justify_items: Some(JustifyItems::Stretch),
            ..Default::default()
        };
        let button = Button(WidgetData::with_style(
            gui,
            style,
            Some(VisualStyle::BUTTON),
            ButtonData {
                label: label.clone(),
                was_pressed: Cell::new(false),
                signals: Signals::new(),
            },
        ));
        button.add_child(label);
        button
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

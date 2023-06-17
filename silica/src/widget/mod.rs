use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use glyph_brush::{HorizontalAlign, OwnedSection, OwnedText, VerticalAlign};
use taffy::prelude::*;

use crate::{
    signal, GraphicsContext, Gui, PointerState, Signals, ThemeColor, VisualStyle, WidgetData,
    WidgetObject,
};

macro_rules! define_widget {
    ($widget_ty:ident, $data_ty:ty) => {
        #[derive(Clone)]
        pub struct $widget_ty(Rc<$crate::WidgetData<$data_ty>>);
        impl From<$widget_ty> for $crate::Widget {
            fn from(widget: $widget_ty) -> Self {
                widget.0
            }
        }
        impl std::ops::Deref for $widget_ty {
            type Target = $crate::WidgetData<$data_ty>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

impl WidgetObject for () {
    fn draw(_data: &WidgetData<Self>, _context: &mut dyn GraphicsContext, _size: Size<f32>) {}
}

define_widget!(Container, ());

impl Container {
    pub fn new(gui: Rc<Gui>) -> Self {
        Container(WidgetData::new(gui, false, ()))
    }
}

pub struct LabelData {
    text: RefCell<OwnedSection>,
}

impl WidgetObject for LabelData {
    fn draw(data: &WidgetData<Self>, context: &mut dyn GraphicsContext, size: Size<f32>) {
        context.draw_text(size, data.object.text.borrow().to_borrowed());
    }
}

define_widget!(Label, LabelData);

impl Label {
    pub fn new(gui: Rc<Gui>) -> Self {
        Label(WidgetData::new(
            gui,
            true,
            LabelData {
                text: RefCell::new(OwnedSection::default()),
            },
        ))
    }
    pub fn with_text(gui: Rc<Gui>, string: String) -> Self {
        let label = Self::new(gui);
        label.set_text(string);
        label
    }

    pub fn text(&self) -> String {
        let text = self.object.text.borrow();
        if text.text.is_empty() {
            return String::new();
        } else {
            return text.text[0].text.clone();
        }
    }
    pub fn set_text(&self, string: String) {
        let mut text = self.object.text.borrow_mut();
        text.text = vec![OwnedText::new(string)];
    }
    pub fn set_color(&self, color: [f32; 4]) {
        let mut text = self.object.text.borrow_mut();
        if !text.text.is_empty() {
            text.text[0].extra.color = color;
        }
    }
    pub fn set_halign(&self, h_align: HorizontalAlign) {
        let mut text = self.object.text.borrow_mut();
        text.layout = text.layout.h_align(h_align);
    }
    pub fn set_valign(&self, v_align: VerticalAlign) {
        let mut text = self.object.text.borrow_mut();
        text.layout = text.layout.v_align(v_align);
    }
}

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

use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

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
    fn draw(_data: &WidgetData<Self>, _context: &dyn GraphicsContext, _size: Size<f32>) {}
}

define_widget!(Container, ());

impl Container {
    pub fn new(gui: Rc<Gui>) -> Self {
        Container(WidgetData::new(gui, false, ()))
    }
}

pub struct LabelData {
    text: RefCell<String>,
}

impl WidgetObject for LabelData {
    fn draw(data: &WidgetData<Self>, context: &dyn GraphicsContext, _size: Size<f32>) {
        context.draw_text(&data.object.text.borrow());
    }
}

define_widget!(Label, LabelData);

impl Label {
    pub fn new(gui: Rc<Gui>) -> Self {
        Self::with_text(gui, String::new())
    }
    pub fn with_text(gui: Rc<Gui>, text: String) -> Self {
        Label(WidgetData::new(
            gui,
            true,
            LabelData {
                text: RefCell::new(text),
            },
        ))
    }
    pub fn text(&self) -> Ref<String> {
        self.object.text.borrow()
    }
    pub fn set_text(&self, text: String) {
        *self.object.text.borrow_mut() = text;
    }
}

pub struct ButtonData {
    label: Label,
    was_pressed: Cell<bool>,
    signals: Signals<Button>,
}

impl WidgetObject for ButtonData {
    fn draw(_data: &WidgetData<Self>, _context: &dyn GraphicsContext, _size: Size<f32>) {}
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
        let style = Style {
            min_size: Size {
                width: Dimension::Points(128.),
                height: Dimension::Points(32.),
            },
            border: Rect::points(2.0),
            ..Default::default()
        };
        let label = Label::with_text(gui.clone(), label_text);
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

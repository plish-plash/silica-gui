use std::{cell::RefCell, rc::Rc};

use taffy::geometry::{Point, Size};

use crate::{
    define_widget, GraphicsContext, Gui, HorizontalAlign, TextSection, VerticalAlign, WidgetData,
    WidgetObject,
};

pub struct LabelData {
    text: RefCell<TextSection>,
}

impl WidgetObject for LabelData {
    fn draw(data: &WidgetData<Self>, context: &mut dyn GraphicsContext, size: Size<f32>) {
        context.draw_text(Point::ZERO, size, &data.object.text.borrow());
    }
}

define_widget!(Label, LabelData);

impl Label {
    pub fn new(gui: Rc<Gui>) -> Self {
        Label(WidgetData::new(
            gui,
            true,
            LabelData {
                text: RefCell::new(TextSection::default()),
            },
        ))
    }
    pub fn with_text(gui: Rc<Gui>, string: String) -> Self {
        let label = Self::new(gui);
        label.set_text(string);
        label
    }

    pub fn text(&self) -> String {
        self.object.text.borrow().text.clone()
    }
    pub fn set_text(&self, string: String) {
        let mut text = self.object.text.borrow_mut();
        text.text = string;
    }
    pub fn set_font(&self, font_id: usize) {
        let mut text = self.object.text.borrow_mut();
        text.font_id = font_id;
    }
    pub fn set_font_size(&self, font_size: f32) {
        let mut text = self.object.text.borrow_mut();
        text.font_size = font_size;
    }
    pub fn set_halign(&self, h_align: HorizontalAlign) {
        let mut text = self.object.text.borrow_mut();
        text.h_align = h_align;
    }
    pub fn set_valign(&self, v_align: VerticalAlign) {
        let mut text = self.object.text.borrow_mut();
        text.v_align = v_align;
    }
}

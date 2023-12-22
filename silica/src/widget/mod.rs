mod button;
mod label;

pub use button::{Button, Checkbox};
pub use label::Label;

use std::rc::Rc;
use taffy::geometry::Size;

use crate::{GraphicsContext, Gui, WidgetData, WidgetObject};

#[macro_export]
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

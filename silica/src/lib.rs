mod graphics;
pub mod model;
pub mod signal;
pub mod widget;

use std::{
    cell::{Cell, Ref, RefCell},
    rc::{Rc, Weak},
};

use taffy::prelude::*;

pub use graphics::*;
pub use signal::{Signal, Signals};
pub use taffy;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerState {
    None,
    Over,
    Press,
}

pub trait WidgetObject: Sized {
    fn draw(data: &WidgetData<Self>, context: &mut dyn GraphicsContext, size: Size<f32>);
    fn set_pointer_state(_data: Rc<WidgetData<Self>>, _state: PointerState) {}
}

pub trait WidgetDataUntyped {
    fn node(&self) -> Node;
    fn draw(&self, context: &mut dyn GraphicsContext, size: Size<f32>);
    fn children(&self) -> Ref<Vec<Widget>>;
    fn visual(&self) -> Ref<Option<VisualStyle>>;
    fn can_highlight(&self) -> bool;
    fn set_pointer_state(self: Rc<Self>, state: PointerState);
}

pub type Widget = Rc<dyn WidgetDataUntyped>;
pub type WidgetWeak = Weak<dyn WidgetDataUntyped>;

pub struct WidgetData<T> {
    gui: Rc<Gui>,
    node: Node,
    visual: RefCell<Option<VisualStyle>>,
    can_highlight: bool,
    object: T,

    // parent: RefCell<Option<WidgetWeak>>,
    children: RefCell<Vec<Widget>>,
}

impl<T> WidgetData<T> {
    pub fn new(gui: Rc<Gui>, draw: bool, object: T) -> Rc<Self> {
        let visual = if draw {
            Some(VisualStyle::default())
        } else {
            None
        };
        Self::with_style(gui, Style::DEFAULT, visual, object)
    }
    pub fn with_style(
        gui: Rc<Gui>,
        style: Style,
        visual: Option<VisualStyle>,
        object: T,
    ) -> Rc<Self> {
        let node = gui.layout.borrow_mut().new_leaf(style).unwrap();
        let can_highlight = visual
            .as_ref()
            .map(|vis| vis.background.is_some())
            .unwrap_or(false);
        Rc::new(WidgetData {
            gui,
            node,
            visual: RefCell::new(visual),
            can_highlight,
            object,
            children: RefCell::default(),
        })
    }

    pub fn gui(&self) -> Rc<Gui> {
        self.gui.clone()
    }
    pub fn size(&self) -> Size<f32> {
        let layout_tree = self.gui.layout.borrow();
        let layout = layout_tree.layout(self.node).unwrap();
        layout.size
    }

    pub fn set_layout(&self, layout: Style) {
        self.gui
            .layout
            .borrow_mut()
            .set_style(self.node, layout)
            .unwrap();
        self.gui.mark_dirty();
    }
    pub fn set_visual(&self, visual: Option<VisualStyle>) {
        *self.visual.borrow_mut() = visual;
        self.gui.mark_dirty();
    }
    pub fn add_child<W>(&self, child: W)
    where
        W: Into<Widget>,
    {
        let child = child.into();
        self.gui
            .layout
            .borrow_mut()
            .add_child(self.node, child.node())
            .unwrap();
        self.children.borrow_mut().push(child);
        self.gui.mark_dirty();
    }
}

impl<T> Drop for WidgetData<T> {
    fn drop(&mut self) {
        let _ = self.gui.layout.borrow_mut().remove(self.node);
    }
}

impl<T> WidgetDataUntyped for WidgetData<T>
where
    T: WidgetObject,
{
    fn node(&self) -> Node {
        self.node
    }
    fn draw(&self, context: &mut dyn GraphicsContext, size: Size<f32>) {
        T::draw(self, context, size);
    }
    fn children(&self) -> Ref<Vec<Widget>> {
        self.children.borrow()
    }
    fn visual(&self) -> Ref<Option<VisualStyle>> {
        self.visual.borrow()
    }
    fn can_highlight(&self) -> bool {
        self.can_highlight
    }
    fn set_pointer_state(self: Rc<Self>, state: PointerState) {
        T::set_pointer_state(self, state);
    }
}

#[derive(Default)]
pub struct GuiState {
    highlight: Option<Widget>,
    pointer_press: bool,
}

impl GuiState {
    pub fn on_pointer_move(&mut self, gui: &Gui, root: &Widget, x: f32, y: f32) {
        let highlight = gui.hit_highlightable_widget(x, y, root);
        if highlight.as_ref().map(|w| w.node()) != self.highlight.as_ref().map(|w| w.node()) {
            if let Some(widget) = self.highlight.clone() {
                widget.set_pointer_state(PointerState::None);
            }
            if let Some(widget) = highlight.clone() {
                widget.set_pointer_state(if self.pointer_press {
                    PointerState::Press
                } else {
                    PointerState::Over
                });
            }
            self.highlight = highlight;
        }
    }
    pub fn on_pointer_button(&mut self, pointer_press: bool) {
        if let Some(widget) = self.highlight.clone() {
            widget.set_pointer_state(if pointer_press {
                PointerState::Press
            } else {
                PointerState::Over
            });
        }
        self.pointer_press = pointer_press;
    }
}

pub struct Gui {
    dirty: Cell<bool>,
    layout: RefCell<Taffy>,
    state: RefCell<GuiState>,
    signals: Signals<Rc<Self>>,
}

impl Gui {
    pub fn new() -> widget::Container {
        let gui = Rc::new(Gui {
            dirty: Cell::new(false),
            layout: RefCell::new(Taffy::new()),
            state: RefCell::default(),
            signals: Signals::new(),
        });
        let root = widget::Container::new(gui.clone());
        gui.signals.connect({
            let root = root.clone();
            move |gui, signal::Layout(available_space)| {
                let available_space = available_space
                    .map(|size| size.map(AvailableSpace::Definite))
                    .unwrap_or(Size::MAX_CONTENT);
                gui.layout
                    .borrow_mut()
                    .compute_layout(root.node(), available_space)
                    .unwrap();
            }
        });
        gui.signals.connect({
            let root = root.clone().into();
            move |gui, signal::PointerMotion { x, y }| {
                gui.state.borrow_mut().on_pointer_move(&gui, &root, x, y);
            }
        });
        gui.signals.connect(|gui, button: signal::PointerButton| {
            if let signal::PointerButton::Primary(state) = button {
                gui.state.borrow_mut().on_pointer_button(state);
            }
        });
        root
    }

    fn mark_dirty(&self) {
        self.dirty.set(true);
    }
    pub fn check_dirty(&self) -> bool {
        self.dirty.replace(false)
    }
    pub fn draw(&self, context: &mut dyn GraphicsContext, root: widget::Container) {
        self.dirty.set(false);
        self.draw_widget(context, &root.into());
    }

    pub fn emit_layout(self: &Rc<Self>, available_space: Option<Size<f32>>) {
        self.signals
            .emit(self.clone(), signal::Layout(available_space));
    }
    pub fn emit_pointer_motion(self: &Rc<Self>, x: f32, y: f32) {
        self.signals
            .emit(self.clone(), signal::PointerMotion { x, y });
    }
    pub fn emit_pointer_button(self: &Rc<Self>, button: signal::PointerButton) {
        self.signals.emit(self.clone(), button);
    }

    fn hit_highlightable_widget(&self, mut x: f32, mut y: f32, widget: &Widget) -> Option<Widget> {
        let layout_tree = self.layout.borrow();
        let layout = layout_tree.layout(widget.node()).unwrap();
        x -= layout.location.x;
        y -= layout.location.y;
        if x >= 0.0 && y >= 0.0 && x < layout.size.width && y < layout.size.height {
            for child in widget.children().iter().rev() {
                if let Some(hit_widget) = self.hit_highlightable_widget(x, y, child) {
                    return Some(hit_widget);
                }
            }
            if widget.can_highlight() {
                return Some(widget.clone());
            }
        }
        None
    }

    fn draw_widget(&self, context: &mut dyn GraphicsContext, widget: &Widget) {
        context.save();
        let layout_tree = self.layout.borrow();
        let layout = layout_tree.layout(widget.node()).unwrap();
        context.translate(layout.location.x, layout.location.y);

        if let Some(visual) = widget.visual().as_ref() {
            if let Some(background) = visual.background {
                context.set_color(background);
                context.draw_rect(layout.size);
            }
            if let Some(border) = visual.border {
                context.set_color(border);
                let border_style = layout_tree.style(widget.node()).unwrap().border;
                context.draw_border(layout.size, border_style);
            }
            if let Some(foreground) = visual.foreground {
                context.set_color(foreground);
                widget.draw(context, layout.size);
            }
        }

        for child in widget.children().iter() {
            self.draw_widget(context, child);
        }

        context.restore();
    }
}

use std::cell::RefCell;

use taffy::prelude::Size;
use type_map::TypeMap;

use crate::GraphicsContext;

pub trait Signal: 'static {}

pub struct Signals<T> {
    handlers: RefCell<TypeMap>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Signals<T>
where
    T: 'static,
{
    pub fn new() -> Self {
        Signals {
            handlers: RefCell::new(TypeMap::new()),
            _marker: std::marker::PhantomData,
        }
    }
    pub fn connect<S, F>(&self, handler: F)
    where
        S: Signal,
        F: FnMut(T, S) + 'static,
    {
        let boxed_handler: Box<dyn FnMut(T, S)> = Box::new(handler);
        self.handlers.borrow_mut().insert(boxed_handler);
    }
    pub fn emit<S>(&self, source: T, signal: S)
    where
        S: Signal,
    {
        if let Some(handler) = self.handlers.borrow_mut().get_mut::<Box<dyn FnMut(T, S)>>() {
            handler(source, signal);
        }
    }
}

pub struct Change;
impl Signal for Change {}

pub struct Activate;
impl Signal for Activate {}

pub struct Layout(pub Option<Size<f32>>);
impl Signal for Layout {}

pub struct Draw(pub Box<dyn GraphicsContext>);
impl Signal for Draw {}

pub struct PointerMotion {
    pub x: f32,
    pub y: f32,
}
impl Signal for PointerMotion {}

pub struct PointerButton {
    pub button: u8,
    pub state: bool,
}
impl Signal for PointerButton {}

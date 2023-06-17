use std::{cell::Cell, rc::Rc};

use crate::signal::*;

pub struct CellModel<T>
where
    T: Copy + 'static,
{
    data: Cell<T>,
    signals: Signals<Rc<Self>>,
}

impl<T> CellModel<T>
where
    T: Copy + 'static,
{
    pub fn new(val: T) -> Rc<Self> {
        Rc::new(CellModel {
            data: Cell::new(val),
            signals: Signals::new(),
        })
    }
    pub fn get(&self) -> T {
        self.data.get()
    }
    pub fn set(self: &Rc<Self>, val: T) {
        self.data.set(val);
        self.signals.emit(self.clone(), Change);
    }
    pub fn connect_change<F>(self: &Rc<Self>, mut handler: F)
    where
        F: FnMut(Rc<Self>, Change) + 'static,
    {
        handler(self.clone(), Change);
        self.signals.connect(handler);
    }
}

impl CellModel<i32> {
    pub fn increment(self: &Rc<Self>) {
        let val = self.get();
        self.set(val + 1);
    }
    pub fn decrement(self: &Rc<Self>) {
        let val = self.get();
        self.set(val - 1);
    }
}

pub type IntModel = CellModel<i32>;
pub type FloatModel = CellModel<f32>;

use std::{cell::Cell, rc::Rc};

use crate::signal::*;

pub trait Model<T> {
    fn get(&self) -> T;
    fn set(self: Rc<Self>, val: T);
}

pub struct CellModel<T>
where
    T: Copy + 'static,
{
    data: Cell<T>,
    signals: Signals<Rc<Self>>,
}

impl<T> Model<T> for CellModel<T>
where
    T: Copy + 'static,
{
    fn get(&self) -> T {
        self.data.get()
    }
    fn set(self: Rc<Self>, val: T) {
        self.data.set(val);
        self.signals.emit(self.clone(), Change);
    }
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
        self.clone().set(val + 1);
    }
    pub fn decrement(self: &Rc<Self>) {
        let val = self.get();
        self.clone().set(val - 1);
    }
}

pub type BoolModel = CellModel<bool>;
pub type IntModel = CellModel<i32>;
pub type FloatModel = CellModel<f32>;

pub struct SelectValue<T>
where
    T: Clone + PartialEq,
{
    model: Rc<dyn Model<T>>,
    value: T,
}

impl<T> SelectValue<T>
where
    T: Clone + PartialEq,
{
    pub fn new(model: Rc<dyn Model<T>>, value: T) -> Rc<Self> {
        Rc::new(SelectValue { model, value })
    }
}

impl<T> Model<bool> for SelectValue<T>
where
    T: Clone + PartialEq,
{
    fn get(&self) -> bool {
        self.model.get() == self.value
    }
    fn set(self: Rc<Self>, val: bool) {
        if val {
            self.model.clone().set(self.value.clone());
        }
    }
}

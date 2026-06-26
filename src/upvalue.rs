use crate::value::Value;
//use crate::prelude::Closure;
use crate::utils::do_if_some;
use crate::managed::{GcTrace, Managed, Manage};
use std::{mem};


#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Upvalue {
    Open(usize),
    Closed(Box<Value>),
}

impl Upvalue {
    pub fn hoist(&mut self, stack: &Vec<Value>) {
        match self {
            Upvalue::Open(index) => {
                let value = unsafe { stack.get_unchecked(*index) }.clone();
                let _ = mem::replace(self, Upvalue::Closed(Box::new(value)));
            },
            Upvalue::Closed(_) => panic!("Attempted to hoist closed upvalue"),
        }
    }

    pub fn is_open(&self) -> bool {
        match self {
            Upvalue::Open(_) => true,
            Upvalue::Closed(_) => false,
        }
    }
}

impl GcTrace for Upvalue {
    fn trace(&self, mark: &mut dyn FnMut(Managed<dyn Manage>)) -> bool {
        if let Upvalue::Closed(upvalue) = self {
            do_if_some(upvalue.get_dyn_managed(), |obj| mark(obj));
        }
        true
    }
}


impl Manage for Upvalue {
    fn type_name(&self) -> &str{
        "upvalue"
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}


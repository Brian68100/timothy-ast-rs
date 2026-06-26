use crate::function_core::*;
use crate::value::Value;
use crate::utils::do_if_some;
use crate::managed::*;
use std::{fmt, mem};
//use crate::upvalue::*;


#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Closure {
    core: Managed<FunctionCore>,
    upvalues: Managed<Vec<Value>>,
}

impl Closure {
    pub fn new
    (
        core: Managed<FunctionCore>,
        upvalues: Managed<Vec<Value>>,
    ) -> Self
    {
        Self {
            core,
            upvalues,
        }
    }

    #[inline]
    pub fn get_arity(&self) -> usize {
        self.core.get_arity()
    }
    #[inline]
    pub fn get_core(&self) -> Managed<FunctionCore> {
        self.core
    } 
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Closure")
            .field("core", &self.core)
            .field("upvalues", &self.upvalues)
            .finish()
    }
}

impl GcTrace for Closure {
    fn trace(&self, mark: &mut dyn FnMut(Managed<dyn Manage>)) -> bool {
        self
            .upvalues
            .iter()
            .for_each(|constant| do_if_some(constant.get_dyn_managed(),
                |obj|  mark(obj)));

        mark(self.core.clone_dyn());
        true
    }
}

impl Manage for Closure {
    fn type_name(&self) -> &str {
        "closure"
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

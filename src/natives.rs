use crate::gc::Gc;
use crate::parser::Parser;
use crate::native_fun::{add_default_fn_natives, NativeFun};
use crate::parser::MAX_MODULES;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) fn add_default_natives(
    parser: &mut Parser,
    gc: &Gc,
)
{
    add_default_fn_natives(parser, gc);
}

use crate::gc::Gc;
use crate::parser::Parser;
use crate::native_fun::add_default_fn_natives;

pub(crate) fn add_default_natives(
    parser: &mut Parser,
    gc: &Gc,
)
{
    add_default_fn_natives(parser, gc);
}

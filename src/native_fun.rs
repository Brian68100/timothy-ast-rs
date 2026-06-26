use crate::value::Value;
use crate::parser::Parser;
use crate::managed::*;
use std::{fmt, mem, ptr};
use std::io;
use std::io::Write;
use crate::gc::{Gc, NO_GC};

type FunResult = Result<Value, String>;
type FunPtr = fn(&Gc, &[Value]) -> FunResult;

#[derive(Eq, Hash, Clone)]
pub struct NativeFun {
    name: String,
    arity: u16,
    fun: FunPtr,
}

impl NativeFun {
    pub fn new
    (
        name: String,
        arity: u16,
        fun: FunPtr,
    ) -> Self
    {
        Self {
            name,
            arity,
            fun,
        }
    }

    pub fn get_name(&self) -> String{
        self.name.clone()
    }

    pub fn get_arity(&self) -> u16 {
        self.arity
    }

    pub fn call(&mut self, gc: &Gc, args: &[Value]) -> FunResult {
        (self.fun)(gc, args)
    }
}

impl PartialEq<NativeFun> for NativeFun {
	fn eq(&self, rhs: &NativeFun) -> bool {
		ptr::eq(self, rhs)
	}
}

impl fmt::Debug for NativeFun {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("NativeFun")
			.field("name", &self.name)
			.field("arity", &self.arity)
			.finish()
	}
}

impl GcTrace for NativeFun {
    fn trace(&self, _: &mut dyn FnMut(Managed<dyn Manage>)) -> bool {
        true
    }
}

impl Manage for NativeFun {
    fn type_name(&self) -> &str {
        "native fun"
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}


pub(crate) fn add_default_fn_natives(parser: &mut Parser, gc: &Gc,) {
    parser.load(Value::NativeFun(gc.manage(NativeFun::new("print".to_string(), 1, builtin_fun_print_1), &NO_GC)));
    parser.load(Value::NativeFun(gc.manage(NativeFun::new("println".to_string(), 0, builtin_fun_println_0), &NO_GC)));
    parser.load(Value::NativeFun(gc.manage(NativeFun::new("println".to_string(), 1, builtin_fun_println_1), &NO_GC)));
    parser.load(Value::NativeFun(gc.manage(NativeFun::new("input".to_string(), 0, builtin_fun_input_0), &NO_GC))); 
}

fn builtin_fun_print_1(gc: &Gc, args: &[Value]) -> FunResult {
    if args.is_empty() {
    } else if args[0].is_array() {
	    if let Ok(arr) = args[0].get_array() {
	        for i in &*arr {
		        if let Ok(j) = i.get_string() {
		            print!("{}", *j);
		        }
	        }
	    }
    }
    Ok(Value::Nil)
}
    
fn builtin_fun_println_0(gc: &Gc, args: &[Value]) -> FunResult {
    println!();
    Ok(Value::Nil)
}
    
fn builtin_fun_println_1(gc: &Gc, args: &[Value]) -> FunResult {
    if args.is_empty() {
    } else if args[0].is_array() {
	if let Ok(arr) = args[0].get_array() {
	    for i in &*arr {
		if let Ok(j) = i.get_string() {
		    
		    print!("{}", *j);
		}
	    }
	}
    }
    println!();
    Ok(Value::Nil)
}

fn builtin_fun_input_0(gc: &Gc, args: &[Value]) -> FunResult {
    let mut input = String::new();
    io::stdout().flush().unwrap();
    match io::stdin().read_line(&mut input) {
	    Ok(num) => {
	        Ok(Value::String(gc.manage(input.clone(), &NO_GC)))
	    }
	    Err(_) => {
	        Ok(Value::Nil)
	    }
    }
}

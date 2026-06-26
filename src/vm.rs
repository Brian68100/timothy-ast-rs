use crate::value::*;
use crate::function_core::{FunctionCore, FunctionType};
use crate::closure::Closure;
use crate::managed::*;
use crate::opcode::*;
use crate::ast::*;
use crate::parser::*;
use crate::gc::{Gc, NO_GC};
use std::collections::HashMap;
use std::ptr;
use ordered_float::*;
use std::iter::Rev;
use std::io;

#[derive(Debug, Clone, PartialEq)]
struct CallFrame {
    fun: Value,
    closure: Option<Managed<Closure>>,
    core: Option<Managed<FunctionCore>>,
    pc: usize,
    fp: usize,
}

impl CallFrame {
    pub fn new(fun: Value) -> Self {
        let _closure: Option<Managed<Closure>> = None;
        let _core: Option<Managed<FunctionCore>> = None;
        
        match fun {
            Value::Closure(c) => {
                Self {
                    fun,
                    closure: Some(c),
                    core: Some(c.get_core()),
                    pc: 0,
                    fp: 0,
                }
            },
            _ => { panic!("'{}' got pushed onto the call stack!", fun); }
        }
    }
    pub fn get_fun(&self) -> &Value {
        &self.fun
    }

    pub fn get_closure(&self) -> Option<Managed<Closure>> {
        self.closure
    }

    fn get_core(&self) -> Option<Managed<FunctionCore>> {
        self.core
    }

    fn get_pc(&self) -> usize {
        self.pc.clone()
    }

    fn inc_pc(&mut self, amount: usize) -> Option<usize> {
        if self.pc + amount > 200_000 {
            None
        } else {
            self.pc += amount;
            Some(self.pc)
        }
    }

    fn dec_pc(&mut self, amount: usize) -> Option<usize> {
        let mut new_pc = self.pc as isize;
        new_pc -= amount as isize;
        if new_pc < 0 {
            None
        } else {
            self.pc -= amount;
            Some(self.pc)
        }
    }

    #[inline]
    fn get_fp(&self) -> usize {
        self.fp
    }

    #[inline]
    fn set_fp(&mut self, index: usize) -> usize{
        self.fp = index;
	self.fp
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CallStack {
    stack: Vec<CallFrame>,
} 

impl CallStack {
    fn new() -> Self {
        Self {
            stack: vec![],
        }
    }

    #[inline]
    fn get_fp(&self) -> usize {
        self.stack.last().unwrap().get_fp()
    }
    
    #[inline]
    fn set_fp(&mut self, index: usize) {
        self.stack.last_mut().unwrap().set_fp(index);
    }

    #[inline]
    fn get_pc(&self) -> usize {
        self.stack.last().unwrap().get_pc()
    }

    fn clear_pc(&mut self) {
        self.stack.last_mut().unwrap().pc = 0;
    }

    #[inline]
    fn inc_pc(&mut self, amount: usize) -> Option<usize> {
	if let Some(last) = self.stack.last_mut() {
	    last.inc_pc(amount)
	} else {
	    None
	}
    }

    #[inline]
    fn dec_pc(&mut self, amount: usize) -> Option<usize> {
	if let Some(last) = self.stack.last_mut() {
	    last.dec_pc(amount)
	} else {
	    None
	}
    }

    fn top(&self) -> Option<CallFrame> {
        if let Some(last) = self.stack.last() {
	    return Some(last.clone());
        } else {
            None
        }
    }

    fn next_from_top(&self) -> Option<CallFrame> {
        if self.stack.len() > 1 {
            let tmp = self.stack.len() - 2;
            let nxt = self.stack[tmp].clone();
            Some(nxt)
        } else {
            None
        }
    } 

    fn push(&mut self, fun: Value) {
        self.stack.push(CallFrame::new(fun))
    }

    fn pop(&mut self) -> Option<CallFrame> {
        self.stack.pop()
    }

    fn clear(&mut self) {
        self.stack.clear();
    }

    #[allow(unused_assignments)]
    pub(crate) fn get_backtrace(&self) -> String {
        let mut string = format!("\nBacktrace:\n");
        
        let mut iter = self.stack.iter();
        loop {
            let elem = iter.next();
            if elem.is_none() {
                break;
            }

            let mut arity = 0usize;
            let mut name = String::from("");
            let mut fun_ty = FunctionType::NamedClosure;
            let mut fun_ty_name = String::from("");
            let filename = String::from("");
            if let Some(f) = elem {
                match f.fun {
                    Value::Closure(c) => {
                        name = c.get_core().get_name();
                        arity = c.get_core().get_arity();
                        fun_ty = c.get_core().get_type();
                        match fun_ty {
                            FunctionType::TopLevel => {
                                fun_ty_name = String::from("top-level");
                            },
                            FunctionType::NamedClosure => {
                                fun_ty_name = String::from("named closure");
                            },
                            FunctionType::AnonClosure => {
                                fun_ty_name = String::from("anonymous closure");
                            },
                            _ => { unreachable!(); }
                        }
                    },
                    _ => {},
                }
                string = string + &format!("[ {} {}(#{}) at {} line {} ]\n\n", 
					   fun_ty_name,
					   name,
					   arity,
					   filename,
					   0,
                );
            }
        }
        string
    }
}

pub fn is_false(x: Value) -> bool {
    x == Value::Nil || (x.is_bool() && !x.as_bool())
}

macro_rules! read_byte {
    ($pc:ident, $code:ident, $out:ident) => {
        $pc += 1;
        let $out = $code[$pc - 1];
    };
}

macro_rules! read_word {
    ($pc:ident, $code:ident, $out:ident) => {
        $pc += 2;
        let $out = (($code[$pc - 2] as usize) << 8usize) | $code[$pc - 1] as usize;
    };
}

#[allow(dead_code)]
pub struct VM {
    vars: Vec<Value>,
    main_fun: Option<Managed<Closure>>,
    ast_parser: Option<AstParser>,
    calls: CallStack,
    stack: Vec<Value>,
    sp: usize,

    compiler: Compiler,
}



#[allow(dead_code)]
impl VM {
    pub fn new() -> Self
    {
        Self {
            vars:Vec::<Value>::new(),
            main_fun: None,
            ast_parser: None,
            calls: CallStack::new(),
            stack: Vec::<Value>::new(),
            sp: 0usize,

            compiler: Compiler::new(),
        }
    }

    fn parse_ast
	(
            &mut self, 
            root: &Option<Box<Ast>>,
            gc: &Gc,
            compile: bool,
	) -> Option<Managed<Closure>>
    {
        let mut ast_parser = AstParser::new();
        ast_parser.parse(root, self, gc, compile)
    }

    #[inline]
    fn get_sp(&self) -> usize {
        self.sp
    }

    #[inline]
    fn set_sp(&mut self, index: usize) {
        self.sp = index;
    }

    #[inline]
    fn get_fp(&self) -> usize {
        self.calls.get_fp()
    }

    #[inline]
    fn set_fp(&mut self, index: usize) {
        self.calls.set_fp(index);
    }
    #[inline]
    fn get_pc(&mut self) -> Option<usize>{
        if let Some(frame) = self.calls.top() {
            Some(frame.get_pc())
        } else {
            None
        }
    }
    #[inline]
    fn inc_pc(&mut self, amount: usize) -> Option<usize> {
        if let Some(mut frame) = self.calls.top() {
            frame.inc_pc(amount)
        } else {
            None
        }
    }
    #[inline]
    fn dec_pc(&mut self, amount: usize) -> Option<usize> {
        if let Some(mut frame) = self.calls.top() {
            frame.dec_pc(amount)
        } else {
            None
        }
    }
    fn get_stack_at_index(&mut self, index: usize) -> Option<Value> {
        if let Some(val) = self.stack.get(index) {
            Some((*val).clone())
        } else {
            None
        }
    } 

    fn get_stack_top(&mut self) -> Option<Value> {
        if let Some(val) = self.stack.last_mut() {
            Some((*val).clone())
        } else {
            None
        }
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn stack_pop(&mut self) {
        self.stack.pop();
    }

    fn get_var(&mut self, index: usize) -> Option<Value> {
        if let Some(val) = self.vars.get_mut(index) {
            Some((*val).clone())
        } else {
            None
        }
    }

    fn set_var(&mut self, index: usize, value: Value) {
        if let Some(val) = self.vars.get_mut(index) {
            *val = value;
        }
    }

    fn add_var(&mut self) {
        self.vars.push(Value::Nil);
    }

    fn push_call_frame
	(
            &mut self,
            num_args: usize,
	)
    {
        let top = self.stack.len(); 
        let call = &self.stack[top - num_args as usize - 1];
        self.calls.push(call.clone());
        self.calls.set_fp(top - num_args as usize - 1);
        self.calls.clear_pc();
    }

    fn pop_call_frame(&mut self, mut fp: usize) -> usize {
        self.stack.resize(fp + 1, Value::Nil);
        
        if let Some(new_top) = self.calls.next_from_top() {
            fp = new_top.fp;
        } else if let Some(new_top) = self.calls.top() {
            fp = new_top.fp;
        }
        self.calls.pop();
        fp
    }

    fn check_arity
	(
            &self,
            fun: &Value,
            num_args: usize,
	) -> Result<usize, String>
    {
        let top = self.stack.len();
        let mut arity = 0;
        match fun {
            Value::Closure(c) => {
                arity = c.get_arity();
            },
            _ => {
                return Err(format!("not a function type, but {}", fun.value_type()));
            },
        }
        if num_args != arity {
            Err(format!("{} args expected, found {}", arity, num_args))
        } else {
            Ok(num_args)
        }
    }

    fn call_value
	(
            &mut self,   
            num_args: usize,
            debug: bool,
            gc: &Gc,
	) -> Result<Value, String>
    {
        let fun_index = self.stack.len() - (num_args as usize) - 1;
        let fun = &self.stack[fun_index];
        let args = &self.stack[fun_index..fun_index + (num_args as usize) + 1];
        match self.check_arity(fun, num_args) {
            Ok(_) => {},
            Err(e) => {
                return Err(e);
            },
        }       
        match fun {
            Value::Closure(_) => {
                self.call_closure(num_args, gc, debug)
            },
            _ => {
                Err(format!("not a function type, but {}", fun.value_type()))
            }
        } 
    }

    #[allow(unused_assignments)]
    fn call_closure
	(
            &mut self,
            num_args: usize,
            gc: &Gc,
            debug: bool,
	) -> Result<Value, String>
    {
        let mut pc = 0usize;

        if let Some(counter) = self.get_pc() {
            pc = counter;
        } else {
            panic!("no functions on call stack");
        }
        /*if debug {
        return call_closure_debug(self, u8, gc);
    }*/

        self.push_call_frame(num_args);

        let mut result = Ok(Value::Nil);
        let mut curr_byte = 0u8;
        let _sp = self.get_sp();
        let mut fp = self.get_fp();
        let mut closure: Managed<Closure> = self.stack[fp].as_closure();
        let fc = closure.get_core();
        let code: &[u8] = fc.get_code();
        

        let _sp = 0usize;
        while pc < fc.len() && result == Ok(Value::Nil) {
            closure = self.stack[fp].as_closure();
            fp = self.calls.get_fp(); 
            
            read_byte!(pc, code, byte);
            curr_byte = byte;  

            let opcode = OpCode::from(curr_byte);

            #[cfg(feature = "debug_opcode")]
            {
                println!("Opcode {}:", opcode);
            }
            #[cfg(feature = "debug_vars")]
            {
                println!("");
                self.debug_vars();
            }
            #[cfg(feature = "debug_stack")]
            {
                println!("");
                self.debug_stack(fp);
            }
            
            result = match opcode {
                OpCode::PushN => {
                    read_byte!(pc, code, arg);
                    for _i in 0..arg {
                        self.stack.push(Value::Nil);
                    }
                    
                    Ok(Value::Nil)
                },
                OpCode::Pop => {
                    self.stack_pop();
                    Ok(Value::Nil)
                },
                OpCode::PopN => {
                    read_byte!(pc, code, arg);
                    for _i in 0..arg {
                        self.stack_pop();
                    }
                    
                    Ok(Value::Nil)
                },
                OpCode::Dup => {
                    let last = self.get_stack_top();
                    if let Some(i) = last {
                        self.stack.push(i.clone());
                    } else {
                        panic!("accessed stack from out of bounds");
                    } 
                    Ok(Value::Nil)
                },
                OpCode::LoadTrue => {
                    self.stack.push(Value::Bool(true));
                    Ok(Value::Nil)
                },
                OpCode::LoadFalse => {
                    self.stack.push(Value::Bool(false));
                    Ok(Value::Nil)
                },
                OpCode::LoadNil => {
                    self.stack.push(Value::Nil);
                    Ok(Value::Nil)
                },
                OpCode::Load0 |
                OpCode::Load1 |
                OpCode::Load2 |
                OpCode::Load3 |
                OpCode::Load4 |
                OpCode::Load5 |
                OpCode::Load6 |
                OpCode::Load7 |
                OpCode::Load8 |
                OpCode::Load9 |
                OpCode::Load10 => {
                    self.stack.push(Value::Int((u8::from(opcode) - (u8::from(OpCode::Load0))) as i64));
                    Ok(Value::Nil)
                }, 
                OpCode::LoadValue => {
                    read_word!(pc, code, arg);
                    self.stack.push(closure.get_core().get_constants()[arg as usize].clone());
                    Ok(Value::Nil)
                },
                OpCode::Array => {
                    Ok(Value::Nil)
                },
                OpCode::Dict => {
                    Ok(Value::Nil)
                },
                OpCode::DefVar => {
                    read_word!(pc, code, index);
                    let top = self.get_stack_top();
                    
                    if let Some(val) = top {
                        self.set_var(index as usize, val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }, 
                OpCode::LoadVar => {
                    read_word!(pc, code, arg);
                    let elem = self.get_var(arg as usize).clone();
                    if let Some(val) = elem {
                        self.stack.push(val);
                    } else {
			panic!("var index out of range: {}; limit: {}", arg, self.vars.len());
                    }
                    
                    Ok(Value::Nil)
                }, 
                OpCode::StoreVar => {
                    read_word!(pc, code, arg);
                    if let Some(j) = self.get_stack_top() { 
                        self.set_var(arg as usize, j.clone());
                    }
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal0 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal1 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal2 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal3 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal4 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal5 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal6 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal7 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadLocal8 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal0 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal1 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal2 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal3 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal4 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal5 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal6 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal7 => {
                    Ok(Value::Nil)
                },
                OpCode::StoreLocal8 => {
                    Ok(Value::Nil)
                },
                OpCode::LoadUpvalue => {
                    Ok(Value::Nil)
                },
                OpCode::StoreUpvalue => {
                    Ok(Value::Nil)
                },
                OpCode::LoadField => {
                    Ok(Value::Nil)
                },
                OpCode::StoreField => {
                    Ok(Value::Nil)
                },
                OpCode::LoadStatic => {
                    Ok(Value::Nil)
                },
                OpCode::StoreStatic => {
                    Ok(Value::Nil)
                }, 
                OpCode::LoadMethod => {
                    Ok(Value::Nil)
                }, 
                OpCode::LoadStaticMethod => {
                    Ok(Value::Nil)
                }, 
                OpCode::LoadSuperMethod => {
                    Ok(Value::Nil)
                },
                OpCode::JumpFwd => {
                    read_word!(pc, code, arg);
                    pc += arg as usize; 
                    Ok(Value::Nil)
                }, 
                OpCode::JumpBack => {
                    read_word!(pc, code, arg);
                    pc -= arg as usize;
                    Ok(Value::Nil)
                }, 
                OpCode::JumpTrue => {
                    read_word!(pc, code, arg);
                    let val = self.get_stack_top();
                    if let Some(i) = val {
                        if !is_false(i.clone()) {
                            pc += arg as usize;
                        }
                    }
                    Ok(Value::Nil)
                }, 
                OpCode::JumpFalse => {
                    read_word!(pc, code, arg);
                    let val = self.get_stack_top();
                    if let Some(i) = val {
                        if is_false(i.clone()) {
                            pc += arg as usize;
                        }
                    }
                    Ok(Value::Nil)
                },
                OpCode::DefStatic => {
                    Ok(Value::Nil)
                },
                OpCode::Method => {
                    Ok(Value::Nil)
                },
                OpCode::StaticMethod => {
                    Ok(Value::Nil)
                },
                OpCode::Instance => {
                    Ok(Value::Nil)
                },
                OpCode::Closure => {
                    Ok(Value::Nil)
                },
                OpCode::CloseUpvalue => {
                    Ok(Value::Nil)
                },
                OpCode::Call => {
                    read_byte!(pc, code, num_args);
                    println!("num_args = {}", num_args);
                    
                    
                    match self.call_value((num_args as usize) + 1, debug, gc) { 
                        Ok(ref res) => {
                            let top = self.stack.len();
                            self.stack[top - (num_args as usize) - 1] = res.clone();
                            Ok(res.clone())
                        },
                        Err(e) => {
                            Err(e)
                        },
                    }
                },
                OpCode::Return => {
                    if let Some(val) = self.calls.top() {
                        fp = val.fp;
                    } else {
                        panic!("no calls left on the call stack");
                    }

                    let top = self.stack.len();
                    let last = self.stack[top - 1].clone();
                    self.stack[fp] = last.clone();
                    fp = self.pop_call_frame(fp);

                    Ok(last.clone())
                },
                OpCode::Ternary => {
                    Ok(Value::Nil)
                },
                OpCode::Neg => {
                    if let Some(i) = self.get_stack_top() {
                        match i {
                            Value::Int(int) => {
                                let val: Value = Value::Int(-int);
                                let mut elem = self.get_stack_top();
                                if let Some(ref mut e) = elem {
                                    *e = val;
                                } else {
                                    panic!("stack is empty");
                                }
                                Ok(Value::Nil)
                            },
                            Value::Float(float) => {
                                let val: Value = Value::Float(-float);
                                let mut elem = self.get_stack_top();
                                if let Some(ref mut e) = elem {
                                    *e = val;
                                } else {
                                    panic!("stack is empty");
                                }
                                Ok(Value::Nil)
                            },
                            _ => Err("cannot negate value".to_string())
                        }
                    } else {
                        panic!("stack is empty");
                    }
                },
                OpCode::Print => {
                    if let Some(val) = self.get_stack_top() {
                        print!("{}", val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                },
                OpCode::Println => {
                    if let Some(val) = self.get_stack_top() {
                        println!("{}", val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                },
                OpCode::Input => {
                    Ok(Value::Nil)
                },
                OpCode::MathOp => {
                    read_byte!(pc, code, arg);
                    self.execute_math_op(MathOp::from(arg), gc)
                },
                OpCode::MathAssignOp => {
                    Ok(Value::Nil)
                },
                OpCode::BitwiseOp => {
                    Ok(Value::Nil)
                },
                OpCode::BitwiseAssignOp => {
                    Ok(Value::Nil)
                },
                OpCode::Invalid => {
                    panic!("{}", "invalid instruction".to_string());
                }
            }; 
        }

        match result {
            Ok(ref res) => {
                let top = self.stack.len();
                self.stack[top - (num_args as usize) - 1] = res.clone();
                Ok(res.clone())
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    /*fn call_closure_debug
    (
    &mut self,
    num_args: u8,
    gc: &Gc,
)
    {
    self.push_call_frame(num_args);

    let mut result = Ok(Value::Nil);
    let mut curr_byte = 0u8;
    let _sp = self.get_sp();
    let mut fp = self.get_fp();
    let mut closure: Managed<Closure> = self.stack[fp].as_closure();
    let fc = closure.get_core();
    let code: &[u8] = fc.get_code();
    

    let _sp = 0usize;

    let mut debugger = Debugger::new();
    while pc < fc.len() && result == Ok(Value::Nil) {
    read_byte!(pc, code, byte);

    let opcode = OpCode::from(byte);

    let mut command = debugger.get_command() { 
    
    result = match opcode {
    OpCode::PushN => {
    read_byte!(pc, code, arg);
    for _i in 0..arg {
    self.stack.push(Value::Nil);
}
    
    Ok(Value::Nil)
},
    OpCode::Pop => {
    self.stack_pop();
    Ok(Value::Nil)
},
    OpCode::PopN => {
    read_byte!(pc, code, arg);
    for _i in 0..arg {
    self.stack_pop();
}
    
    Ok(Value::Nil)
},
    OpCode::Dup => {
    let last = self.get_stack_top();
    if let Some(i) = last {
    self.stack.push(i.clone());
} else {
    panic!("accessed stack from out of bounds");
} 
    Ok(Value::Nil)
},
    OpCode::LoadTrue => {
    self.stack.push(Value::Bool(true));
    Ok(Value::Nil)
},
    OpCode::LoadFalse => {
    self.stack.push(Value::Bool(false));
    Ok(Value::Nil)
},
    OpCode::LoadNil => {
    self.stack.push(Value::Nil);
    Ok(Value::Nil)
},
    OpCode::Load0 |
    OpCode::Load1 |
    OpCode::Load2 |
    OpCode::Load3 |
    OpCode::Load4 |
    OpCode::Load5 |
    OpCode::Load6 |
    OpCode::Load7 |
    OpCode::Load8 |
    OpCode::Load9 |
    OpCode::Load10 => {
    self.stack.push(Value::Int((u8::from(opcode) - (u8::from(OpCode::Load0))) as i64));
    Ok(Value::Nil)
}, 
    OpCode::LoadValue => {
    read_word!(pc, code, arg);
    self.stack.push(closure.get_core().get_constants()[arg as usize].clone());
    Ok(Value::Nil)
},
    OpCode::Array => {
    Ok(Value::Nil)
},
    OpCode::Dict => {
    Ok(Value::Nil)
},
    OpCode::DefVar => {
    read_word!(pc, code, arg);
    let top = self.get_stack_top();
    
    if let Some(val) = top {
    self.add_var(val);
} else {
    panic!("stack is empty");
}
    Ok(Value::Nil)
}, 
    OpCode::LoadVar => {
    read_word!(pc, code, arg);
    let elem = self.get_var(arg as usize).clone();
    if let Some(val) = elem {
    self.stack.push(val);
} else {
    unreachable!();
}
    
    Ok(Value::Nil)
}, 
    OpCode::StoreVar => {
    read_word!(pc, code, arg);
    if let Some(j) = self.get_stack_top() { 
    self.set_var(arg as usize, j.clone());
}
    Ok(Value::Nil)
},
    OpCode::LoadLocal => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal0 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal1 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal2 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal3 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal4 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal5 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal6 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal7 => {
    Ok(Value::Nil)
},
    OpCode::LoadLocal8 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal0 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal1 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal2 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal3 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal4 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal5 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal6 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal7 => {
    Ok(Value::Nil)
},
    OpCode::StoreLocal8 => {
    Ok(Value::Nil)
},
    OpCode::LoadUpvalue => {
    Ok(Value::Nil)
},
    OpCode::StoreUpvalue => {
    Ok(Value::Nil)
},
    OpCode::LoadField => {
    Ok(Value::Nil)
},
    OpCode::StoreField => {
    Ok(Value::Nil)
},
    OpCode::LoadStatic => {
    Ok(Value::Nil)
},
    OpCode::StoreStatic => {
    Ok(Value::Nil)
}, 
    OpCode::LoadMethod => {
    Ok(Value::Nil)
}, 
    OpCode::LoadStaticMethod => {
    Ok(Value::Nil)
}, 
    OpCode::LoadSuperMethod => {
    Ok(Value::Nil)
},
    OpCode::JumpFwd => {
    read_word!(pc, code, arg);
    pc += arg as usize; 
    Ok(Value::Nil)
}, 
    OpCode::JumpBack => {
    read_word!(pc, code, arg);
    pc -= arg as usize;
    Ok(Value::Nil)
}, 
    OpCode::JumpTrue => {
    read_word!(pc, code, arg);
    let val = self.get_stack_top();
    if let Some(i) = val {
    if !is_false(i.clone()) {
    pc += arg as usize;
}
}
    Ok(Value::Nil)
}, 
    OpCode::JumpFalse => {
    read_word!(pc, code, arg);
    let val = self.get_stack_top();
    if let Some(i) = val {
    if is_false(i.clone()) {
    pc += arg as usize;
}
}
    Ok(Value::Nil)
},
    OpCode::DefStatic => {
    Ok(Value::Nil)
},
    OpCode::Method => {
    Ok(Value::Nil)
},
    OpCode::StaticMethod => {
    Ok(Value::Nil)
},
    OpCode::Instance => {
    Ok(Value::Nil)
},
    OpCode::Closure => {
    Ok(Value::Nil)
},
    OpCode::CloseUpvalue => {
    Ok(Value::Nil)
},
    OpCode::Call => {
    read_byte!(pc, code, num_args);
    println!("num_args = {}", num_args);
    
    
    match self.call_value(num_args, debug, gc) { 
    Ok(ref res) => {
    let top = self.stack.len();
    self.stack[top - (num_args as usize) - 1] = res.clone();
    Ok(res.clone())
},
    Err(e) => {
    Err(e)
},
}
},
    OpCode::Return => {
    if let Some(val) = self.calls.top() {
    fp = val.fp;
} else {
    panic!("no calls left on the call stack");
}
    let
    let top = self.stack.len();
    let last = self.stack[top - 1].clone();
    self.stack[fp] = last.clone();
    fp = self.pop_call_frame(fp);

    Ok(last.clone())
},
    OpCode::Ternary => {
    Ok(Value::Nil)
},
    OpCode::Neg => {
    if let Some(i) = self.get_stack_top() {
    match i {
    Value::Int(int) => {
    let val: Value = Value::Int(-int);
    let mut elem = self.get_stack_top();
    if let Some(ref mut e) = elem {
     *e = val;
} else {
    panic!("stack is empty");
}
    Ok(Value::Nil)
},
    Value::Float(float) => {
    let val: Value = Value::Float(-float);
    let mut elem = self.get_stack_top();
    if let Some(ref mut e) = elem {
     *e = val;
} else {
    panic!("stack is empty");
}
    Ok(Value::Nil)
},
    _ => Err("cannot negate value".to_string())
}
} else {
    panic!("stack is empty");
}
},
    OpCode::Print => {
    if let Some(val) = self.get_stack_top() {
    print!("{}", val);
} else {
    panic!("stack is empty");
}
    Ok(Value::Nil)
},
    OpCode::Println => {
    if let Some(val) = self.get_stack_top() {
    println!("{}", val);
} else {
    panic!("stack is empty");
}
    Ok(Value::Nil)
},
    OpCode::Input => {
    Ok(Value::Nil)
},
    OpCode::MathOp => {
    read_byte!(pc, code, arg);
    self.execute_math_op(MathOp::from(arg), gc)
},
    OpCode::MathAssignOp => {
    Ok(Value::Nil)
},
    OpCode::BitwiseOp => {
    Ok(Value::Nil)
},
    OpCode::BitwiseAssignOp => {
    Ok(Value::Nil)
},
    OpCode::Invalid => {
    panic!("{}", "invalid instruction".to_string());
}
}; 
}

    match result {
    Ok(ref res) => {
    let top = self.stack.len();
    self.stack[top - (num_args as usize) - 1] = res.clone();
    Ok(res.clone())
},
    Err(e) => {
    Err(e)
},
}
}*/

    fn debug_vars(&self) {
        print!("NS Vars: [");
        for i in 0..self.vars.len() {
            print!("[{}]", self.vars[i]);
        }
        println!("]");
    }
    fn debug_stack(&self, fp: usize) {
        print!("[");
        for i in 0..self.stack.len() { 
            print!("[");
            if i == fp {
                print!("> ");
            }
            print!("{}", self.stack[i]);
            if i == fp {
                print!(" <");
            }
            print!("]");
        }
        println!("]");
    }

    fn execute_math_op
	(
            &mut self,
            op: MathOp,
            gc: &Gc,
	) -> Result<Value, String>
    {
        

        match op {
            MathOp::Add => { 
                self.execute_add(gc)
            },
            MathOp::Subtract => {
                self.execute_subtract(gc)
            },
            MathOp::Multiply => {
                self.execute_multiply(gc)
            },
            MathOp::Divide => {
                self.execute_divide(gc)
            },
            MathOp::Modulo => {
                self.execute_modulo(gc)
            },
            MathOp::Power => {
                self.execute_power(gc)
            },
            _ => { panic!("invalid math operator");},
        }
    } 

    fn execute_math_assign_op
	(
            &mut self,
            op: MathOp,
            gc: &Gc,
	) -> Result<Value, String>
    {
        

        match op {
            MathOp::Add => { 
                self.execute_add_assign(gc)
            },
            MathOp::Subtract => {
                self.execute_subtract_assign(gc)
            },
            MathOp::Multiply => {
                self.execute_multiply_assign(gc)
            },
            MathOp::Divide => {
                self.execute_divide_assign(gc)
            },
            MathOp::Modulo => {
                self.execute_modulo_assign(gc)
            },
            MathOp::Power => {
                self.execute_power_assign(gc)
            },
            _ => { panic!("invalid compound assignment operator");},
        }
    }

    fn execute_add
	(
            &mut self,
            gc: &Gc,
	) -> Result<Value, String>  {
            let arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            arg1 = Value::Int(val1 + val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		Value::Float(val1) => {
                    match arg2 {
			Value::Float(val2) => {
                            arg1 = Value::Float(val1 + val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		Value::String(val1) => {
                    match arg2 {
			Value::String(val2) => {
                            arg1 = Value::String(gc.manage(format!("{}{}", *val1, *val2), &NO_GC));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {},
            }
            Err(format!(
		"+: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_subtract
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String>  {
            let arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            arg1 = Value::Int(val1 - val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(val1) => {
                    match arg2 {
			Value::Float(val2) => {
                            arg1 = Value::Float(val1 - val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {}, 
            }
            Err(format!(
		"-: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_multiply
	(
            &mut self,
            gc: &Gc,
	) -> Result<Value, String> {
            let arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            arg1 = Value::Int(val1 * val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(val1) => {
                    match arg2 {
			Value::Float(val2) => {
                            arg1 = Value::Float(val1 * val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		Value::String(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            arg1 = Value::String(gc.manage(val1.repeat(val2 as usize), &NO_GC));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {},
            }
            Err(format!(
		"*: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_divide
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String> {
            let arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            if val2 <= 0 {
				return Err(String::from("division by zero"));
                            }
                            arg1 = Value::Int(val1 / val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(val1) => {
                    match arg2 {
			Value::Float(val2) => {
                            if val2 <= OrderedFloat(0.0) {
				return Err(String::from("division by zero"));
                            }
                            arg1 = Value::Float(val1 / val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {}, 
            }
            Err(format!(
		"/: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_modulo
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String> {
            let arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            if val2 <= 0 {
				return Err(String::from("modulo by zero"));
                            }
                            arg1 = Value::Int(val1 / val2);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		}, 
		_ => {}, 
            }
            Err(format!(
		"/: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_power
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String> {
            let arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(val1) => {
                    match arg2 {
			Value::Int(val2) => {
                            arg1 = Value::Int(val1.pow(val2 as u32));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(val1) => {
                    match arg2 {
			Value::Float(val2) => {
                            arg1 = Value::Float(val1.pow(val2));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {}, 
            }
            Err(format!(
		"**: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    // compound assignment math operators
    fn execute_add_assign
	(
            &mut self,
            gc: &Gc,
	) -> Result<Value, String>  {
            let mut arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            *val1 += *val2;
                            arg1 = Value::Int(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		Value::Float(ref mut val1) => {
                    match arg2 {
			Value::Float(ref mut val2) => {
                            *val1 += *val2;
                            arg1 = Value::Float(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		Value::String(ref mut val1) => {
                    match arg2 {
			Value::String(ref mut val2) => { 
                            **val1 = format!("{}{}", **val1, **val2); 
                            arg1 = Value::String(gc.manage(val1.to_string(), &NO_GC));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {},
            }
            Err(format!(
		"+=: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_subtract_assign
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String>  {
            let mut arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            *val1 -= *val2;
                            arg1 = Value::Int(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(ref mut val1) => {
                    match arg2 {
			Value::Float(ref mut val2) => {
                            *val1 -= *val2;
                            arg1 = Value::Float(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {}, 
            }
            Err(format!(
		"-=: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_multiply_assign
	(
            &mut self,
            gc: &Gc,
	) -> Result<Value, String> {
            let mut arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            *val1 *= *val2;
                            arg1 = Value::Int(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(ref mut val1) => {
                    match arg2 {
			Value::Float(val2) => {
                            *val1 *= *val2;
                            arg1 = Value::Float(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		Value::String(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            **val1 = val1.repeat(*val2 as usize); 
                            arg1 = Value::String(gc.manage(val1.clone().to_string(), &NO_GC));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {},
            }
            Err(format!(
		"*=: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_divide_assign
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String> {
            let mut arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            if *val2 <= 0 {
				return Err(String::from("division by zero"));
                            }
                            *val1 /= *val2;
                            arg1 = Value::Int(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(ref mut val1) => {
                    match arg2 {
			Value::Float(ref mut val2) => {
                            if *val2 <= OrderedFloat(0.0) {
				return Err(String::from("division by zero"));
                            }
                            *val1 /= *val2;
                            arg1 = Value::Float(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {}, 
            }
            Err(format!(
		"/=: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_modulo_assign
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String> {
            let mut arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            if *val2 <= 0 {
				return Err(String::from("modulo by zero"));
                            }
                            *val1 %= *val2;
                            arg1 = Value::Int(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		}, 
		_ => {}, 
            }
            Err(format!(
		"%=: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute_power_assign
	(
            &mut self,
            _gc: &Gc,
	) -> Result<Value, String> {
            let mut arg2 = self.stack.pop().unwrap();
            let mut arg1 = self.stack.pop().unwrap();

            match arg1 {
		Value::Int(ref mut val1) => {
                    match arg2 {
			Value::Int(ref mut val2) => {
                            arg1 = Value::Int(val1.pow(val1.pow(*val2 as u32) as u32));
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {}, 
                    }
		},
		Value::Float(ref mut val1) => {
                    match arg2 {
			Value::Float(ref mut val2) => {
                            *val1 = val1.pow(*val2);
                            arg1 = Value::Float(*val1);
                            self.stack.push(arg1.clone());
                            return Ok(arg1);
			},
			_ => {},
                    }
		},
		_ => {}, 
            }
            Err(format!(
		"**=: invalid operands {} and {}",
		arg1.value_type(),
		arg2.value_type()))
	}

    fn execute
	(
            &mut self,
            root: &Option<Box<Ast>>,
            gc: &Gc,
            debug: bool,
            compile: bool,
	) -> Result<Value, InterpretErrorType>
    {
        self.calls.clear();
        self.stack.clear();
        let closure = self.parse_ast(root, gc, compile);
        if compile && closure.is_some() {
            return Ok(Value::Nil);
        }
        
        if let Some(c) = closure {
            self.main_fun = closure;
            
            // push main function onto the stack
            self.stack.push(Value::Closure(c));
            
            self.push_call_frame(0usize);
            let result = self.call_closure(0usize, gc, debug);
            match result {
                Ok(val) => {
                    return Ok(val);
                },
                Err(e) => {
                    println!("\n\nRuntime error.");
                    self.print_backtrace();
                    println!("Error: {}", e);
                    return Err(InterpretErrorType::RuntimeError);
                }
            }
        } else {
            return Err(InterpretErrorType::CompileTimeError);
        } 
    }

    pub(crate) fn run
	(
            &mut self, 
            root: &Option<Box<Ast>>,
            gc: &Gc,
            debug: bool,
            compile: bool,
	) -> Result<Value, InterpretErrorType> {
            self.execute(root, gc, debug, compile)
	}
    
    pub fn print_backtrace(&self) {
        println!("{}", self.calls.get_backtrace());
    }
}

struct AstParser {
    error_count: u8,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,
}

impl AstParser {
    fn new() -> Self {
        Self {
            error_count: 0u8,
            in_panic_mode: false,
            in_panic_lock_mode: false,
        }
    }

    fn parse
	(
            &mut self,
            ast: &Option<Box<Ast>>,
            vm: &mut VM,
            gc: &Gc,
	    compile: bool,
	) -> Option<Managed<Closure>>
    {
        let vm_ptr : *mut VM = ptr::from_mut(vm);
	if !compile {
            self.code_gen(ast, vm_ptr, gc)
	} else {
	    //self.code_gen_c(ast, vm_ptr, gc)
	    None
	} 
    }

    fn error
	(
            &mut self,
            _ast: &Ast,
            err_message: String
	)
    {
        /*if self.in_panic_mode || self.in_panic_lock_mode {
        return;
    }*/ 
        
        /*eprint!("{}:{}:{}: error", ast.file_name(), ast.line(), ast.col());
        eprint!(" at '{}'", ast.token().lexeme);

        eprintln!(": {}", err_message);
        eprintln!("");*/

        eprintln!("error: {}", err_message);

        self.error_count += 1;
	
        if self.error_count >= 20 {
            if !self.in_panic_mode {
                eprintln!("error: bailing out; too many errors");
                self.in_panic_lock_mode = true;
            }
        }
        self.in_panic_mode = true;
    }

    fn begin_function
	(
            &mut self,
            _ast: &mut Ast,
            _vm: *mut VM,
            _gc: &Gc,
	)
    {
    }

    #[allow(unused_variables)]
    fn code_gen_param<'a>
	(
            &mut self,
            ast: &Ast,
            vm: *mut VM,
            gc: &Gc,
	) -> bool
    {
        match *ast {
            Ast::Variable{ref name, ..} => {
                if let Some(ref mut last) = unsafe{(*vm).compiler.fc_vec.last_mut()}
                {
                    let var =
                        unsafe {
                            (*vm).compiler.add_local_var(self, ast, gc, name, None, true)
                        };
                    match var {
                        Ok(var_info) => {
                            unsafe {
                                (*vm).compiler.define_var(vm, last, 0)
                            };
                            return true;
                        },
                        Err(e) => {
                            return false;
                        },
                    }
                }
            },
            _ => {
                self.error(ast, "function parameter should be a variable".to_string());
                return false;
            }
        }
        false
    } 
    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen<'a>
	(
            &mut self,
            ast: &Option<Box<Ast>>,
            vm: *mut VM,
            gc: &Gc,
	) -> Option<Managed<Closure>>
    {
        let mut result: Option<Managed<Closure>> = None;
        if let Some(ref node) = ast {
            match **node {
                Ast::FunDecl {
		    ref name,
                    ref fun_ty,
                    ref params,
                    ref listing,
                    ref closure} => {
                    
                    let ty = fun_ty.clone();
                    match ty {
                        FunctionType::TopLevel => {
                            unsafe{(*vm).compiler.push_main(self, &node, gc)};
                            unsafe{(*vm).compiler.fc_vec.last_mut().unwrap().closure = Some(gc.manage(closure.clone(), &NO_GC))};
                        },
                        _ => {
                            unsafe {(*vm).compiler.push_function(self, &node, gc)};
                            unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure = Some(gc.manage(closure.clone(), &NO_GC))};
                        },
                    }

                    result = unsafe{(*vm).compiler.fc_vec.last_mut().unwrap().closure.clone()};
                    
                    if ty != FunctionType::TopLevel {
                        unsafe {(*vm).compiler.begin_scope()};
                    }
                    
                    for i in params {
                        let ret = self.code_gen_param(&i.clone(), vm, gc);
                        if !ret {
                            result = None;
                        }
                    }
		    
                    
                    for i in listing {
                        println!("here");
                        let ret = self.code_gen(&mut Some(i.clone()), vm, gc);
                        println!("ret = {:#?}", ret);
                        if ret.is_none() {
                            result = None;
                        } 
                    }
                    
                    if ty != FunctionType::TopLevel {
                        unsafe {(*vm).compiler.end_scope()};
                    }

                    if self.error_count == 0 {
                        if ty == FunctionType::NamedClosure ||
                            ty == FunctionType::AnonClosure
                        { 
                            let fc =
                                unsafe {
                                    &mut (*vm).compiler.fc_vec.last_mut().unwrap()
                                };
                            
                            let prev_fc = 
                                unsafe {
                                    &mut (&mut(*vm).compiler.fc_vec)[(*vm).compiler.fc_vec.len() - 2]
                                };
                            if let Some(closure) = fc.closure {
                                let constant = prev_fc.get_constant(&Value::Closure(closure));
                                prev_fc.emit_word_op(OpCode::LoadValue, constant as usize);
                            } else {
                                panic!("internal closure doesn't exist");
                            }
                            
                            if ty == FunctionType::NamedClosure {
                                let last = unsafe {
                                    (*vm).compiler.fc_vec.last_mut().unwrap()
                                };
                                let scope = unsafe {
                                    (*vm).compiler.get_scope_depth()
                                };
                                let var: Result<VarInfo, VarError> = 
				    Err(VarError::new(
					name.clone(),
					VarErrorType::Undefined,
				    ));
				let fun = unsafe {
				    (*vm).compiler.add_var(
                                        vm, self, &node, gc, name, Some(params.len()), true)
				};
				
                                match fun {
				    Ok(var_info) => {
                                        println!("here!");
					unsafe {
					    (*vm).compiler.define_var(vm, prev_fc, var_info.index)
					};
				    },
				    _ => {},
				}  
			    }                
                        }
			if result.is_some() { 
			    result = self.code_gen_return(ast, vm, gc);
			}
			unsafe{(*vm).compiler.fc_vec.pop()};
                        if ty == FunctionType::TopLevel && result.is_none() {
                            if self.error_count > 1 {
                                println!("Found {} errors.", self.error_count);
                            } else if self.error_count == 1 {
                                println!("Found 1 error.");
                            } else {
                                panic!("error count should not be zero");
                            }
                        }
			return result;
		    } else {
                        if ty == FunctionType::TopLevel {
                            if self.error_count > 1 {
                                println!("Found {} errors.", self.error_count);
                            } else if self.error_count == 1 {
                                println!("Found 1 error.");
                            } else {
                                panic!("error count should not be zero");
                            }
                        }
                        return None;
                    }
                },
                Ast::LetDecl{
		    ref name,
                    ref expr,
		} => {
                    let last = unsafe {
                        (*vm).compiler.fc_vec.last_mut().unwrap()
                    };
                    let scope = unsafe {
                        (*vm).compiler.get_scope_depth()
                    };
                    let mut var: Result<VarInfo, VarError> =
                        Err(VarError::new(
                            "".to_string(),
                            VarErrorType::Undefined,
			));
                    
                    let mut arity: Option<usize> = None;
                    let is_function = match &**expr {
                        Ast::FunDecl{
			    ref name,
                            ref fun_ty,
                            ref params, 
                            ref listing,
                            ref closure
			} => {
                            arity = Some(params.len());
                            true
                        },
                        _ => { false },
                    };

                    var = unsafe {(*vm).compiler.add_var(vm, self, &node, gc, name, arity, true)};
                    if self.code_gen(&mut Some(expr.clone()), vm, gc).is_none() {
                        return None;
                    }
                    
                    match var {
                        Ok(var_info) => {
                            unsafe{(*vm).compiler.define_var(vm, last, var_info.index)};
                            return last.closure;
                        },
                        Err(var_error) => {
                            None
                        }
                    }
                },
                Ast::ReturnStmt{ref expr, ..} => {
                    let last = unsafe {
                        (*vm).compiler.fc_vec.last_mut().unwrap()
                    };
                    
                    last.closure
                },
                Ast::Block {
		    ref listing
		} => { 
                    for i in listing {
                        self.code_gen(&mut Some(i.clone()), vm, gc);
                    }
                    unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure}
                },
                Ast::ExprStmt{
		    ref expr
		} => {
                    self.code_gen(&mut Some(expr.clone()), vm, gc)
                },
                Ast::Value {ref val, ..} => {
                    if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
                        match val {
                            Value::Int(i) => {
                                #[cfg(not(feature = "reg_mach"))]
                                if *i >= 0 && *i <= 10 {
                                    last.emit_op(OpCode::from(u8::from(OpCode::Load0) + (*i as u8)));
                                } else {
                                    let constant = last.get_constant(val);
                                    last.emit_word_op(OpCode::LoadValue, constant as usize);
                                }

                                #[cfg(feature = "reg_mach")]
                                if *i >= -32768 && *i <= 32767 {
                                    last.emit_op(OpCode::LoadI16(self.curr_reg(), *i as i16));
                                } else {
                                    last.emit_op(OpCode::LoadReg(self.curr_reg(), self.next_avail_reg()));
                                }
                                return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone();
                            },
                            Value::Float(_) => {
                                let constant = last.get_constant(val);
                                last.emit_word_op(OpCode::LoadValue, constant as usize);
                                return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
                            },
                            Value::Bool(b) => {
                                if *b == true {
                                    last.emit_op(OpCode::LoadTrue);
                                } else {
                                    last.emit_op(OpCode::LoadFalse);
                                }
                                return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
                            },
                            Value::Nil => {
                                last.emit_op(OpCode::LoadNil);
                                return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
                            },
                            Value::String(_) => {
                                let constant = last.get_constant(val);
                                last.emit_word_op(OpCode::LoadValue, constant as usize);
                                return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
                            },
                            _ => {
                                self.error(&node, "value is not a valid constant".to_string());
                                return None; 
                            }
                        }
                    } else {
                        eprintln!("function compiler stack is empty");
                        return None;
                    }
                },
		Ast::Variable{
		    ref name,
		    ref arity,
		} => {
                    let var = unsafe {
                        (*vm).compiler.resolve_var(self, vm, &node, gc, name, None, true)
                    };
                    match var {
                        Err(e) => {
                            if e.error_type == VarErrorType::Undefined {
                                self.error(&node, format!("variable {} is undefined", e.name));
                            }
                            return None;
                        }
                        Ok(v) => {
                            if let Some(last) = 
                                unsafe {(*vm).compiler.fc_vec.last_mut()}
                            {
                                self.emit_var_load(last, v);
                            };
                        },
                    };
                    
                    unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure}
                },
                Ast::Add(..) |
                Ast::Subtract(..) |
                Ast::Multiply(..) |
                Ast::Divide(..) |
                Ast::Modulo(..) |
                Ast::Power(..) => {
                    self.emit_math_op(&mut Some(node.clone()), vm, gc);
                    unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone()
                },
		Ast::Assign(
		    ref left,
		    ref right,
		) => {
		    self.code_gen(
			right, vm, gc,
		    );
		    if let Some(l) = left {
			match **l {
			    Ast::Variable {
                                ref name,
				ref arity,
			    } => {
				let var = unsafe{
				    (*vm).compiler.resolve_var(
					self, vm, &**l, gc, name, *arity, true
				    )
				};
                                match var {
                                    Ok(v) => {
                                        if let Some(last) = unsafe {
                                            (*vm).compiler.fc_vec.last_mut()
                                        } {
                                            last.emit_word_op(OpCode::StoreVar, v.index as usize);
                                            last.closure
                                        } else {
                                            panic!("function compiler stack is empty");
                                        }
                                    },
                                    Err(e) => {
                                        if e.error_type == VarErrorType::Undefined {
                                            self.error(&**l, 
                                                       format!("variable {} is undefined", e.name)
                                            );
                                        }
                                        None
                                    },
                                }
			    },
			    _ => {
				self.error(&**l, "only variables and fields can be on left side of assignment".to_string());
				None
			    },
			}
                    } else {
			panic!("left node of assignment doesn't exist");
                    }
		},
                Ast::Call(
		    ref recv,
		    ref args,
		) => {
                    self.code_gen_call(vm, &*ast.clone().unwrap(), gc, &recv, args)
                },
                _ => {
                    self.error(&node, format!("unrecognized node type {}", *node));
                    None
                },
            }
        } else {
            eprintln!("ast node doesn't exist");
            None
        }
    }
    
    fn code_gen_call
	(
            &mut self,
            vm: *mut VM,
            ast: &Ast,
            gc: &Gc,
            recv: &Option<Box<Ast>>,
            args: &Vec<Box<Ast>>,
	) -> Option<Managed<Closure>>
    {
        if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
            match *recv.clone().unwrap() {
                Ast::Variable {
                    ref name,
                    ref arity,
                } => {
                    let var = unsafe {(*vm).compiler.resolve_var(self, vm, ast, gc, name, Some(args.len()), false)};
                    match var {
                        Ok(v) => {
                            self.emit_var_load(last, v);
                        },
                        Err(e) => {
                            if e.error_type == VarErrorType::Undefined {
                                self.error(&*recv.clone().unwrap(), format!("variable {} is undefined", e.name));
                            }
                            return None;
                        },
                    }
                },
                _ => {},
            }
            for i in &*args {
                if self.code_gen(&mut Some(i.clone()), vm, gc).is_none() {
                    return None;
                }
            }
            #[cfg(not(feature = "reg_mach"))]
            last.emit_byte_op(OpCode::Call, args.len() as u8);       
            
            #[cfg(feature = "reg_mach")]
            last.emit_op(OpCode::Call, args.len() as u8);

            unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure.clone()}
        } else {
            panic!("no function compilers left on stack");
        }
    }
    
    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen_return<'a>
	(
            &mut self,
            ast: &Option<Box<Ast>>,
            vm: *mut VM,
            gc: &Gc,
	) -> Option<Managed<Closure>>
    {
        if let Some(node) = ast {
            match **node {
                Ast::FunDecl {
		    ref name,
                    ref fun_ty,
                    ref params,
                    ref listing,
                    ref closure} => {

                    if self.error_count == 0 {
                        if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
                            if listing.is_empty() {
                                last.emit_op(OpCode::LoadNil);
                                last.emit_op(OpCode::Return);
                            } else {
                                if let Some(l) = listing.last() { 
                                    match **l {
                                        Ast::ExprStmt{..} => {
                                            last.emit_op(OpCode::Return);
                                        },
                                        Ast::ReturnStmt{ref expr, ..} => {
                                            if let Some(e) = expr {
                                                self.code_gen(&Some::<Box<Ast>>(e.clone()), vm, gc);
                                            } else {
                                                last.emit_op(OpCode::LoadNil);
                                            }
                                            last.emit_op(OpCode::Return);
                                        },
                                        _ => {},
                                    }
                                }
                            } 
                        }
                        let c = unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure};
                        if c.is_some() {
                            c.unwrap().get_core().finish(
                                unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().constant_arr.clone()},
                                unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().code.clone()}
                            );
                            return c;
                        }
                        return None;
                    } else {
                        return None;
                    }
                },
                _ => {
                    unreachable!();
                }
            }
        }
        None
    }

    fn emit_var_load(&mut self, fc: &mut FunCompiler, var: VarInfo) {
        match var.scope_type {
            ScopeType::Ns => {
                fc.emit_word_op(OpCode::LoadVar, var.index as usize);
            },
            ScopeType::Local => {
                if var.index < 9 {
                    fc.emit_op(OpCode::from(u8::from(OpCode::LoadLocal0) + var.index as u8));
                } else {
                    fc.emit_byte_op(OpCode::LoadLocal, var.index as u8);
                }
            },
            ScopeType::Upvalue => {
                fc.emit_byte_op(OpCode::LoadUpvalue, var.index as u8);
            }
        }
    }

    fn emit_math_op<'a>
	(
            &mut self,
            ast: &mut Option<Box<Ast>>,
            vm: *mut VM,
            gc: &Gc,
	) -> Option<Managed<Closure>>
    {
        if let Some(node) = ast {
            if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
                match **node {
                    Ast::Add(ref left, ref right) => {
                        self.code_gen(&mut left.clone(), vm, gc);
                        self.code_gen(&mut right.clone(), vm, gc);
                        last.emit_math_op(MathOp::Add);
                        last.closure
                    },
                    Ast::Subtract(ref left, ref right) => {
                        self.code_gen(&mut left.clone(), vm, gc);
                        self.code_gen(&mut right.clone(), vm, gc);
                        last.emit_math_op(MathOp::Subtract);
                        last.closure
                    },
                    Ast::Multiply(ref left, ref right) => {
                        self.code_gen(&mut left.clone(), vm, gc);
                        self.code_gen(&mut right.clone(), vm, gc);
                        last.emit_math_op(MathOp::Multiply);
                        last.closure
                    },
                    Ast::Divide(ref left, ref right) => {
                        self.code_gen(&mut left.clone(), vm, gc);
                        self.code_gen(&mut right.clone(), vm, gc);
                        last.emit_math_op(MathOp::Divide);
                        last.closure
                    },
                    Ast::Modulo(ref left, ref right) => {
                        self.code_gen(&mut left.clone(), vm, gc);
                        self.code_gen(&mut right.clone(), vm, gc);
                        last.emit_math_op(MathOp::Modulo);
                        last.closure
                    },
                    Ast::Power(ref left, ref right, ..) => {
                        self.code_gen(&mut left.clone(), vm, gc);
                        self.code_gen(&mut right.clone(), vm, gc);
                        last.emit_math_op(MathOp::Power);
                        last.closure
                    },
                    _ => { unreachable!(); }
                }
            } else {
                panic!("function compiler stack is empty");
            }
        } else {
            panic!("ast node doesn't exist");
        }
    }


    // C code generator

    fn code_gen_param_c<'a>
	(
	    &mut self,
	    ast: &Ast,
	    vm: *mut VM,
	    gc: &Gc,
	) -> bool
    {
	match *ast {
	    Ast::Variable{ref name, ..} => {
		if let Some(ref mut last) = unsafe{(*vm).compiler.fc_vec.last_mut()}
		{
		    let var =
			unsafe {
			    (*vm).compiler.add_local_var(self, ast, gc, name, None, true)
			};
		    match var {
			Ok(_) => {
			    unsafe {
				(*vm).compiler.define_var(vm, last, 0)
			    };
			    return true;
			},
			Err(_) => {
			    return false;
			},
		    }
		}
	    },
	    _ => {
		self.error(ast, "function parameter should be a variable".to_string());
		return false;
	    }
	}
	false
    }

    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen_c<'a>
	(
	    &mut self,
	    ast: &Option<Box<Ast>>,
	    vm: *mut VM,
	    gc: &Gc,
	) -> Option<Managed<Closure>>
    {
	let mut result: Option<Managed<Closure>> = None;
	if let Some(ref node) = ast {
	    match **node {
		Ast::FunDecl {
		    ref name,
		    ref fun_ty,
		    ref params,
		    ref listing,
		    ref closure
		} => {
		    
		    let ty = fun_ty.clone();
		    match ty {
			FunctionType::TopLevel => {
			    
			    unsafe{(*vm).compiler.push_main(self, &node, gc)};
			    unsafe{(*vm).compiler.fc_vec.last_mut().unwrap().closure = Some(gc.manage(closure.clone(), &NO_GC))};
			},
			_ => {
			    unsafe {(*vm).compiler.push_function(self, &node, gc)};
			    unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure = Some(gc.manage(closure.clone(), &NO_GC))};
			},
		    }

		    result = unsafe{(*vm).compiler.fc_vec.last_mut().unwrap().closure.clone()};
		    
		    if ty != FunctionType::TopLevel {
			unsafe {(*vm).compiler.begin_scope()};
		    }
		    
		    let mut num_params = 0;
		    for i in params {
			let ret = self.code_gen_param_c(&i.clone(), vm, gc);
			num_params += 1;
			if !ret {
			    result = None;
			}
		    }
		    
		    
		    for i in listing {
			let ret = self.code_gen_c(&mut Some(i.clone()), vm, gc);
			if ret.is_none() {
			    result = None;
			}
		    }
		    
		    if ty != FunctionType::TopLevel {
			unsafe {(*vm).compiler.end_scope()};
		    }

		    if self.error_count == 0 {
			if ty == FunctionType::NamedClosure ||
			    ty == FunctionType::AnonClosure
			{
			    if let Some(last) =
				unsafe{(*vm).compiler.fc_vec.last_mut()}
			    {
				if let Some(closure) = last.closure {
				    let constant = last.get_constant(
					&Value::Closure(closure)
				    );
				    last.emit_word_op(
					OpCode::LoadValue, constant as usize
				    );
				}
			    }
			    let fc =
				unsafe {
				    &mut (*vm).compiler.fc_vec.last_mut().unwrap()
				};
			    
			    let prev_fc = 
				unsafe {
				    &mut (&mut(*vm).compiler.fc_vec)[(*vm).compiler.fc_vec.len() - 2]
				};
			    if let Some(closure) = fc.closure {
				let constant = prev_fc.get_constant(&Value::Closure(closure));
				prev_fc.emit_word_op(OpCode::LoadValue, constant as usize);
			    } else {
				panic!("internal closure doesn't exist");
			    }
			    if ty == FunctionType::NamedClosure {
				let last = unsafe {
				    (*vm).compiler.fc_vec.last_mut().unwrap()
				};
				let scope = unsafe {
				    (*vm).compiler.get_scope_depth()
				};
				let mut var: Result<VarInfo, VarError> = 
				    Err(VarError::new(
					name.clone(),
					VarErrorType::Undefined,
				    ));
				println!("scope = {}", scope);
				if scope == 0 {
				    var = unsafe {
					(*vm).compiler.add_ns_var(vm, self, &node, name, 
								  Some(num_params), true
					)
				    };
				    match var {
					Ok(var_info) => {
					    unsafe {
						(*vm).compiler.define_var(vm, prev_fc, var_info.index);
					    };
					},
					Err(_) => {
					    result = None;
					},
				    };
				} else {
				    var  = last.declare_local(self, &node, gc, name, 
							      Some(num_params), true
				    );
				    match var {
					Ok(var_info) => {
					    unsafe {
						(*vm).compiler.define_var(vm, fc, 0)
					    };
					},
					Err(_) => {
					    result = None;
					},
				    } 
				}
				
			    }
			    
			}
			if result.is_some() { 
			    result = self.code_gen_return_c(ast, vm, gc);
			}
			unsafe{(*vm).compiler.fc_vec.pop()};
			return result;
		    } else {
			return None;
		    }
		},
		Ast::LetDecl{
		    ref name,
		    ref expr
		} => {

		    let last = unsafe {
			(*vm).compiler.fc_vec.last_mut().unwrap()
		    };
		    let scope = unsafe {
			(*vm).compiler.get_scope_depth()
		    };
		    let mut var: Result<VarInfo, VarError> =
			Err(VarError::new(
			    "".to_string(),
			    VarErrorType::Undefined,
			));
		    println!("scope = {}", scope);
		    if scope == 0 {
			var = unsafe {(*vm).compiler.add_ns_var(vm, self, &node, name, None, true)};
			if self.code_gen_c(&mut Some(expr.clone()), vm, gc).is_none() {
			    return None;
			} 
		    } else {
			var = last.declare_local(self, node, gc, &name, None, true);
			if self.code_gen_c(&mut Some(expr.clone()), vm, gc).is_none() {
			    return None;
			}
		    }
		    
		    match var {
			Ok(var_info) => {
			    unsafe{(*vm).compiler.define_var(vm, last, var_info.index)};
			    return last.closure;
			},
			Err(var_error) => {
			    None
			}
		    }
		}
		Ast::ReturnStmt{ref expr} => {
		    let last = unsafe {
			(*vm).compiler.fc_vec.last_mut().unwrap()
		    };
		    
		    last.closure
		},
		Ast::Block {
		    ref listing
		} => { 
		    for i in listing {
			self.code_gen_c(&mut Some(i.clone()), vm, gc);
		    }
		    unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure}
		},
		Ast::ExprStmt{
		    ref expr
		} => {
		    self.code_gen_c(&mut Some(expr.clone()), vm, gc)
		},
		Ast::Value {
		    ref val
		} => {
		    if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
			match val {
			    Value::Int(i) => {
				if *i >= 0 && *i <= 10 {
				    last.emit_op(OpCode::from(u8::from(OpCode::Load0) + (*i as u8)));
				} else {
				    let constant = last.get_constant(val);
				    last.emit_word_op(OpCode::LoadValue, constant as usize);
				}
				return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone();
			    },
			    Value::Float(_) => {
				let constant = last.get_constant(val);
				last.emit_word_op(OpCode::LoadValue, constant as usize);
				return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
			    },
			    Value::Bool(b) => {
				if *b == true {
				    last.emit_op(OpCode::LoadTrue);
				} else {
				    last.emit_op(OpCode::LoadFalse);
				}
				return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
			    },
			    Value::Nil => {
				last.emit_op(OpCode::LoadNil);
				return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
			    },
			    Value::String(_) => {
				let constant = last.get_constant(val);
				last.emit_word_op(OpCode::LoadValue, constant as usize);
				return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone(); 
			    },
			    _ => {
				self.error(&node, "value is not a valid constant".to_string());
				return None; 
			    }
			}
		    } else {
			eprintln!("function compiler stack is empty");
			return None;
		    }
		},
		Ast::Variable{
		    ref name,
		    ref arity,
		}  => {
		    let var = unsafe {
			(*vm).compiler.resolve_var(self, vm, &node, gc, name, None, true)
		    };
		    match var {
			Err(e) => {
			    self.error(&node, format!("variable {} is undefined", e.name));
			    return None;
			}
			Ok(_) => {},
		    };
		    
		    unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure}
		},
		Ast::Add(..) |
		Ast::Subtract(..) |
		Ast::Multiply(..) |
		Ast::Divide(..) |
		Ast::Modulo(..) |
		Ast::Power(..) => {
		    self.emit_math_op(&mut Some(node.clone()), vm, gc);
		    unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone()
		},
		Ast::Call(ref recv, ref args) => {
		    if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
			if self.code_gen_c(&mut recv.clone(), vm, gc).is_none() {
			    return None;
			}
			for i in &*args {
			    self.code_gen_c(&mut Some(i.clone()), vm, gc);
			} 
			last.emit_byte_op(OpCode::Call, args.len() as u8);
			println!("closure = {:#?}", unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone());
			return unsafe{(*vm).compiler.fc_vec.last_mut()}.unwrap().closure.clone()
		    } else {
			panic!("no function compilers left on stack");
		    }
		},
		_ => {
		    self.error(&node, format!("unrecognized node type {}", *node));
		    return None;
		}
	    }
	} else {
	    eprintln!("ast node doesn't exist");
	    None
	}
    }

    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen_return_c<'a>
	(
	    &mut self,
	    ast: &Option<Box<Ast>>,
	    vm: *mut VM,
	    gc: &Gc,
	) -> Option<Managed<Closure>>
    {
	if let Some(node) = ast {
	    match **node {
		Ast::FunDecl {
		    ref name,
		    ref fun_ty,
		    ref params,
		    ref listing,
		    ref closure} => {

		    if self.error_count == 0 {
			if let Some(last) = unsafe{(*vm).compiler.fc_vec.last_mut()} {
			    if listing.is_empty() {
				last.emit_op(OpCode::LoadNil);
				last.emit_op(OpCode::Return);
			    } else {
				if let Some(l) = listing.last() { 
				    match **l {
					Ast::ExprStmt{..} => {
					    last.emit_op(OpCode::Return);
					},
					Ast::ReturnStmt{
					    ref expr,
					} => {
					    if let Some(e) = expr {
						self.code_gen(&Some::<Box<Ast>>(e.clone()), vm, gc);
					    } else {
						last.emit_op(OpCode::LoadNil);
					    }
					    last.emit_op(OpCode::Return);
					},
					_ => {},
				    }
				}
			    } 
			}
			let c = unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().closure};
			if c.is_some() {
			    c.unwrap().get_core().finish(
				unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().constant_arr.clone()},
				unsafe {(*vm).compiler.fc_vec.last_mut().unwrap().code.clone()}
			    );
			    return c;
			}
			return None;
		    } else {
			return None;
		    }
		},
		_ => {
		    unreachable!();
		}
	    }
	}
	None
    }
}



#[derive(Clone)]
struct Local {
    name: String,
    scope: usize,
    upvalue: bool,
    defined: bool,
}

impl Local {
    fn new(name: String, scope: usize, upvalue: bool) -> Self {
        Self {
            name,
            scope,
            upvalue,
            defined: false,
        }
    }
}

#[derive(Clone)]
struct Upvalue {
    local: bool,
    index: usize,
}

impl Upvalue {
    fn new(local: bool, index: usize) -> Self {
        Self {
            local,
            index,
        }
    }
}

#[derive(Clone)]
struct FunCompiler {
    name: String,
    ty: FunctionType,
    scope: usize,
    arity: usize,

    locals: Vec<Local>,
    upvalues: Vec<Upvalue>,
    code: Vec<u8>,
    constants: HashMap<Value, usize>,
    constant_arr: Vec<Value>,
    local_fun_symtab: HashMap<String, VarInfo>,
    closure: Option<Managed<Closure>>,
}

impl FunCompiler {
    fn new_main
	(
            parser: &mut AstParser,
            ast: &Ast,
            gc: &Gc,
	) -> Self 
    {
        Self::function_setup(parser, ast, gc, "<main>", 0, 0, FunctionType::TopLevel)
    }

    fn new_named_closure
	(
            parser: &mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &str, 
            arity: usize, 
            scope: usize,
	) -> Self
    {
        
        Self::function_setup(parser, ast, gc, name, arity, scope, FunctionType::NamedClosure)
    }

    fn new_anon_closure
	(
            parser: &mut AstParser,
            ast: &Ast,
            gc: &Gc,
            arity: usize,
            scope: usize,
	) -> Self
    {
        Self::function_setup(parser, ast, gc, "", arity, scope, FunctionType::AnonClosure)
    }

    /*fn new_function
    (
    parser: &mut AstParser,
    name: &str,
    arity: usize
) -> Self
    {
}
    
    fn new_constructor
    (
    parser: &mut AstParser,
    arity: usize
) -> Self
    {
}
    
    fn new_method
    (
    parser: &mut AstParser,
    name: &str,
    arity: usize
) -> Self 
    {
}

     */
    
    fn function_setup
	(
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &str,
            arity: usize,
            scope: usize,
            ty: FunctionType,
	) -> Self
    {
        let mut s = Self {
            name: name.to_string(),
            ty: ty,
            scope,
            arity: arity,
            locals: vec![],
            upvalues: vec![],
            code: vec![],
            constants: HashMap::new(),
            constant_arr: vec![],
            local_fun_symtab: HashMap::new(),
            closure: None,
            
        };
        if s.ty == FunctionType::Constructor || s.ty == FunctionType::Method {
            s.push_local(
                parser, ast, gc, "this".to_string(), 
                s.get_scope_depth(), false
            );
        } else {
            s.push_local(
                parser, ast, gc, "".to_string(), 
                s.get_scope_depth(), false
            );
        } 
        s
    }

    fn resolve_local_internal
	(
            &mut self,
            parser: *mut AstParser,
            ast: &Ast,
            _gc: &Gc,
            name: &String,
            arity: Option<usize>,
            do_print_err: bool,
	) -> Result<VarInfo, VarError>
    {
        for i in (0..self.locals.len()).rev() { 
            if self.locals[i].name == *name {
                if !self.locals[i].defined && do_print_err {
                    unsafe {
                        (*parser).error(ast, "self-initialization of local variable '".to_string() + &self.locals[i].name)
                    };
                    return Err(VarError::new(
                        self.locals[i].name.clone(),
                        VarErrorType::SelfInit,
                    )); 
                } else {
                    return Ok(VarInfo::new(
                        self.locals[i].name.clone(),
                        arity,
                        VarType::Var,
                        ScopeType::Local,
                        self.locals[i].scope,
                        i,
                        0,
                    ));
                }
            }
        }
        
        Err(VarError::new(
	    name.clone(),
	    VarErrorType::Undefined,
	))
    }

    fn resolve_upvalue_internal
	(
            &mut self,
            parser: *mut AstParser,
            vm: *mut VM,
            ast: &Ast,
            gc: &Gc, 
            name: &String,
            arity: Option<usize>,
            print_error: bool,
            mut iter: Rev<std::slice::IterMut<'_, FunCompiler>>,
	) -> Result<VarInfo, VarError>
    {
        let mut fc = iter.next();
        if let Some(ref mut fun_compiler) = fc {

            let local = fun_compiler.resolve_local_internal(parser, ast, gc, name, arity, true);
            match local {
                Ok(var_info) => {
                    if let Some(fun_compiler) = fc {
                        fun_compiler.locals[var_info.index].upvalue = true;
                        return fun_compiler.add_upvalue(parser, ast, gc, &name, arity, var_info.upvalue_local_index, true);
                    }
                },
                _ => {},
            }

            fc = iter.next();
            if fc.is_some() {
                if let Some(fun_compiler) = fc {
                    let upvalue = fun_compiler.resolve_upvalue_internal(parser, vm, ast, gc, name, arity, false, iter);
                    
                    match upvalue {
                        Ok(var_info) => {
                            if let Some(last) = unsafe {(*vm).compiler.clone()}.fc_vec.last_mut() {
                                return last.add_upvalue(parser, ast, gc, name, arity, var_info.upvalue_local_index, false);
                            }
                        },
                        Err(_) => {},
                    }
                }
            }
        }

        Err(VarError::new(
	    name.clone(),
	    VarErrorType::Undefined,
	))
    }

    fn add_upvalue
	(
            &mut self, 
            parser: *mut AstParser, 
            ast: &Ast,
            _gc: &Gc,
            name: &String,
            arity: Option<usize>,
            index: usize, 
            is_local: bool
	) -> Result<VarInfo, VarError>
    {
        for i in 0..self.locals.len() {
            if i == index && is_local {
                return Ok(VarInfo::new(
                    self.locals[i].name.clone(),
                    arity,
                    VarType::Var,
                    ScopeType::Upvalue,
                    self.locals[i].scope,
                    i,
                    0,
                ))
            }
        }

        if self.upvalues.len() >= MAX_UPVALUES {
            unsafe {(*parser).error(ast, "too many closure variables".to_string())};
            return Err(VarError::new(
                name.clone(),
                VarErrorType::TooMany,
            ));
        }

        let upvalue = Upvalue::new(is_local, index);
        self.upvalues.push(upvalue);

        Ok(VarInfo::new(
            self.locals[index].name.clone(),
            arity,
            VarType::Var,
            ScopeType::Upvalue,
            self.locals[index].scope,
            self.upvalues.len() - 1,
            index,
        ))
    } 

    fn push_local
	(
            &mut self,
            parser: *mut AstParser,
            ast: &Ast,
            _gc: &Gc,
            name: String,
            scope: usize,
            upvalue: bool
	) -> Option<&mut Local>
    {
        if self.locals.len() == MAX_LOCALS {
            unsafe {(*parser).error(ast, "too many locals in function".to_string())};
            return None;
        }
        self.locals.push(Local::new(name.to_string(), scope, upvalue));

        self.locals.last_mut()
    }

    fn declare_local
	(
            &mut self, 
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: Option<usize>,
            print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        let mut actual_name = "".to_string();
        if let Some(a) = arity {
            actual_name = format!("{}(#{})", name, a);
        } else {
            actual_name = name.to_string();
        }
        let mut iter = self.locals.iter_mut().rev();
        let mut scope = 0usize;
        loop { 
            if let Some(i) = iter.next() {
                if i.scope < self.scope {
                    scope = i.scope;
                    break;
                }
                if i.name == *actual_name {
                    if print_error {
                        unsafe {
                            (*parser).error(
                                ast, "local variable '".to_string() + &i.name + "' already exists"
                            )
                        };
                    }
                    return Err(VarError::new(
                        actual_name.clone(),
                        VarErrorType::AlreadyExists,
                    ));
                }
            } else {
                return Err(VarError::new(
                    actual_name.clone(),
                    VarErrorType::Undefined,
                ));
            }
        }
        self.push_local(parser, ast, gc, actual_name.clone(), scope, false);
        
        Ok(VarInfo::new(
            actual_name.clone(),
            arity,
            VarType::Var,
            ScopeType::Local,
            scope,
            self.locals.len() - 1,
            0,
        ))
    } 

    fn declare_local_function
	(
            &mut self,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: usize,
	    print_err: bool,
	) -> Result<VarInfo, VarError>
    {
        let fun_name = format!("{}(#{})", name, arity as u32);
        if self.local_fun_symtab.get(&fun_name).is_some() {
            unsafe {(*parser).error(ast, "local function '".to_string() + &fun_name + "' already exists")};
            Err(
                VarError::new(
                    fun_name,
                    VarErrorType::AlreadyExists,
                )
            )
        } else {
            let closure = self.closure.unwrap();
            let constant = self.get_constant(&Value::Closure(closure)); 
            let fun = VarInfo::new(
                fun_name.clone(),
                Some(arity),
                VarType::Var,
                ScopeType::Local,
                self.get_scope_depth(),
                constant,
                0,
            );
            self.local_fun_symtab.insert(
                fun_name.clone(),
                fun.clone(),
            );

            Ok(fun.clone())
        }
    }

    fn get_scope_depth(&self) -> usize {
        self.scope
    }

    fn begin_scope(&mut self) {
        self.scope += 1;
    }
    
    fn end_scope(&mut self) {
        if self.scope > 0 {
            self.scope -= 1;
        }
    }
    
    fn get_constant(&mut self, val: &Value) -> usize {
        let len = self.constants.len();

        let pair = self.constants.get_key_value(val);
        if pair.is_none() { 
            self.constants.insert(val.clone(), self.constants.len());
            self.constant_arr.push(val.clone());
        } else {
            return *pair.unwrap().1
        }
        return len;
    }

    fn emit_op(&mut self, opcode: OpCode) {
        #[cfg(feature = "debug_opcode")]
        println!("gen: {}: {}", self.closure.unwrap().get_core().get_name(), opcode);
        self.code.push(u8::from(opcode));
    }

    fn emit_byte_op(&mut self, opcode: OpCode, byte: u8) {
        #[cfg(feature = "debug_opcode")]
        println!("gen: {}: {} byte {}", 
		 self.closure.unwrap().get_core().get_name(), opcode, byte);
        self.code.push(u8::from(opcode));
        self.code.push(byte);
    }

    fn emit_word_op(&mut self, opcode: OpCode, word: usize) {
        #[cfg(feature = "debug_opcode")]
        println!("gen: {}: {} word {}", 
		 self.closure.unwrap().get_core().get_name(), opcode, word);
        self.code.push(u8::from(opcode)); 
        self.code.push(((word >> 8) & 0xff) as u8);
        self.code.push((word & 0xff) as u8);
    }

    fn emit_three_byte_op(&mut self, opcode: OpCode, three_byte: u32) {
        #[cfg(feature = "debug_opcode")]
        println!("gen: {}: {} three byte {}", 
		 self.closure.unwrap().get_core().get_name(), opcode, three_byte);
        self.code.push(u8::from(opcode));
        self.code.push(((three_byte >> 16) & 0xff) as u8);
        self.code.push(((three_byte >> 8) & 0xff) as u8);
        self.code.push((three_byte & 0xff) as u8);
    }

    fn emit_math_op
	(
            &mut self,
            op: MathOp,
	)
    {
        println!("gen: {} op {}", OpCode::MathOp, op);
        self.code.push(u8::from(OpCode::MathOp));
        self.code.push(u8::from(op));
    }
}

#[derive(Clone, PartialEq)]
enum VarType {
    Var,
    Function,
    Class,
    Const,
}

#[derive(Clone, PartialEq)]
enum ScopeType {
    Ns,
    Local,
    Upvalue,
}

#[derive(Clone, PartialEq)]
enum VarErrorType {
    Undefined,
    AlreadyExists,
    TooMany,
    SelfInit,
    IsFunction,
    Other,
}

#[derive(Clone)]
struct VarInfo {
    name: String,
    arity: Option<usize>,
    var_type: VarType,
    scope_type: ScopeType,
    scope: usize,
    index: usize,
    upvalue_local_index: usize,
}

impl VarInfo {
    fn new
	(
            name: String,
            arity: Option<usize>,
            var_type: VarType,
            scope_type: ScopeType,
            scope: usize,
            index: usize,
            upvalue_local_index: usize,
	) -> Self
    {
        Self {
            name,
            arity,
            var_type,
            scope_type,
            scope,
            index,
            upvalue_local_index,
        }
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_arity(&self) -> Option<usize> {
        self.arity
    }

    fn get_var_type(&self) -> VarType {
        self.var_type.clone()
    }

    fn get_scope_depth(&self) -> usize {
        self.scope
    }

    fn get_index(&self) -> usize {
        self.index
    }
}

#[derive(Clone)]
struct VarError {
    name: String,
    error_type: VarErrorType,
}

impl VarError {
    fn new(
        name: String,
        error_type: VarErrorType,
    ) -> Self
    {
        Self {
            name,
            error_type,
        }
    }

    fn get_name(&self) -> String{
        self.name.clone()
    }

    fn get_error_type(&self) -> VarErrorType {
        self.error_type.clone()
    }
}



#[derive(Clone)]
struct Compiler {
    var_symtab: HashMap<String, VarInfo>,
    fun_symtab: HashMap<String, VarInfo>,
    var_count: usize,
    fun_count: usize,
    class_symtab: HashMap<String, VarInfo>,
    fc_vec: Vec<FunCompiler>,
}

impl<'a> Compiler {
    fn new() -> Self {
        Self {
            var_symtab: HashMap::new(),
	    fun_symtab: HashMap::new(),
            var_count: 0usize,
            fun_count: 0usize,
	    class_symtab: HashMap::new(),
            fc_vec: vec![],
        }
    }

    fn push_main
	(
            &mut self,
            parser: &mut AstParser,
            ast: &Ast,
            gc: &Gc,
	)
    {
        if self.fc_vec.is_empty() {
            self.fc_vec.push(FunCompiler::new_main(parser, ast, gc));
        } else {
            self.fc_vec[0] = FunCompiler::new_main(parser, ast, gc);
        } 
    }

    fn push_function
	(
            &mut self,
            parser: &mut AstParser,
            ast: &Ast,
            gc: &Gc,
	)
    {
        if self.fc_vec.last_mut().is_some() {
            match *ast {
                Ast::FunDecl{ref name, 
                             ref fun_ty,  
                             ref closure, ..} => {
                    match fun_ty {
                        FunctionType::NamedClosure => {
                            self.fc_vec.push(
                                FunCompiler::new_named_closure(
                                    parser, ast, gc, name.as_str(), 
                                    closure.get_core().get_arity(),
                                    self.fc_vec.last().unwrap().get_scope_depth()
                                )
                            );
                        },
                        FunctionType::AnonClosure => {
                            self.fc_vec.push(
                                FunCompiler::new_anon_closure(
                                    parser, &mut ast.clone(), gc,
                                    closure.get_core().get_arity(), 
                                    self.fc_vec.last().unwrap().get_scope_depth()
                                )
                            );
                        },
                        _ => {
                            panic!("push function: invalid function type");
                        }
                    }
                },
                _ => {
                    panic!("push_function: invalid node type '{}'", ast);
                }
            }
        } else {
            panic!("function compiler stack is empty");
        }
    }
    fn begin_scope(&mut self) {
        if let Some(var) = self.fc_vec.last_mut() {
            var.begin_scope();
        }
    }

    fn end_scope(&mut self) {
        if let Some(x) = self.fc_vec.last_mut() {
            x.end_scope();
        }
    }

    fn get_local_var_scope_depth(&mut self, index: usize) -> Option<usize> {
        if let Some(x) = self.fc_vec.last_mut() {
            if let Some(y) = x.locals.get(index) {
                return Some(y.scope);
            } else {
                return None;
            }
        } else {
            None
        }
    }

    fn get_scope_depth(&mut self) -> usize {
        if let Some(x) = self.fc_vec.last_mut() {
            x.get_scope_depth()
        } else {
            0
        }
    }

    fn get_ns_var_count(&self) ->usize {
        self.var_count
    } 

    fn inc_ns_var_count(&mut self) {
        self.var_count += 1;
    }

    fn get_ns_fun_count(&self) -> usize {
	self.fun_count
    }
    
    fn inc_ns_fun_count(&mut self) {
        self.fun_count += 1;
    }

    fn add_function
	(
            &mut self,
            vm: *mut VM,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: usize,
	    print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        if name.starts_with(".") {
            self.add_ns_function(
                vm, parser, ast, gc, &name[1..name.len()].to_string(), arity, print_error
            )
        } else {
            if self.get_scope_depth() == 0 {
                self.add_ns_function(vm, parser, ast, gc, name, arity, print_error)
            } else {
                self.add_local_function(vm, parser, ast, gc, name, arity, print_error)
            }
        }
    }

    fn add_ns_function
	(
            &mut self,
            vm: *mut VM,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: usize,
	    print_error: bool,
	) -> Result<VarInfo, VarError>
    {

        let fun_name = format!("{}(#{})", name.clone(), arity);
        if let Some(fun) = self.var_symtab.get(&fun_name) {
	    if print_error {
		unsafe {(*parser).error(ast, format!("namespace function '{}' already exists", fun_name))};
	    }
            return Err(VarError::new(
                fun_name.clone(),
                VarErrorType::AlreadyExists,
            ));
        } else {
            let fun =
                VarInfo::new(
                    fun_name.to_string(),
                    Some(arity),
                    VarType::Function,
                    ScopeType::Ns,
                    self.get_scope_depth(),
                    self.get_ns_var_count(),
                    0,
                );
            self.fun_symtab.insert(
                fun_name.to_string(), 
                fun.clone(),
            );
            
            Ok(fun)
        }
    }

    fn add_local_function
	(
            &mut self,
            vm: *mut VM,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: usize,
	    print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        if let Some(last) = self.fc_vec.last_mut() {
            return last.declare_local_function(
                parser, ast, gc, name, arity, print_error,
            );
        } else {
            unreachable!();
        }
    }        

    fn add_var
	(
            &mut self,
            vm: *mut VM,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
	    arity: Option<usize>,
	    print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        if name.starts_with(".") {
            self.add_ns_var(vm, parser, ast, &name[1..name.len()].to_string(), arity, print_error)
        } else if self.get_scope_depth() == 0 {
            self.add_ns_var(vm, parser, ast, name, arity, print_error)
        } else {
            self.add_local_var(parser, ast, gc, name, arity, print_error)
        }
    }

    fn add_ns_var
	(
            &mut self,
            vm: *mut VM,
            parser: *mut AstParser,
            ast: &Ast,
            name: &String,
            arity: Option<usize>,
            print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        let mut actual_name = "".to_string();
        if let Some(a) = arity {
            actual_name = format!("{}(#{})", name, a);
        } else {
            actual_name = name.to_string();
        }
        
        if let Some(v) = self.var_symtab.get(name) {
            if arity.is_none() {
                if v.var_type == VarType::Function &&
                    print_error
                {
                    unsafe {(*parser).error(ast, format!("namespace variable {} is a function", name))};
                }
                return Err(VarError::new(
                    name.clone(),
                    VarErrorType::IsFunction,
                ));
            }
        }

        let is_some = self.var_symtab.get(&actual_name).is_some();
        let ns_var_count = self.get_ns_var_count();

        let var = VarInfo::new(
            name.to_string(),
            None,
            VarType::Function,
            ScopeType::Ns,
            self.get_scope_depth(),
            0,
            0,
        );
        if self.var_symtab.get(name).is_none() {
            self.var_symtab.insert(
                name.to_string(), 
                var.clone(),
            );
        }
        
        if is_some {
            if print_error {
                let mut what = "";
                if arity.is_none() {
                    what = "variable";
                } else {
                    what = "function";
                }
                unsafe {(*parser).error(ast, format!("namespace {} {} already exists", what, actual_name))};
            }
            return Err(VarError::new(
                actual_name.clone(),
                VarErrorType::AlreadyExists,
            ));
        } else {
            if arity.is_none() {
                let var =
                    VarInfo::new(
                        actual_name.clone(),
                        arity,
                        VarType::Var,
                        ScopeType::Ns,
                        self.get_scope_depth(),
                        self.get_ns_var_count(),
                        0,
                    );
                self.inc_ns_var_count();
                self.var_symtab.insert(
                    actual_name.to_string(), 
                    var.clone(),
                );
                
                Ok(var.clone())
            } else {
                let var = 
                    VarInfo::new(
                        actual_name.clone(),
                        arity,
                        VarType::Function,
                        ScopeType::Ns,
                        self.get_scope_depth(),
                        self.get_ns_var_count(),
                        0,
                    );
                self.inc_ns_var_count();
                self.var_symtab.insert(
                    actual_name.to_string(), 
                    var.clone(),
                );
                
                Ok(var.clone()) 
            } 
        }
    }

    fn add_local_var
	(
            &mut self,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: Option<usize>,
            print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        if let Some(last) = self.fc_vec.last_mut() {
            return last.declare_local(parser, ast, gc, name, arity, print_error);
        } else {
            panic!("function compiler stack is empty");
        }
    }

    fn resolve_var
	(
            &mut self,
            parser: *mut AstParser,
            vm: *mut VM,
            ast: &Ast,
            gc: &Gc,
            name: &String,
	    arity: Option<usize>,
	    print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        let mut actual_name = "".to_string();
        if let Some(a) = arity {
            actual_name = format!("{}(#{})", name, a);
        } else {
            actual_name = name.to_string();
        }
        let var = self.resolve_other_var(parser, vm, ast, gc, &actual_name, arity);
	match var {
            Ok(v) => {
		return Ok(v)
            },
            _ => {}, 
        }
	let var = self.resolve_ns_var(parser, ast, &actual_name, arity);
        match var {
	    Ok(v) => {
                Ok(v)
            },
            Err(e) => {
		Err(e)
            },
        }
    }

    fn resolve_ns_var
	(
            &mut self, 
            parser: *mut AstParser,
            ast: &Ast,
            name: &String,
	    arity: Option<usize>,
	) -> Result<VarInfo, VarError>
    {
        if let Some(var) = self.var_symtab.get(name) {
            if let Some(ref mut last) = self.fc_vec.last_mut() {
                if arity.is_none() && 
                    var.arity.is_none() && 
                    var.var_type == VarType::Function
                {
                    unsafe{
                        (*parser).error(ast, format!(
                            "namespace variable {} is a function",
                            name
                        ))
                    };
                    return Err(VarError::new(
                        name.to_string(),
                        VarErrorType::IsFunction,
                    ));
                } 
            }
	    Ok(var.clone())
	} else {
            Err(VarError::new(
                name.to_string(),
                VarErrorType::Undefined,
            ))
        }
    }

    fn resolve_other_var
	(
            &mut self,
            parser: *mut AstParser,
            vm: *mut VM,
            ast: &Ast,
            gc: &Gc,
            name: &String,
	    arity: Option<usize>,
	) -> Result<VarInfo, VarError>
    {
        let var = self.resolve_local(parser, ast, gc, name, arity);
        match var {
            Ok(_) => {
		return var;
            },
            _ => {},
        }
	self.resolve_upvalue(parser, vm, ast, gc, name, arity)
    }

    fn resolve_local
	(
            &mut self,
            parser: *mut AstParser,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: Option<usize>,
	) -> Result<VarInfo, VarError>
    {
        if let Some(last) = self.fc_vec.last_mut() {
            return last.resolve_local_internal(parser, ast, gc, name, arity, true);
	}
        panic!("function compiler stack is empty");
    }

    fn resolve_upvalue
	(
            &mut self,
            parser: *mut AstParser,
            vm: *mut VM,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: Option<usize>,
	) -> Result<VarInfo, VarError>
    {
        let mut iter = self.fc_vec.iter_mut().rev();
        let fc = iter.next();

        if let Some(fun_compiler) = fc {
            return fun_compiler.resolve_upvalue_internal(parser, vm, ast, gc, name, arity, true, iter);
        }
        
	unreachable!();
    }

    fn resolve_function
	(
            &mut self,
            parser: *mut AstParser,
            vm: *mut VM,
            ast: &Ast,
            gc: &Gc,
            name: &String,
            arity: usize,
	    print_error: bool,
	) -> Result<VarInfo, VarError>
    {
        self.resolve_var(parser, vm, ast, gc, name, Some(arity), print_error)
    }

    fn resolve_ns_fun(
	&mut self,
	parser: *mut AstParser,
        vm: *mut VM,
        ast: &Ast,
        gc: &Gc,
        name: &String,
        arity: usize,
    ) -> Result<VarInfo, VarError>
    {
	let actual_name = format!("{}(#{})", name, arity as u32);
	if let Some(var) = self.var_symtab.get_mut(&actual_name) {
	    Ok(var.clone())
	} else {
	    Err(VarError::new(
		actual_name,
		VarErrorType::Undefined,
	    ))
	}
    }

    fn resolve_local_fun(
	&mut self,
	parser: *mut AstParser,
        vm: *mut VM,
        ast: &Ast,
        gc: &Gc,
        name: &String,
        arity: usize,
	print_error: bool,
    ) -> Result<VarInfo, VarError>
    {
	if let Some(last) = self.fc_vec.last_mut() {
	    let actual_name = format!("{}(#{})", name, arity);
	    last.resolve_local_internal(
		parser, ast, gc, &actual_name, Some(arity), print_error)
	} else {
	    panic!("function compiler stack is empty");
	}
    }
    
    fn define_var(&mut self, vm: *mut VM, fc: &mut FunCompiler, index: usize) {
        if fc.get_scope_depth() > 0 {
            fc.locals.last_mut().unwrap().defined = true;
        } else {
            unsafe {
                (*vm).vars.push(Value::Nil)
            };
            fc.emit_word_op(OpCode::DefVar, index as usize);
        }
    }
}

#[cfg(test)]


mod tests{
    use super::*;
    use crate::parser::{
        interpret_string
    };
    use crate::gc::{Gc, NO_GC};

    #[test]
    fn test_add_ns_var() {
        let mut deps = prep_for_test();
	
        let mut code = "let var = 10;";
        let mut result = interpret_string(&mut code.to_string(), &mut deps.0, &mut deps.1, false, false, false);
        assert_eq!(result, Ok(Value::Nil));

        code = "var;";
        result = interpret_string(&mut code.to_string(), &mut deps.0, &mut deps.1, false, false, false);
        assert_eq!(result, Ok(Value::Int(10)));
    }


    
    fn prep_for_test() -> (VM, Gc) {
        (VM::new(), Gc::new())
    }
}


use ordered_float::{OrderedFloat, Pow};

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
            Value::Closure(c) => Self {
                fun,
                closure: Some(c),
                core: Some(c.get_core()),
                pc: 0,
                fp: 0,
            },
            _ => {
                panic!("'{}' got pushed onto the call stack!", fun);
            }
        }
    }
    pub fn get_fun(&mut self) -> &Value {
        &mut self.fun
    }

    pub fn get_closure(&mut self) -> Option<Managed<Closure>> {
        self.closure
    }

    fn get_core(&mut self) -> Option<Managed<FunctionCore>> {
        self.core
    }

    fn get_pc(&mut self) -> usize {
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
    fn get_fp(&mut self) -> usize {
        self.fp
    }

    #[inline]
    fn set_fp(&mut self, index: usize) -> usize {
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
        Self { stack: vec![] }
    }

    #[inline]
    fn get_fp(&mut self) -> usize {
        self.stack.last_mut().unwrap().get_fp()
    }

    #[inline]
    fn set_fp(&mut self, index: usize) {
        self.stack.last_mut().unwrap().set_fp(index);
    }

    #[inline]
    fn get_pc(&mut self) -> usize {
        self.stack.last_mut().unwrap().get_pc()
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

    fn top(&mut self) -> Option<CallFrame> {
        if let Some(last) = self.stack.last() {
            return Some(last.clone());
        } else {
            None
        }
    }

    fn next_from_top(&mut self) -> Option<CallFrame> {
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
    pub(crate) fn get_backtrace(&mut self) -> String {
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
                            }
                            FunctionType::NamedClosure => {
                                fun_ty_name = String::from("named closure");
                            }
                            FunctionType::AnonClosure => {
                                fun_ty_name = String::from("anonymous closure");
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    }
                    _ => {}
                }
                string = string
                    + &format!(
                        "[ {} {}(#{}) at {} line {} ]\n\n",
                        fun_ty_name, name, arity, filename, 0,
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
    compiler: Compiler,
    mod_vec: Vec<Vec<Value>>,
    mod_index: usize,
    main_fun: Option<Managed<Closure>>,
    calls: CallStack,
    stack: Vec<Value>,
    sp: usize,
}

#[allow(dead_code)]
impl VM {
    pub fn new() -> Self {
        Self {
            compiler: Compiler::new(),
            mod_vec: vec![vec![]],
            mod_index: 0usize,
            main_fun: None,
            calls: CallStack::new(),
            stack: Vec::<Value>::new(),
            sp: 0usize,
        }
    }

    #[inline]
    fn get_sp(&mut self) -> usize {
        self.sp
    }

    #[inline]
    fn set_sp(&mut self, index: usize) {
        self.sp = index;
    }

    #[inline]
    fn get_fp(&mut self) -> usize {
        self.calls.get_fp()
    }

    #[inline]
    fn set_fp(&mut self, index: usize) {
        self.calls.set_fp(index);
    }
    #[inline]
    fn get_pc(&mut self) -> Option<usize> {
        if let Some(mut frame) = self.calls.top() {
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
        if let Some(val) = self.mod_vec[self.mod_index].get_mut(index) {
            Some((*val).clone())
        } else {
            None
        }
    }

    fn set_var(&mut self, index: usize, value: Value) -> Option<&Value> {
        if let Some(val) = self.mod_vec[self.mod_index].get_mut(index) {
            *val = value;
            Some(val)
        } else {
            None
        }
    }

    fn add_var(&mut self) {
        self.mod_vec[self.mod_index].push(Value::Nil);
    }

    fn push_call_frame(&mut self, num_args: usize) {
        let top = self.stack.len();
        let call = &mut self.stack[top - num_args as usize - 1];
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

    fn check_arity(&self, fun: &Value, num_args: usize) -> Result<usize, String> {
        let top = self.stack.len();
        let mut arity = 0;
        match fun {
            Value::Closure(c) => {
                arity = c.get_arity();
            }
            _ => {
                return Err(format!("not a function type, but {}", fun.value_type()));
            }
        }
        if num_args != arity {
            Err(format!("{} args expected, found {}", arity, num_args))
        } else {
            Ok(num_args)
        }
    }

    fn call_value(&mut self, num_args: usize, debug: bool, gc: &Gc) -> Result<Value, String> {
        let fun_index = self.stack.len() - (num_args as usize) - 1;
        let fun = &self.stack[fun_index];
        let args = &self.stack[fun_index + 1..fun_index + (num_args as usize) + 1];
        match self.check_arity(fun, num_args) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }
        match fun {
            Value::Closure(_) => self.call_closure(num_args, gc, debug),
            _ => Err(format!("not a function type, but {}", fun.value_type())),
        }
    }

    #[allow(unused_assignments)]
    fn call_closure(&mut self, num_args: usize, gc: &Gc, debug: bool) -> Result<Value, String> {
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
                }
                OpCode::Pop => {
                    self.stack_pop();
                    Ok(Value::Nil)
                }
                OpCode::PopN => {
                    read_byte!(pc, code, arg);
                    for _i in 0..arg {
                        self.stack_pop();
                    }

                    Ok(Value::Nil)
                }
                OpCode::Dup => {
                    let last = self.get_stack_top();
                    if let Some(i) = last {
                        self.stack.push(i.clone());
                    } else {
                        panic!("accessed stack from out of bounds");
                    }
                    Ok(Value::Nil)
                }
                OpCode::AddMod => Ok(Value::Nil),
                OpCode::LoadMod => Ok(Value::Nil),
                OpCode::LoadTrue => {
                    self.stack.push(Value::Bool(true));
                    Ok(Value::Nil)
                }
                OpCode::LoadFalse => {
                    self.stack.push(Value::Bool(false));
                    Ok(Value::Nil)
                }
                OpCode::LoadNil => {
                    self.stack.push(Value::Nil);
                    Ok(Value::Nil)
                }
                OpCode::Load0
                | OpCode::Load1
                | OpCode::Load2
                | OpCode::Load3
                | OpCode::Load4
                | OpCode::Load5
                | OpCode::Load6
                | OpCode::Load7
                | OpCode::Load8
                | OpCode::Load9
                | OpCode::Load10 => {
                    self.stack.push(Value::Int(
                        (u8::from(opcode) - (u8::from(OpCode::Load0))) as i64,
                    ));
                    Ok(Value::Nil)
                }
                OpCode::LoadValue => {
                    read_word!(pc, code, arg);
                    self.stack
                        .push(closure.get_core().get_constants()[arg as usize].clone());
                    Ok(Value::Nil)
                }
                OpCode::Array => Ok(Value::Nil),
                OpCode::Dict => Ok(Value::Nil),
                OpCode::DefVar => {
                    read_word!(pc, code, index);
                    let top = self.get_stack_top();

                    if let Some(val) = top {
                        self.set_var(index as usize, val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }
                OpCode::LoadVar => {
                    read_word!(pc, code, arg);
                    let elem = self.get_var(arg as usize).clone();
                    if let Some(val) = elem {
                        self.stack.push(val);
                    } else {
                        match self.stack[0].get_uintsize() {
                            Ok(v) => {
                                return Err(format!(
                                    "var index out of range: {}; limit: {}",
                                    arg,
                                    self.mod_vec[v].len()
                                ));
                            }
                            Err(e) => {
                                panic!("internal error: {}", e);
                            }
                        }
                    }

                    Ok(Value::Nil)
                }
                OpCode::StoreVar => {
                    read_word!(pc, code, arg);
                    if let Some(j) = self.get_stack_top() {
                        self.set_var(arg as usize, j.clone());
                    }
                    Ok(Value::Nil)
                }
                OpCode::LoadLocal => Ok(Value::Nil),
                OpCode::LoadLocal0 => Ok(Value::Nil),
                OpCode::LoadLocal1 => Ok(Value::Nil),
                OpCode::LoadLocal2 => Ok(Value::Nil),
                OpCode::LoadLocal3 => Ok(Value::Nil),
                OpCode::LoadLocal4 => Ok(Value::Nil),
                OpCode::LoadLocal5 => Ok(Value::Nil),
                OpCode::LoadLocal6 => Ok(Value::Nil),
                OpCode::LoadLocal7 => Ok(Value::Nil),
                OpCode::LoadLocal8 => Ok(Value::Nil),
                OpCode::StoreLocal => Ok(Value::Nil),
                OpCode::StoreLocal0 => Ok(Value::Nil),
                OpCode::StoreLocal1 => Ok(Value::Nil),
                OpCode::StoreLocal2 => Ok(Value::Nil),
                OpCode::StoreLocal3 => Ok(Value::Nil),
                OpCode::StoreLocal4 => Ok(Value::Nil),
                OpCode::StoreLocal5 => Ok(Value::Nil),
                OpCode::StoreLocal6 => Ok(Value::Nil),
                OpCode::StoreLocal7 => Ok(Value::Nil),
                OpCode::StoreLocal8 => Ok(Value::Nil),
                OpCode::LoadUpvalue => Ok(Value::Nil),
                OpCode::StoreUpvalue => Ok(Value::Nil),
                OpCode::LoadField => Ok(Value::Nil),
                OpCode::StoreField => Ok(Value::Nil),
                OpCode::LoadStatic => Ok(Value::Nil),
                OpCode::StoreStatic => Ok(Value::Nil),
                OpCode::LoadMethod => Ok(Value::Nil),
                OpCode::LoadStaticMethod => Ok(Value::Nil),
                OpCode::LoadSuperMethod => Ok(Value::Nil),
                OpCode::JumpFwd => {
                    read_word!(pc, code, arg);
                    pc += arg as usize;
                    Ok(Value::Nil)
                }
                OpCode::JumpBack => {
                    read_word!(pc, code, arg);
                    pc -= arg as usize;
                    Ok(Value::Nil)
                }
                OpCode::JumpTrue => {
                    read_word!(pc, code, arg);
                    let val = self.get_stack_top();
                    if let Some(i) = val {
                        if !is_false(i.clone()) {
                            pc += arg as usize;
                        }
                    }
                    Ok(Value::Nil)
                }
                OpCode::JumpFalse => {
                    read_word!(pc, code, arg);
                    let val = self.get_stack_top();
                    if let Some(i) = val {
                        if is_false(i.clone()) {
                            pc += arg as usize;
                        }
                    }
                    Ok(Value::Nil)
                }
                OpCode::DefStatic => Ok(Value::Nil),
                OpCode::Method => Ok(Value::Nil),
                OpCode::StaticMethod => Ok(Value::Nil),
                OpCode::Instance => Ok(Value::Nil),
                OpCode::Closure => Ok(Value::Nil),
                OpCode::CloseUpvalue => Ok(Value::Nil),
                OpCode::Call => {
                    read_byte!(pc, code, num_args);
                    println!("num_args = {}", num_args);

                    match self.call_value((num_args as usize) + 1, debug, gc) {
                        Ok(ref res) => {
                            let top = self.stack.len();
                            self.stack[top - (num_args as usize) - 1] = res.clone();
                            Ok(res.clone())
                        }
                        Err(e) => Err(e),
                    }
                }
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
                }
                OpCode::Ternary => Ok(Value::Nil),
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
                            }
                            Value::Float(float) => {
                                let val: Value = Value::Float(-float);
                                let mut elem = self.get_stack_top();
                                if let Some(ref mut e) = elem {
                                    *e = val;
                                } else {
                                    panic!("stack is empty");
                                }
                                Ok(Value::Nil)
                            }
                            _ => Err("cannot negate value".to_string()),
                        }
                    } else {
                        panic!("stack is empty");
                    }
                }
                OpCode::Print => {
                    if let Some(val) = self.get_stack_top() {
                        print!("{}", val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }
                OpCode::Println => {
                    if let Some(val) = self.get_stack_top() {
                        println!("{}", val);
                    } else {
                        panic!("stack is empty");
                    }
                    Ok(Value::Nil)
                }
                OpCode::Input => Ok(Value::Nil),
                OpCode::MathOp => {
                    read_byte!(pc, code, arg);
                    self.execute_math_op(MathOp::from(arg), gc)
                }
                OpCode::MathAssignOp => Ok(Value::Nil),
                OpCode::BitwiseOp => Ok(Value::Nil),
                OpCode::BitwiseAssignOp => Ok(Value::Nil),
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
            }
            Err(e) => Err(e),
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
        for j in 0..self.mod_vec.len() {
            for i in 0..self.mod_vec[j].len() {
                print!("[{}]", self.mod_vec[j][i]);
            }
        }
        println!("]");
    }
    fn debug_stack(&mut self, fp: usize) {
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

    fn execute_math_op(&mut self, op: MathOp, gc: &Gc) -> Result<Value, String> {
        match op {
            MathOp::Add => self.execute_add(gc),
            MathOp::Subtract => self.execute_subtract(gc),
            MathOp::Multiply => self.execute_multiply(gc),
            MathOp::Divide => self.execute_divide(gc),
            MathOp::Modulo => self.execute_modulo(gc),
            MathOp::Power => self.execute_power(gc),
            _ => {
                panic!("invalid math operator");
            }
        }
    }

    fn execute_math_assign_op(&mut self, op: MathOp, gc: &Gc) -> Result<Value, String> {
        match op {
            MathOp::Add => self.execute_add_assign(gc),
            MathOp::Subtract => self.execute_subtract_assign(gc),
            MathOp::Multiply => self.execute_multiply_assign(gc),
            MathOp::Divide => self.execute_divide_assign(gc),
            MathOp::Modulo => self.execute_modulo_assign(gc),
            MathOp::Power => self.execute_power_assign(gc),
            _ => {
                panic!("invalid compound assignment operator");
            }
        }
    }

    fn execute_add(&mut self, gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1 + val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1 + val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(val1) => match arg2 {
                Value::String(val2) => {
                    arg1 = Value::String(gc.manage(format!("{}{}", *val1, *val2), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "+: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_subtract(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1 - val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1 - val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "-: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_multiply(&mut self, gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1 * val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1 * val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::String(gc.manage(val1.repeat(val2 as usize), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "*: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_divide(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    if val2 <= 0 {
                        return Err(String::from("division by zero"));
                    }
                    arg1 = Value::Int(val1 / val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    if val2 <= OrderedFloat(0.0) {
                        return Err(String::from("division by zero"));
                    }
                    arg1 = Value::Float(val1 / val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "/: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_modulo(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    if val2 <= 0 {
                        return Err(String::from("modulo by zero"));
                    }
                    arg1 = Value::Int(val1 / val2);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "/: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_power(&mut self, _gc: &Gc) -> Result<Value, String> {
        let arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(val1) => match arg2 {
                Value::Int(val2) => {
                    arg1 = Value::Int(val1.pow(val2 as u32));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(val1) => match arg2 {
                Value::Float(val2) => {
                    arg1 = Value::Float(val1.pow(val2));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "**: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    // compound assignment math operators
    fn execute_add_assign(&mut self, gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    *val1 += *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    *val1 += *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(ref mut val1) => match arg2 {
                Value::String(ref mut val2) => {
                    **val1 = format!("{}{}", **val1, **val2);
                    arg1 = Value::String(gc.manage(val1.to_string(), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "+=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_subtract_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    *val1 -= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    *val1 -= *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "-=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_multiply_assign(&mut self, gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    *val1 *= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(val2) => {
                    *val1 *= *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::String(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    **val1 = val1.repeat(*val2 as usize);
                    arg1 = Value::String(gc.manage(val1.clone().to_string(), &NO_GC));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "*=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_divide_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    if *val2 <= 0 {
                        return Err(String::from("division by zero"));
                    }
                    *val1 /= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    if *val2 <= OrderedFloat(0.0) {
                        return Err(String::from("division by zero"));
                    }
                    *val1 /= *val2;
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "/=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_modulo_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    if *val2 <= 0 {
                        return Err(String::from("modulo by zero"));
                    }
                    *val1 %= *val2;
                    arg1 = Value::Int(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "%=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute_power_assign(&mut self, _gc: &Gc) -> Result<Value, String> {
        let mut arg2 = self.stack.pop().unwrap();
        let mut arg1 = self.stack.pop().unwrap();

        match arg1 {
            Value::Int(ref mut val1) => match arg2 {
                Value::Int(ref mut val2) => {
                    arg1 = Value::Int(val1.pow(val1.pow(*val2 as u32) as u32));
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            Value::Float(ref mut val1) => match arg2 {
                Value::Float(ref mut val2) => {
                    *val1 = val1.pow(*val2);
                    arg1 = Value::Float(*val1);
                    self.stack.push(arg1.clone());
                    return Ok(arg1);
                }
                _ => {}
            },
            _ => {}
        }
        Err(format!(
            "**=: invalid operands {} and {}",
            arg1.value_type(),
            arg2.value_type()
        ))
    }

    fn execute(
        &mut self,
        root: &Option<Box<Ast>>,
        compiler: &Compiler,
        gc: &Gc,
        debug: bool,
        compile: bool,
    ) -> Result<Value, InterpretErrorType> {
        self.calls.clear();
        self.stack.clear();
        let closure = compiler.compile(root, self, gc);
        if compile && closure.is_some() {
            return Ok(Value::Nil);
        }

        if let Some(c) = closure {
            self.main_fun = closure;

            // push main function onto the stack
            self.stack.push(Value::Closure(c));

            self.push_call_frame(0usize);
            let result = self.call_closure(0usize, &gc, debug);
            match result {
                Ok(val) => {
                    return Ok(val);
                }
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

    pub(crate) fn run(
        &mut self,
        root: &Option<Box<Ast>>,
        gc: &Gc,
        debug: bool,
        compile: bool,
    ) -> Result<Value, InterpretErrorType> {
        let compiler = self.compiler;
        self.execute(root, &compiler, gc, debug, compile)
    }

    pub fn print_backtrace(&mut self) {
        println!("{}", self.calls.get_backtrace());
    }
}

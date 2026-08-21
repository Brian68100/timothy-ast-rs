use crate::managed::*;
use crate::lexer::*;
use crate::token::*;
use crate::function_core::*;
use crate::opcode::*;
use crate::closure::*;
use crate::ast::*;
use crate::value::*;
use crate::gc::{Gc, NO_GC};
use crate::natives::*;
use std::collections::HashMap;
use std::iter::Rev;
use ordered_float::{OrderedFloat, Pow};


pub(crate) const MAX_MODULES: usize = 65536;
pub(crate) const MAX_LOCALS: usize = 256;
pub(crate) const MAX_UPVALUES: usize = 256;
pub(crate) const MAX_CONSTANTS: usize = 65536;
pub(crate) const MAX_VARIABLES: usize = 65536;
pub(crate) const MAX_FIELDS: usize = 256;
pub(crate) const MAX_METHODS: usize = 65536;

#[derive(Debug, Clone, PartialEq)]
pub enum InterpretErrorType {
    ParseError,
    CompileTimeError,
    RuntimeError,
    OtherError,
}

#[derive(Clone)]
enum BindingPower {
    NoBp,
    Assignment,     // = += -= *= /= &= |= ^= <<= >>=

    Ternary,        // ? :
    LOr,            // ||
    LAnd,           // &&
    Equality,       // == !=
    Comparison,     // < > <= >=
    Bitwise,        // & ^ |
    Shift,          // << >>
    Term,           // + -
    Factor,         // * /
    Unary,          // ! ~ + -
    Exponent,       // **
    Call,           // . () []
    Primary,
}

impl From<u8> for BindingPower {
    fn from(val: u8) -> Self {
        match val {
            0  => BindingPower::NoBp,
            1  => BindingPower::Assignment,     // = += -= *= /= &= |= ^= <<= >>=
            2  => BindingPower::Ternary,        // ? :
            3  => BindingPower::LOr,             // ||
            4  => BindingPower::LAnd,            // &&
            5  => BindingPower::Equality,       // == !=
            6  => BindingPower::Comparison,     // < > <= >=
            7  => BindingPower::Bitwise,        // & ^ |
            8  => BindingPower::Shift,          // << >>
            9  => BindingPower::Term,           // + -
            10 => BindingPower::Factor,         // * /
            11 => BindingPower::Unary,          // ! ~ + -
            12 => BindingPower::Exponent,       // **
            13 => BindingPower::Call,           // . (| ( ) [ ]
            14 => BindingPower::Primary,
            _  => panic!("Invalid binding power value {}", val)
        }
    }
}

impl Into<u8> for BindingPower  {
    fn into(self) -> u8 {
        match self {
            BindingPower::NoBp => 0,
            BindingPower::Assignment => 1,     // = += -= *= /= &= |= ^= <<= >>=
            BindingPower::Ternary => 2,        // ? :
            BindingPower::LOr => 3,             // ||
            BindingPower::LAnd => 4,            // &&
            BindingPower::Equality => 5,       // == !=
            BindingPower::Comparison => 6,     // < > <= >=
            BindingPower::Bitwise => 7,        // & ^ |
            BindingPower::Shift => 8,          // << >>
            BindingPower::Term => 9,           // + -
            BindingPower::Factor => 10,         // * /
            BindingPower::Unary => 11,          // ! ~ + -
            BindingPower::Exponent => 12,       // **
            BindingPower::Call => 13,           // . () []
            BindingPower::Primary => 14,
        }
    }
} 

struct LexerData {
    lexer: Option<Lexer>,
    curr: Token,
    prev: Token,
}

pub struct Parser<'a> {
    lexer_stack: Vec<LexerData>,
    error_count: usize,
    gc: Gc,
    print_ast_flag: bool,
    compile_flag: bool,
    debug_flag: bool,
    
    in_panic_mode: bool,
    in_panic_lock_mode: bool,
    compiler: Compiler<'a>,
    vm: VM,
}

impl<'a> Parser<'_> { 
    pub fn new(
        print_ast: bool,
        compile: bool,
        debug: bool
    ) -> Self {
        let mut s = Self {
            print_ast_flag: print_ast,
            compile_flag: compile,
            debug_flag: debug,
            gc: Gc::new(),
            vm: VM::new(),
            lexer_stack: vec![],
            in_panic_mode: false,
            in_panic_lock_mode: false,
            error_count: 0,
            compiler: Compiler::new(compile),
            /*table: vec![
                { NoPrec, false, false }, // Err
                { NoPrec, false, false }, // false
                { NoPrec, false, false }, // Done
                { NoPrec, false, false }, // Ident
                { NoPrec, false, false }, // Int
                { NoPrec, false,match bp {
                            Some(x)
                        } false }, // Uint
                { NoPrec, false, false }, // Float
                { NoPrec, false, false }, // String
                { NoPrec, false, false }, // LetKw
                { NoPrec, false, false }, // MacroKw
                { NoPrec, false, false }, // FunKw
                { NoPrec, false, false }, // AnonClosure
                { NoPrec, false, false }, // AnonFnKw
                { NoPrec, false, false }, // IfKw
                { NoPrec, false, false }, // ElseKw
                { NoPrec, false, false }, // WhileKw
                { NoPrec, false, false }, // DoKw
                { NoPrec, false, false }, // ForKw
                { NoPrec, false, false }, // ClassKw
                { NoPrec, false, false }, // ThisKw
                { NoPrec, false,match bp {
                            Some(x)
                        } false }, // SuperKw
                { NoPrec, false, false }, // StaticKw
                { NoPrec, false, false }, // ContinueKw
                { NoPrec, false, false }, // BreakKw
                { NoPrec, false, false }, // ReturnKw
                { NoPrec, false, false }, // NewKw
                { NoPrec, false, false }, // NullKw
                { NoPrec, false, false }, // TrueKw
                { NoPrec, false, false }, // FalseKw
                { NoPrec, false, match bp {
                            Some(x)
                        }false }, // ConstKw
                { NoPrec, false, false }, // NsKw
                { NoPrec, false, false }, // ImportKw
                { NoPrec, false, false }, // LParen
                { NoPrec, false, false }, // RParen
                { NoPrec, false, false }, // LBrace
                { NoPrec, false, false }, // RBrace
                { NoPrec, false, false }, // LBracket
                { NoPrec, false, false }, // RBracket
                { NoPrec, false, false }, // Question
                { NoPrec, false, false }, // Semicolon
                { NoPrec, false, false }, // Colon
                { NoPrec, false, false }, // DoubleColon
                { NoPrec, false, false }, // Comma
                { NoPrec, false, false }, // Dot
                { NoPrec, false, false }, // Field
                { NoPrec, false, false }, // StaticField
                { NoPrec, false, false }, // LAnd
                { NoPrec, false, false }, // LOr
                { NoPrec, false, false }, // LNot
                { NoPrec, false, false }, // Equal
                { NoPrec, false, false }, // NEqual
                { NoPrec, false, false }, // Less
                { NoPrec, false, false }, // LessEqual
                { NoPrec, false, false }, // Greater
                { NoPrec, false, false }, // GreaterEqual
                { NoPrec, false, false }, // Add
                { NoPrec, false, false }, // Subtract
                { NoPrec, false, false }, // Multiply
                { NoPrec, false, false }, // Divide
                { NoPrec, false, false }, // Modulo
                { NoPrec, false, false }, // Power
                { NoPrec, false, false }, // And
                { NoPrec, false, false }, // Or
                { NoPrec, false, false }, // XOr
                { NoPrec, false, false }, // Not
                { NoPrec, false, false }, // LShift
                { NoPrec, false, false }, // RShift
                { NoPrec, false, false }, // Assign
                { NoPrec, false, false }, // AddAssign
                { NoPrec, false, false }, // SubtractAssign
                { NoPrec, false, false }, // MultiplyAssign
                { NoPrec, false, false }, // DivideAssign
                { NoPrec, false, false }, // ModuloAssign
                { NoPrec, false, false }, // AndAssign
                { NoPrec, false, false }, // OrAssign
                { NoPrec, false, false }, // XorAssign
                { NoPrec, false, false }, // LShiftAssign
                { NoPrec, false, false }, // RShiftAssign
                { NoPrec, false, false }, // IndexMethod
                { NoPrec, false, false }, // IndexAssignMethod
            ],*/
        };
        s.lexer_stack = vec![];
        s.error_count = 0;
        s.lexer_stack.push(LexerData {
            lexer: None,
            curr: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
            prev: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
        });
        s
    }

    pub fn parse_text(
        &mut self, 
        text: &mut String,
    ) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Lexer::new_from_text(text); 

        self.lexer_stack.clear();
        self.lexer_stack.push(LexerData {
            lexer: Some(lexer),
            curr: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
            prev: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
        });
        self.advance(&mut self.gc);
        
        let mut root: Option<Box<Ast>> = 
            Some(Box::new(Ast::FunDecl {
                name: " main".to_string(),
                fun_ty: FunctionType::TopLevel,
                params: vec![],
                listing: vec![],
                closure: None,
            }));
        
        if !self.parse(root, &mut self.gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            self.vm.run(&mut self.compiler, &mut self.gc, 
                self.debug_flag, self.compile_flag)
        }
    }
    pub fn parse_file(&mut self, fname: String) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Lexer::new_from_file(fname); 
        if lexer.is_none() {
            return Err(InterpretErrorType::ParseError);
        }

        self.lexer_stack.push(LexerData {
            lexer: lexer,
            curr: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
            prev: Token {
                kind: TokenKind::None,
                ty: TokenType::None,
                lexeme: "".to_string(),
                line: 1,
                col: 1,
                value: Value::Nil,
            },
        });
        self.advance(&mut self.gc);

        let mut root: Option<Box<Ast>> = 
            Some(Box::new(Ast::FunDecl {
                name: " main".to_string(),
                fun_ty: FunctionType::TopLevel,
                params: vec![],
                listing: vec![],
                closure: None,
            }));
        
        if !self.parse(root, &self.gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            self.vm.run(&mut self.compiler, &mut &self.gc, 
                self.debug_flag, self.compile_flag)
        }
    }

    pub fn store_fun(
    	&mut self, 
    	name: &str,
        arity: usize,
    	mut fun: fn(gc: &Gc, args: &[Value])
            -> Result<Value, String>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn advance(&mut self, gc: &Gc) {
        self.lexer_stack.last_mut().unwrap().prev = self.lexer_stack.last_mut().unwrap().curr.clone();
        loop {
            self.lexer_stack.last_mut().unwrap().curr = self.lexer_stack.last_mut().unwrap().lexer.as_mut().unwrap().scan(gc);
            if self.current().kind != TokenKind::Err { break; }
            let curr = self.current();
            self.error_at_curr(curr.lexeme)
        }
    }

    fn check(&mut self, tk: TokenKind) -> bool {
        self.current().kind == tk
    }

    fn match_token(&mut self, tk: TokenKind, gc: &Gc) -> bool {
        if self.current().kind != tk { return false; }
        self.advance(gc);
        return true;
    }

    fn consume(&mut self, tk: TokenKind, err_message: String, gc: &Gc) -> bool{
        if self.current().kind == tk {
            self.advance(gc);
            return true;
        }
        self.error_at_curr(err_message);
            
        false
    }

    fn error_at(&mut self, token: Token, err_message: String) {
        if self.in_panic_mode || self.in_panic_lock_mode {
            return;
        } 
        
        eprint!("{}:{}:{}: error", self.lexer_stack.last_mut().unwrap().lexer.as_mut().unwrap().file_name(), token.line, token.col);

        if token.kind == TokenKind::Done {
            eprint!(" at end of file");
        } else if token.kind == TokenKind::Err {
        } else {
            eprint!(" at '{}'", token.lexeme);
        }

        eprintln!(": {}", err_message);
        eprintln!("");
        self.error_count += 1;
    
        if self.error_count >= 20 {
            if !self.in_panic_mode {
                eprintln!("error: bailing out; too many errors");
                self.in_panic_lock_mode = true;
            }
        }
        self.in_panic_mode = true;
    }

    fn error_at_prev(&mut self, err_message: String) {
        let prev = self.previous();
        self.error_at(prev, err_message);
    }

    fn error_at_curr(&mut self, err_message: String) {
        let curr = self.current();
        self.error_at(curr, err_message);
    }

    

    fn current(&mut self) -> Token {
        self.lexer_stack.last_mut().unwrap().curr.clone()
    }

    fn previous(&mut self) -> Token {
        self.lexer_stack.last_mut().unwrap().prev.clone()
    }


    // Entry point of parser
    fn parse(
	&mut self,
        mut root: Option<Box<Ast>>,
        gc: &Gc,
    ) -> bool {
        while self.current().kind != TokenKind::Done {
            let _tk = self.current().kind;
            let mut left: Option<Box<Ast>> = self.parse_stmt(None, gc);
            println!("left = {:#?}", left);
            if left.is_none() {
                continue;
            }

            
            match root {
                Some(ref mut val) => {
                    match **val {
                        Ast::FunDecl {
                            ref mut listing,
                            ..} => {

                            listing.push(left.unwrap());
                        }, 
                        _ => {},
                    };
                },
                _ => { unreachable!(); }
            };
        };

        if self.error_count > 0 {
            if self.error_count > 1 {
                println!("\nFound {} errors.", self.error_count);
            } else {
                println!("\nFound 1 error.");
            }
            false
        } else {
            true
        }
    }

    // Pratt Parser methods

    // Synchronizes the error system
    fn synchronize_error(&mut self, gc: &Gc) {    
        self.in_panic_mode = false;

        while self.current().kind != TokenKind::Done {
            if self.previous().kind == TokenKind::Semicolon { return; }
            match self.current().kind {
                TokenKind::LetKw |
                TokenKind::FunKw |
                TokenKind::AnonClosure |
                //TokenKind::NsKw |
                TokenKind::IfKw |
                TokenKind::WhileKw |
                TokenKind::ForKw |
                TokenKind::ReturnKw => {
                    return;
                },
                _ => {},
            }
 
            self.advance(gc);
        }
    }

    // Calls the corresponding statement function
    fn call_stmt_fn
    (
        &mut self, 
        tk: TokenKind, 
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        match tk {
            TokenKind::LetKw => self.parse_let_stmt(left, gc),
            TokenKind::ReturnKw => self.parse_return_stmt(left, gc),
            TokenKind::LBrace => self.parse_block(left, gc),
            TokenKind::IfKw => self.parse_if_stmt(left, gc),
            _ => None
        }
    }

    // Calls the corresponding prefix function 
    fn call_prefix_fn
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        let tk = self.current().kind;
        match tk {
            TokenKind::Int => self.parse_int(left, gc),
            TokenKind::Float => self.parse_float(left, gc),
            TokenKind::String => self.parse_string(left, gc),
            TokenKind::Ident => self.parse_var(left, gc),
            TokenKind::TrueKw => self.parse_true(left, gc),
            TokenKind::FalseKw => self.parse_false(left, gc),
            TokenKind::NilKw => self.parse_nil(left, gc),
            TokenKind::Subtract => self.parse_negative(left, gc),
            TokenKind::Add => self.parse_positive(left, gc),
            TokenKind::AnonClosure => self.parse_anon_closure(left, gc),
            TokenKind::FunKw => self.parse_named_closure(left, gc),
            TokenKind::LNot => self.parse_lnot(left, gc),
            TokenKind::Not => self.parse_not(left, gc),
            _ => None
        }
    }

    // Calls the corresponding infix function
    fn call_infix_fn
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        match tk {
            TokenKind::Add => self.parse_addition(left, gc),
            TokenKind::Subtract => self.parse_subtraction(left, gc),
            TokenKind::Multiply => self.parse_multiplication(left, gc),
            TokenKind::Divide => self.parse_division(left, gc),
            TokenKind::Modulo => self.parse_modulo(left, gc),
            TokenKind::Power => self.parse_power(left, gc),
            TokenKind::LOr => self.parse_lor(left, gc),
            TokenKind::LAnd => self.parse_land(left, gc),
            TokenKind::Or => self.parse_or(left, gc),
            TokenKind::And => self.parse_and(left, gc),
            TokenKind::XOr => self.parse_xor(left, gc),
            TokenKind::LShift => self.parse_lshift(left, gc),
            TokenKind::RShift => self.parse_rshift(left, gc),
            TokenKind::Assign => self.parse_assignment(left, gc),
            TokenKind::AddAssign => self.parse_compound_addition(left, gc),
            TokenKind::SubtractAssign => self.parse_compound_subtraction(left, gc),
            TokenKind::MultiplyAssign => self.parse_compound_multiplication(left, gc),
            TokenKind::DivideAssign => self.parse_compound_division(left, gc),
            TokenKind::ModuloAssign => self.parse_compound_modulo(left, gc),
            TokenKind::PowerAssign => self.parse_compound_power(left, gc),
            TokenKind::AndAssign => self.parse_compound_and(left, gc),
            TokenKind::OrAssign => self.parse_compound_or(left, gc),
            TokenKind::XOrAssign => self.parse_compound_xor(left, gc),
            TokenKind::LShiftAssign => self.parse_compound_lshift(left, gc),
            TokenKind::RShiftAssign => self.parse_compound_rshift(left, gc),
            TokenKind::LParen => self.parse_function_call(left, gc),
            _ => None
        }
    }

    fn stmt_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::LetKw => Some(BindingPower::NoBp),
            TokenKind::ReturnKw => Some(BindingPower::NoBp),
            TokenKind::LBrace => Some(BindingPower::NoBp),
            _ => None
        }
    }
    
    fn prefix_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::Int | TokenKind::Float |
            TokenKind::String |
            TokenKind::Ident |
            TokenKind::TrueKw | TokenKind::FalseKw |
            TokenKind::NilKw => Some(BindingPower::Primary),
            TokenKind::Add | TokenKind::Subtract |
            TokenKind::LNot | TokenKind::Not => Some(BindingPower::Unary),
            TokenKind::AnonClosure => Some(BindingPower::Call),
            TokenKind::FunKw => Some(BindingPower::Call),
            _ => None
        }
    }

    fn infix_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::Add | TokenKind::Subtract => Some(BindingPower::Term),
            TokenKind::Multiply | TokenKind::Divide | 
            TokenKind::Modulo | TokenKind::Power => Some(BindingPower::Factor),
            TokenKind::LAnd => Some(BindingPower::LAnd),
            TokenKind::LOr => Some(BindingPower::LOr),
            TokenKind::Or | TokenKind::And | TokenKind::XOr => Some(BindingPower::Bitwise),
            TokenKind::LShift | TokenKind::RShift => Some(BindingPower::Shift),
            TokenKind::Assign | TokenKind::AddAssign | TokenKind::SubtractAssign |
            TokenKind::MultiplyAssign | TokenKind::DivideAssign |
            TokenKind::ModuloAssign | TokenKind::PowerAssign |
            TokenKind::OrAssign | TokenKind::AndAssign | TokenKind::XOrAssign |
            TokenKind::LShiftAssign | TokenKind::RShiftAssign => Some(BindingPower::Assignment),
            TokenKind::LParen | 
                TokenKind::LBrace | TokenKind::LBracket => Some(BindingPower::Call),
            _ => None
        }
    }

    fn parse_stmt
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        let mut node = None;
        let tk = self.current().kind;
        let stmt = self.stmt_bp(tk.clone());
        if stmt.is_some() {
            node = self.call_stmt_fn(tk.clone(), None,  gc);
            if self.in_panic_mode {
                self.advance(gc);
                self.synchronize_error(gc);
            }
            return node;
        }

        node = self.parse_expr_stmt(left, gc);
        if self.in_panic_mode {
            self.advance(gc);
            self.synchronize_error(gc);
        }
        
        node
    }

    fn parse_expr
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
        bp:BindingPower,
    ) -> Option<Box<Ast>>
    {
        let mut tk = self.current().kind;
        let prefix = self.prefix_bp(tk);
        if prefix.is_none() {
            self.error_at_curr("expression expected".to_string());
            return None;
        }
        
        let mut l = self.call_prefix_fn(left, gc);

        tk = self.current().kind;
        let mut infix = self.infix_bp(tk);
        if infix.is_none() {
            return l;
        }
        while <BindingPower as Into<u8>>::into(bp.clone()) <= <BindingPower as Into<u8>>::into(infix.unwrap()) {
            tk = self.current().kind;
            infix = self.infix_bp(tk);
            if infix.is_none() {
                return l;
            }
            l = self.call_infix_fn(l,  gc);
        }

        l
    }


    fn parse_let_stmt
    (
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        self.advance(gc);
        self.consume(TokenKind::Ident, "identifier expected after 'let'".to_string(), gc);
        let ident = self.previous().lexeme;
        //self.advance(gc);
        self.consume(TokenKind::Assign, "assignment expected after identifier".to_string(), gc);

        let e = self.parse_expr(None, gc, BindingPower::Assignment);
        if e.is_none() {
            return None;
        }

        if !self.consume(TokenKind::Semicolon, "';' expected after let statement".to_string(), gc) {
            return None;
        }

        Some(Box::new(Ast::LetDecl {
                        name: ident.clone(),
            expr: e.unwrap(),
        }))
    }

    fn parse_const_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        _gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        None
    }

    fn parse_class_decl(&mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        _gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        None
    }

    fn parse_macro_def(&mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>> 
    {
        None
    }

    fn parse_ns_decl
    (
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        _gc: &Gc
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_import_cmd(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>>
    {
        None
    }

    // control flow statements
    
    // IF Statement

    fn parse_if_stmt(
        &mut self,
        left: Option<Box<Ast>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);
        let if_expr = self.parse_expr(left, gc, BindingPower::Assignment);
        if if_expr.is_some() {
            self.advance(gc);
            let else_expr = self.parse_expr(if_expr, gc, BindingPower::Assignment);
            return else_expr;
        }
        None
    }

    // prefix operators

    fn parse_anon_closure (
        &mut self,
        _left: Option<Box<Ast>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);
        let mut params: Vec<Box<Ast>> = vec![];
        let mut code: Vec<Box<Ast>> = vec![];

        loop {
            if !self.check(TokenKind::RParen) {
                if !self.consume(TokenKind::Ident, "parameter name expected".to_string(), gc) {
                    return None;
                }
                let param = Box::new(Ast::Variable {
		                                name: self.previous().lexeme.clone(),
                    arity: None,
                });
                params.push(param);

                self.match_token(TokenKind::Comma, gc);
            } else {
                break;
            }
            
        }
        if !self.consume(TokenKind::RParen, "`)` expected after parameters".to_string(), gc) {
            return None;
        }

        self.consume(TokenKind::LBrace, "`{` expected at start of lambda function body".to_string(), gc);

        while !self.check(TokenKind::RBrace) &&
              !self.check(TokenKind::Done)
        {
            let mut l = None;
            l = self.parse_stmt(l,  gc);
            if let Some(v) = l {
                code.push(v);
            } else {
            }
        }

        if !self.consume(TokenKind::RBrace, "`}` expected at end of lambda function body".to_string(), gc) {
            return None;
        }
        let closure = Closure::new(
            gc.manage(FunctionCore::new_anon_closure(
                params.len(),
            ), &NO_GC),
            gc.manage(Vec::<Value>::new(), &NO_GC),
        );

        Some(Box::new(Ast::FunDecl {
            name: closure.get_core().get_name().clone(),
            fun_ty: closure.get_core().get_type().clone(),
            params,
            listing: code,
            closure: Some(closure),
        }))
    } 

    fn parse_named_closure
    (
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) ->Option<Box<Ast>>
    {
        self.advance(gc);
        let mut params: Vec<Box<Ast>> = vec![];
        let mut code: Vec<Box<Ast>> = vec![];
        
        if !self.consume(TokenKind::Ident, "function name expected after 'fun'".to_string(), gc) {
            self.advance(gc);
        }
        
        let fun_name = self.previous().lexeme.clone();

        if !self.consume(TokenKind::LParen, "'(' expected after function name".to_string(), gc) {
            self.advance(gc);
        }
        loop {
            if !self.check(TokenKind::RParen) {
                if params.len() >= 16 {
                    self.error_at_prev("too many parameters".to_string());
                }
                if !self.consume(TokenKind::Ident, "parameter name expected".to_string(), gc) {
                    self.advance(gc);
                }

                let param = Box::new(Ast::Variable {
                                        name: self.previous().lexeme.clone(),
                    arity: None,
                });
                params.push(param); 
            }
            if !self.match_token(TokenKind::Comma, gc) {
                break;
            }
        }

        if !self.consume(TokenKind::RParen, "')' expected after parameters".to_string(), gc) {
            self.advance(gc);
        }

        if !self.consume(TokenKind::LBrace, "'{' expected at start of function body".to_string(), gc) {
            self.advance(gc);
        }

        while !self.check(TokenKind::RBrace) &&
              !self.check(TokenKind::Done)
        {
            let mut l = None;
            l = self.parse_stmt(l,  gc);
            if let Some(v) = l {
                code.push(v);
            } else {
            }
        }

        if !self.consume(TokenKind::RBrace, "'}' expected at end of function body".to_string(), gc) {
            self.advance(gc);
        }

        if self.error_count > 0 {
            return None;
        }

        let closure = Closure::new(
            gc.manage(FunctionCore::new_named_closure(
                fun_name.clone(), 
                params.len(),
            ), &NO_GC),
            gc.manage(Vec::<Value>::new(), &NO_GC),
        );
        Some(Box::new(Ast::FunDecl {
            name: closure.get_core().get_name().clone(),
            fun_ty: closure.get_core().get_type().clone(),
            params,
            listing: code,
            closure: Some(closure),
        }))
    } 

    fn parse_return_stmt(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);
        let mut ret_expr: Option<Box<Ast>> = None;
        if !self.match_token(TokenKind::Semicolon, gc) {
            ret_expr = self.parse_expr(left, gc, BindingPower::Assignment);
            if ret_expr.is_none() {
                return None;
            }
        }
       
        self.advance(gc);
        
        return Some(Box::new(Ast::ReturnStmt{
                        expr: ret_expr,
        }));
    }

    fn parse_break_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>>
    {
        None
    }

    fn parse_continue_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>>
    {
        None
    }

    fn parse_block(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);

        let mut list: Vec<Box<Ast>> = vec![];

        while !self.check(TokenKind::RBrace) &&
            !self.check(TokenKind::Done)
        {
            let mut l: Option<Box<Ast>> = None;
            l = self.parse_stmt(l,  gc);
            if let Some(i) = l {
                list.push(i.clone());
            } else {
                return None;
            } 
        }

        if !self.consume(TokenKind::RBrace, "'}' expected at end of block".to_string(), gc) {
            self.advance(gc);
        }

        if self.error_count > 0 {
            return None;
        }
        
        Some(Box::new(Ast::Block {
                        listing: list,
        }))
    }

    fn parse_expr_stmt(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        // generate expression nodes
        let node = self.parse_expr(left, gc, BindingPower::Assignment);

        if node.is_none() {
            return None;
        } 

        let es = Box::new(Ast::ExprStmt {
                        expr: node.unwrap(),
        });
       
        if !self.consume(TokenKind::Semicolon, "';' expected after expression".to_string(), gc) {
            return None;
        }

        Some(es)
    } 



    fn parse_int(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value{
                        val: v,
        }))
    }

    fn parse_float(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value{
                        val: v,
        }))
    }

    fn parse_var(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let name = self.current().lexeme;
        self.advance(gc);
	
        Some(Box::new(Ast::Variable {
                        name: name,
            arity: None,
        }))
    }

    fn parse_true(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let v = Value::Bool(true);
        self.advance(gc);
        Some(Box::new(Ast::Value{
	                val: v,
        }))
    }

    fn parse_false(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let v = Value::Bool(false);
        self.advance(gc);
        Some(Box::new(Ast::Value{
                        val: v,
        }))
    }

    fn parse_nil(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value{
                        val: v,
        }))
    }

    fn parse_string
    (
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value{
                        val: v,
        }))
    }

    fn parse_argument_list
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Vec<Box<Ast>>>
    {
        
        let mut arg_list: Vec<Box<Ast>> = vec![];
        if !self.check(TokenKind::RParen) {
            while !self.check(TokenKind::RParen) &&
                !self.check(TokenKind::Done)
            {
                println!("curr = {:#?}", self.current());
                let arg = self.parse_expr(left.clone(), gc, BindingPower::Assignment);
                let Some(a) = arg else {
                    return None;
                };
                
                arg_list.push(a);
                
                if !self.match_token(TokenKind::Comma, gc) {
                    break;
                }
            }
        }

        if !self.consume(TokenKind::RParen, "')' expected after arguments".to_string(), gc)
        {
            self.advance(gc);
            return None;
        }

        return Some(arg_list.clone());
    }

    fn parse_function_call
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);
        let arg_list = self.parse_argument_list(left.clone(), gc);
       
        if arg_list.is_none() {
            return None;
        }

        Some(Box::new(Ast::Call(
                        left.clone(),
            arg_list.clone().unwrap(),
        )))
    }

    fn parse_positive
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);
        self.parse_expr(left, gc, BindingPower::Assignment)
    }

    fn parse_negative
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);

        let right = self.parse_expr(left, gc, BindingPower::Assignment);

        Some(Box::new(Ast::Negate(
                        right.clone(),
        )))
    }

    fn parse_lnot(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);

        let right = self.parse_expr(left, gc, BindingPower::Assignment);

        Some(Box::new(Ast::LNot(
                        right.clone(),
        )))
    }

    fn parse_not
    (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        self.advance(gc);
        
        let right = self.parse_expr(left, gc, BindingPower::Unary);

        Some(Box::new(Ast::Not(
	                right.clone(),
        )))
    }

    fn parse_addition(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }
        
        Some(Box::new(Ast::Add(
                        left.clone(),
            right.clone(),
        )))
    }

    fn parse_subtraction (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Subtract(
                        left.clone(),
            right.clone(),
        )))
    }
   
    fn parse_multiplication (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Multiply(
                        left.clone(),
            right.clone(),
        )))
    }

    fn parse_division (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Divide(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_modulo (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Modulo(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_power (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);

        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc,  BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Power(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_land (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1));
            if right.is_none() {
                return None;
            }
            if bp.is_some() {
                return Some(Box::new(Ast::TrueStmt),
                );
            }
            return Some(Box::new(Ast::FalseStmt));
        }
        None
    }

    fn parse_lor (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        let mut right = None;
        if bp.is_none() {
            return None;
        }
        
        if bp.is_some() {
            self.advance(gc);

            right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1));
        }

        if right.is_none() {
            return None;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1));
            if right.is_none() {
                return None;
            }
            if bp.is_some() {
                return Some(Box::new(Ast::TrueStmt),
                );
            }
            return Some(Box::new(Ast::FalseStmt)
            );
        }
        None
    }

    fn parse_and (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::And(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_or (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Or(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_xor (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::XOr(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_lshift (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::LShift(
	                left.clone(),
            right.clone(),
        )))
    }
    
    fn parse_rshift (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::RShift(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_addition(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }
        
        Some(Box::new(Ast::AddAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_subtraction (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::SubtractAssign(
	                left.clone(),
            right.clone(),
        )))
    }
   
    fn parse_compound_multiplication (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::MultiplyAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_division (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::DivideAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_modulo (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc  
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }
        
        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::ModuloAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_power (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::PowerAssign(
	                left.clone(),
            right.clone(),
        )))
    } 

    fn parse_compound_and (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::AndAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_or (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::OrAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_xor (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::XOrAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_compound_lshift (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,        
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::LShiftAssign(
	                left.clone(),
            right.clone(),
        )))
    }
    
    fn parse_compound_rshift (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,        
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::RShiftAssign(
	                left.clone(),
            right.clone(),
        )))
    }

    fn parse_assignment (
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc
    ) -> Option<Box<Ast>>
    {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(left.clone(), gc, BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1));
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Assign(
	                left.clone(),
            right.clone(),
        )))
    }

    // End Pratt Parser methods 
}




#[derive(PartialEq)]
enum PrinterState {
    Done,
    NotDoneYet,
}

struct AstPrinter;

impl<'a> AstPrinter {
    fn new() -> Self {
        Self {}
    }

    fn run(&mut self, root: &mut Option<Box<Ast>>) {
        let ret = self.print(root, 0);
        match ret {
            PrinterState::Done => {},
            PrinterState::NotDoneYet  => {
                panic!("\nAST printer is not done even though it should be"); 
            },
        }
    }

    fn print(&mut self, node: &mut Option<Box<Ast>>, level: usize) -> PrinterState {
        match node {
            Some(ref mut ast) => {
                self.indent(level);
                println!("(Ast Type: {}; Level: {})", ast, level);
                
                match **ast {
                    Ast::Program { ref mut listing } => {
                        println!("Listing:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in listing {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                    },
                    Ast::FunDecl { ref mut name, 
                                   ref mut fun_ty, 
                                   ref mut params, 
                                   ref mut listing, 
                                   .. } => {
               
                        self.indent(level + 1);
                        println!("Name: {}", name);
                        self.indent(level + 1);
                        println!("Type: {}", fun_ty);
                        self.indent(level + 1);
                        println!("Params:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in params {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                        self.indent(level + 1);
                        println!("Listing:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in listing {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                    },
                    Ast::LetDecl {ref mut name,  
                                  ref mut expr,
                                  ..} => {
                
                        self.indent(level + 1);
                        println!("Name: {}", name);
                        self.indent(level + 1);
                        println!("Expr:");
                        self.print(&mut Some(expr.clone()), level + 2);
                    }
                    Ast::ExprStmt {ref mut expr, ..} => {
                        self.indent(level + 1);
                        println!("Expr:");
                
                        self.indent(level + 1);
                        println!("{{");
                
                        self.print(&mut Some(expr.clone()), level + 1);
                
                        self.indent(level + 1);
                        println!("}}");
                    },
                    Ast::Block {ref mut listing} => {
                        self.indent(level + 1);
                        println!("Listing:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in listing {
                            self.print(&mut Some(i.clone()), level + 1);
                        }
                        self.indent(level + 1);
                        println!("}}");
                    }
                    Ast::Add(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::Subtract(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::Multiply(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::Divide(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::Modulo(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::Power(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::LAnd(
			ref mut left,
			ref mut right
		    ) |
                    Ast::LOr(
			ref mut left,
			ref mut right
		    ) |
                    Ast::And(
			ref mut left,
			ref mut right
		    ) |
                    Ast::Or(
			ref mut left,
			ref mut right
		    ) | 
                    Ast::XOr(
		        ref mut left,
			ref mut right
		    ) |
                    Ast::LShift(
			ref mut left,
		        ref mut right,
		    ) |
                    Ast::RShift(
			ref mut left,
			ref mut right
		    ) |
                    Ast::Assign(
			ref mut left,
			ref mut right,
		    ) | 
                    Ast::AddAssign(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::SubtractAssign(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::MultiplyAssign(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::DivideAssign(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::ModuloAssign(
			ref mut left,
		        ref mut right,
		    ) |
                    Ast::PowerAssign(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::AndAssign(
			ref mut left,
			ref mut right
		    ) |
                    Ast::OrAssign(
			ref mut left,
		        ref mut right
		    ) | 
                    Ast::XOrAssign(
			ref mut left,
			ref mut right
		    ) |
                    Ast::LShiftAssign(
			ref mut left,
			ref mut right,
		    ) |
                    Ast::RShiftAssign(
			ref mut left,
			ref mut right
		    ) => {
                        self.indent(level + 1);
                        println!("Left:");
                        self.print(&mut left.clone(), level + 1);
                        self.indent(level + 1);
                        println!("Right:");
                        self.print(&mut right.clone(), level + 1);
                    }
                    Ast::Value {ref mut val, ..} => {
                        self.indent(level + 1);
                        println!("Value: {}", val);
                    },
                    Ast::Variable {
                        ref mut name,
                        ref mut arity,
                        ..
                    } => {
                        self.indent(level + 1);
                        println!("Name: {}", name); 
                    },
                    Ast::Call(ref mut recv, ref mut args) => {
                        self.indent(level + 1);
                        println!("Receiver:");
                        self.print(&mut recv.clone(), level + 1);
                        self.indent(level + 1);
                        println!("Args:");
                        self.indent(level + 1);
                        println!("{{");
                        for i in 0..args.len() {
                            self.print(&mut Some(args[i].clone()), level + 1);
                            if i < args.len() - 1 {
                                self.indent(level + 1);
                                println!(",");
                            }
                        }
                        self.indent(level + 1);
                        println!("}}");
                    },
                    _ => { panic!("unrecognized node type"); }
                }
            },
            None => {
                panic!("node doesn't exist");
            },
        }
                

        if level == 0 {
            PrinterState::Done
        } else {
            PrinterState::NotDoneYet
        }
    }

    fn indent(&mut self, level: usize) {
        for _i in 0..level * 2 {
            print!(" ");
        }
    }
}

struct Compiler<'a> {
    symbols: SymbolTables<'a>,
    compile_flag: bool,
    error_count: u8,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,
}

impl<'a> Compiler<'a> {
    fn new(compile: bool) -> Self {
        Self {
            symbols: SymbolTables::new(),
            compile_flag: compile,
            error_count: 0u8,
            in_panic_mode: false,
            in_panic_lock_mode: false,
        }
    }

	fn declare_mod(
		&mut self, 
		name: String, 
		parent: Option<&Module>
	) -> Result<(), String> {
		if self.symbols.mods.is_empty() {
            if name == ".".to_string() {
				self.symbols.mods.insert(".".to_string(), Module::new("".to_string(), self.symbols.mods.len(), None));

				if let Some(res) = self.symbols.mods.get_mut(&name) {
					self.set_curr_mod(res.name());
				}
				return Ok(());
			}
		}
		if let Some(res) = parent {
            if res.name() == ".".to_string() {
			    return Err("global module cannot be a parent of any other module".to_string())
            }
            panic!("There should be a module here!");
		} else {
			self.symbols.mods.insert(name, Module::new(name, self.symbols.mods.len(), parent));
			if let Some(res) = self.symbols.mods.get_mut(&name) {
				return self.set_curr_mod(name);
			};
		}

	}
		
	fn set_curr_mod(
		&mut self, 
		name: String
	) -> Result<(), String> {
		if let Some(mut res) = self.symbols.mods.get_mut(name) {
			self.symbols.curr_mod = &res;
			Ok(())
		} else {
			Err("cannot set current module to one which is invalid")
		}
	}
	
	fn get_curr_mod(
		&self,
	) -> Option<&Module> {
		self.symbols.get_curr_mod()
	}

	fn get_mod(
		&mut self,
		name: String
	) {
	}
	
	fn declare_curr_mod_var(
		&mut self,
		parser: &mut Parser,
		name: String,
	) -> Result<VarEntry, VarError> {
		let curr_mod = self.symbols.get_curr_mod();
		if let Some(res) = curr_mod.mod_vars.get_mut(name) {
			Err(VarError::new(
				name.clone(),
				VarErrorType::AlreadyExists
			))
		} else {
			let mut access_type = AccessType::Public;
			if *name.chars().nth(0) == '_' {
				access_type = AccessType::Private;
			}
			let var = VarEntry::new(
				name.clone(),
				VarType::Var,
				ScopeType::Module,
				access_type.clone(),
				self.symbols.get_curr_mod().mod_vars.len(),		
			);
		}
	}
	
    fn get_scope_depth(
        &self
    ) -> usize {
        self.symbols.calls.len() - 1;
    }

    fn compile
    (
        &mut self,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        //let vm_ptr : *mut VM = ptr::from_mut(vm);
		if !self.compile_flag {
            self.code_gen(ast, gc)
		} else {
	    	//self.code_gen_c(ast, vm, gc)
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
                eprintln!("error: too many errors");
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
    fn code_gen_param
    (
        &mut self,
        ast: &Ast,
        vm: *mut VM,
        gc: &Gc,
    ) -> bool
    {
        match *ast {
            Ast::Variable{ref name, ..} => {
                if let Some(ref mut last) = self.symbols.calls.last_mut()
                {
                    let var =
                        unsafe {
                            add_local_var(self, ast, gc, name, None, true)
                        };
                    match var {
                        Ok(var_info) => {
                            unsafe {
                                define_var(vm, last, 0)
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
    fn code_gen
    (
        &mut self,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
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
                    self.code_gen_fun_decl(ast, name.to_string(), fun_ty.clone(), params, listing.to_vec(), closure)
                },
                Ast::LetDecl{
					ref name,
                    ref expr,
				} => {
                    self.code_gen_let_decl(name.to_string(), expr.clone())
                },
                Ast::ReturnStmt{ref expr, ..} => {
                    let last = unsafe {
                        self.symbols.calls.last_mut().unwrap()
                    };
                    
                    last.closure
                },
                Ast::Call(
		            ref recv,
		            ref args,
                ) => {
                    self.code_gen_call(&*ast.clone().unwrap(), gc, &recv, args)
                },
                Ast::Block {
                    ref listing,
                    ..
                } => { 
                    for i in listing {
                        self.code_gen(&mut Some(i.clone()), vm, gc);
                    }
                    self.symbols.calls.last_mut().unwrap().closure
                },
                Ast::ExprStmt{
                    ref expr
                } => {
                    self.code_gen(&mut Some(expr.clone()), vm, gc)
                },
                Ast::Value {ref val, ..} => {
                    self.code_gen_value(val.clone())
                },
                Ast::Variable{
                    ref name,
                    ref arity,
                } => {
                    self.code_gen_var(self.gc, name.to_string(), arity.clone())
                },
                Ast::Add(..) |
                Ast::Subtract(..) |
                Ast::Multiply(..) |
                Ast::Divide(..) |
                Ast::Modulo(..) |
                Ast::Power(..) => {
                    self.emit_math_op(&mut Some(node.clone()), vm, gc);
                    self.symbols.calls.last_mut().unwrap().closure.clone()
                },
                Ast::Assign(
                    ref left,
                    ref right,
                ) => {
                    self.code_gen(right, vm, gc);
                    if let Some(l) = left {
                        match **l {
                            Ast::Variable {
                                ref name,
                                ref arity,
                            } => {
                                let var = unsafe{
                                    resolve_var(self, vm, &**l, gc, name, *arity, true);
                                };
                                match var {
                                    Ok(v) => {
                                        if let Some(last) = self.symbols.calls.last_mut()
                                        {
                                            last.emit_word_op(OpCode::StoreVar, v.index as usize);
                                            last.closure
                                        } else {
                                            panic!("function compiler stack is empty");
                                        }
                                    },
                                    Err(e) => {
                                        if e.error_type == 
                                            VarErrorType::Undefined
                                        {
                                            self.error(&**l, 
                                                format!("variable {} is undefined", e.name))
                                        }
                                    },
				                }
			                    None
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
            }
        } else {
            eprintln!("ast node doesn't exist");
            None
        }
    }
    
    fn code_gen_fun_decl(
        &mut self,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        let mut result: Option<Managed<Closure>> = None;
        if let Some(node) = ast {
            match *ast {
                Ast::FunDecl {..} => { 
                    let ty = node.fun_ty.clone();
                    match ty {
                        FunctionType::TopLevel => {
                            unsafe{push_main(self)};
                            unsafe{self.symbols.calls.last_mut().unwrap().closure = Some(node.closure.clone())};
                        },
                        _ => {
                            unsafe {push_function(self)};
                            unsafe {self.symbols.calls.last_mut().unwrap().closure = Some(node.closure.clone())};
                        },
                    }

                    result = unsafe{self.symbols.calls.last_mut().unwrap().closure.clone()};
                    
                    if ty != FunctionType::TopLevel {
                        self.begin_scope();
                    }
                    
                    for i in node.params {
                        let ret = self.code_gen_param(&i.clone(), gc);
                        if !ret {
                            result = None;
                        }
                    }
                    
                    
                    for i in node.listing {
                        println!("here");
                        let ret = self.code_gen(Some(i.clone()), vm, gc);
                        println!("ret = {:#?}", ret);
                        if ret.is_none() {
                            result = None;
                        }
                    }
                    
                    if ty != FunctionType::TopLevel {
                        unsafe {self.end_scope()};
                    }

                    if self.error_count == 0 {
                        if ty == FunctionType::NamedClosure ||
                            ty == FunctionType::AnonClosure
                        { 
                            let fun_com =
                                &mut self.symbols.calls.last_mut().unwrap();
                            
                            let prev_fun_com = 
                                &mut (&mut self.symbols.calls)[self.symbols.calls.len() - 2];
                            if let Some(closure) = fun_com.closure {
                                let constant = prev_fun_com.get_constant(&Value::Closure(closure));
                                prev_fun_com.emit_word_op(OpCode::LoadValue, constant as usize);
                            } else {
                                panic!("internal closure doesn't exist");
                            }
                            
                            if ty == FunctionType::NamedClosure {
                                let last = 
                                    self.symbols.calls.last_mut().unwrap();

                                let scope = 
                                    self.get_scope_depth();
                                let var: Result<VarInfo, VarError> = 
                                    Err(VarError::new(
                                        node.name.clone(),
                                        VarErrorType::Undefined,
                                    ));
                                let fun =
                                    self.add_var(
                                        node.name, Some(node.params.len()), true
                                    );
                                
                                match fun {
                                    Ok(var_info) => {
                                        println!("here!");
                                        unsafe {
                                            define_var(vm, prev_fun_com, var_info.index)
                                        };
                                    },
                                    _ => {},
                                }  
                            }                
                        }
                        if result.is_some() { 
                            result = self.code_gen_return(ast, vm, gc);
                        }
                        unsafe{self.symbols.calls.pop()};
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
                                eprintln!("Found {} errors.", self.error_count);
                            } else if self.error_count == 1 {
                                eprintln!("Found 1 error.");
                            } else {
                                panic!("error count should not be zero");
                            }
                        }
                        return None;
                    }
                }
            }
        }
    }

    fn code_gen_let_decl(
        &mut self, 
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        let last = 
            self.symbols.calls.last_mut().unwrap();
        let scope =
            self.get_scope_depth();

        let mut var: Result<VarInfo, VarError> =
            Err(VarError::new(
                "".to_string(),
                VarErrorType::Undefined,
            ));
            
        let mut arity: Option<usize> = None;
        if let Some(expr) = ast {
            let function = match *expr {
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

            if let Some(node) = ast {
                var = self.add_var(vm, self, node, gc, node.name, arity, true);
            }

            if self.code_gen(Some(expr.clone()), vm, gc).is_none() {
                return None;
            }
        }            
        match var {
            Ok(var_info) => {
                unsafe{define_var(vm, last, var_info.index)};
                return last.closure;
            },
            Err(var_error) => {
                None
            }
        }
    }

    fn code_gen_value(
        &mut self,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        if let Some(last) = unsafe{self.symbols.calls.last_mut()} {
            if let Some(node) = ast {
                match *node {
                    Ast::Value{ val } => {
                        match val {
                            Value::Int(i) => {
                                #[cfg(not(feature = "reg_mach"))]
                                if i >= 0 && i <= 10 {
                                    last.emit_op(OpCode::from(u8::from(OpCode::Load0) + (i as u8)));
                                } else {
                                    let constant = last.get_constant(&val);
                                    last.emit_word_op(OpCode::LoadValue, constant as usize);
                                }

                                #[cfg(feature = "reg_mach")]
                                if i >= -32768 && i <= 32767 {
                                    last.emit_op(OpCode::LoadI16(self.curr_reg(), *i as i16));
                                } else {
                                    last.emit_op(OpCode::LoadReg(self.curr_reg(), self.next_avail_reg()));
                                }
                                self.symbols.calls.last_mut().unwrap().closure.clone()
                            },
                            Value::Float(_) => {
                                let constant = last.get_constant(&val);
                                last.emit_word_op(OpCode::LoadValue, constant as usize);
                                self.symbols.calls.last_mut().unwrap().closure.clone() 
                            },
                            Value::Bool(b) => {
                                if b == true {
                                    last.emit_op(OpCode::LoadTrue);
                                } else {
                                    last.emit_op(OpCode::LoadFalse);
                                }
                                self.symbols.calls.last_mut().unwrap().closure.clone()
                            },
                            Value::Nil => {
                                last.emit_op(OpCode::LoadNil);
                                self.symbols.calls.last_mut().unwrap().closure.clone() 
                            },
                            Value::String(_) => {
                                let constant = last.get_constant(&val);
                                last.emit_word_op(OpCode::LoadValue, constant as usize);
                                self.symbols.calls.last_mut().unwrap().closure.clone() 
                            },
                            _ => {
                                self.error(&node, "value is not a valid constant".to_string());
                                None
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("function compiler stack is empty");
            None
        }
    }

    fn code_gen_var
    (
        &mut self,
        ast: Option<Box<Ast>>, 
        vm: &mut VM,
        gc: &Gc,
    )
    {
        if let Some(node) = ast {
            let var =
                self.resolve_mod_var(vm, &node, gc, node.name, None, true);
            match var {
                Err(e) => {
                    if e.error_type == VarErrorType::Undefined {
                        self.error(&node, format!("variable {} is undefined", e.name));
                    }
                    return None;
                }
                Ok(v) => {
                    if let Some(last) = 
                        self.symbols.calls.last_mut()
                    {
                        self.emit_var_load(last, v);
                        return self.symbols.calls.last_mut().unwrap().closure;
                    };
                },
            };
        }
        None
    }

    fn code_gen_call
    (
        &mut self,
        compiler: &mut Compiler,
        ast: &Ast,
        recv: Option<Box<Ast>>,
        args: &Vec<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        if let Some(last) = unsafe{self.symbols.calls.last_mut()} {
            match *recv.clone().unwrap() {
                Ast::Variable {
                    ref name,
                    ref arity,
                } => {
                    let var = unsafe {resolve_var(self, vm, ast, gc, name, Some(args.len()), false)};
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

            unsafe {self.symbols.calls.last_mut().unwrap().closure.clone()}
        } else {
            panic!("no function compilers left on stack");
        }
    }
    
    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen_return
    (
        &mut self,
        ast: Option<Box<Ast>>,
        compiler: &Compiler,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        if let Some(node) = ast {
            match *node {
                Ast::FunDecl {
		            ref name,
                    ref fun_ty,
                    ref params,
                    ref listing,
                    ref closure
                } => {
                    if let Some(last) = compiler.symbols.calls.last_mut().unwrap()
                    { 
                        if self.error_count == 0 {
                            if listing.is_empty() {
                                last.emit_op(OpCode::LoadNil);
                                last.emit_op(OpCode::Return);
                            }
                            if let Some(mut l) = listing.last() { 
                                match **l {
                                    Ast::ExprStmt{..} => {
                                        l.emit_op(OpCode::Return);
                                    },
                                    Ast::ReturnStmt{ref expr, ..} => {
                                        if let Some(e) = expr {
                                            self.code_gen(Some::<Box<Ast>>(e.clone()), vm, gc);
                                        } else {
                                            last.emit_op(OpCode::LoadNil);
                                        }
                                        last.emit_op(OpCode::Return);
                                    },
                                    _ => {},
                                }
                            }
                            let c = unsafe {self.symbols.calls.last_mut().unwrap().closure};
                            if c.is_some() {
                                c.unwrap().get_core().finish(
                                    unsafe {self.symbols.calls.last_mut().unwrap().constant_arr.clone()},
                                    unsafe {self.symbols.calls.last_mut().unwrap().code.clone()}
                                );
                                return c;
                            }
                            return None;
                        } else {
                            return None;
                        }
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

    fn emit_math_op
    (
        &mut self,
        ast: &mut Option<Box<Ast>>,
        vm: *mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>>
    {
        if let Some(node) = ast {
            if let Some(last) = unsafe{self.symbols.calls.last_mut()} {
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
    /*
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
		if let Some(ref mut last) = unsafe{self.symbols.calls.last_mut()}
		{
		    let var =
			unsafe {
			    add_local_var(self, ast, gc, name, None, true)
			};
		    match var {
			Ok(_) => {
			    unsafe {
				define_var(vm, last, 0)
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
        ast: Option<Box<Ast>>,
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
			    
			    unsafe{push_main(self, &node, gc)};
			    unsafe{self.symbols.calls.last_mut().unwrap().closure = Some(closure.clone())};
			},
			_ => {
			    unsafe {push_function(self, &node, gc)};
			    unsafe {self.symbols.calls.last_mut().unwrap().closure = Some(closure.clone())};
			},
		    }

		    result = unsafe{self.symbols.calls.last_mut().unwrap().closure.clone()};
		    
		    if ty != FunctionType::TopLevel {
			unsafe {begin_scope()};
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
			unsafe {end_scope()};
		    }

		    if self.error_count == 0 {
			if ty == FunctionType::NamedClosure ||
			    ty == FunctionType::AnonClosure
			{
			    if let Some(last) =
				unsafe{self.symbols.calls.last_mut()}
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
				    &mut self.symbols.calls.last_mut().unwrap()
				};
			    
			    let prev_fc = 
				unsafe {
				    &mut (&mutself.symbols.calls)[self.symbols.calls.len() - 2]
				};
			    if let Some(closure) = fc.closure {
				let constant = prev_fc.get_constant(&Value::Closure(closure));
				prev_fc.emit_word_op(OpCode::LoadValue, constant as usize);
			    } else {
				panic!("internal closure doesn't exist");
			    }
			    if ty == FunctionType::NamedClosure {
				let last = unsafe {
				    self.symbols.calls.last_mut().unwrap()
				};
				let scope = unsafe {
				    get_scope_depth()
				};
				let mut var: Result<VarInfo, VarError> = 
				    Err(VarError::new(
					name.clone(),
					VarErrorType::Undefined,
				    ));
				println!("scope = {}", scope);
				if scope == 0 {
				    var = unsafe {
					add_ns_var(vm, self, &node, name, 
								  Some(num_params), true
					)
				    };
				    match var {
					Ok(var_info) => {
					    unsafe {
						define_var(vm, prev_fc, var_info.index);
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
						define_var(vm, fc, 0)
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
			unsafe{self.symbols.calls.pop()};
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
			self.symbols.calls.last_mut().unwrap()
		    };
		    let scope = unsafe {
			get_scope_depth()
		    };
		    let mut var: Result<VarInfo, VarError> =
			Err(VarError::new(
			    "".to_string(),
			    VarErrorType::Undefined,
			));
		    println!("scope = {}", scope);
		    if scope == 0 {
			var = unsafe {add_ns_var(vm, self, &node, name, None, true)};
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
			    unsafe{define_var(vm, last, var_info.index)};
			    return last.closure;
			},
			Err(var_error) => {
			    None
			}
		    }
		}
		Ast::ReturnStmt{ref expr} => {
		    let last = unsafe {
			self.symbols.calls.last_mut().unwrap()
		    };
		    
		    last.closure
		},
		Ast::Block {
		    ref listing
		} => { 
		    for i in listing {
			self.code_gen_c(&mut Some(i.clone()), vm, gc);
		    }
		    unsafe {self.symbols.calls.last_mut().unwrap().closure}
		},
		Ast::ExprStmt{
		    ref expr
		} => {
		    self.code_gen_c(&mut Some(expr.clone()), vm, gc)
		},
		Ast::Value {
		    ref val
		} => {
		    if let Some(last) = unsafe{self.symbols.calls.last_mut()} {
			match val {
			    Value::Int(i) => {
				if *i >= 0 && *i <= 10 {
				    last.emit_op(OpCode::from(u8::from(OpCode::Load0) + (*i as u8)));
				} else {
				    let constant = last.get_constant(val);
				    last.emit_word_op(OpCode::LoadValue, constant as usize);
				}
				return unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone();
			    },
			    Value::Float(_) => {
				let constant = last.get_constant(val);
				last.emit_word_op(OpCode::LoadValue, constant as usize);
				return unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone(); 
			    },
			    Value::Bool(b) => {
				if *b == true {
				    last.emit_op(OpCode::LoadTrue);
				} else {
				    last.emit_op(OpCode::LoadFalse);
				}
				return unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone(); 
			    },
			    Value::Nil => {
				last.emit_op(OpCode::LoadNil);
				return unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone(); 
			    },
			    Value::String(_) => {
				let constant = last.get_constant(val);
				last.emit_word_op(OpCode::LoadValue, constant as usize);
				return unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone(); 
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
			resolve_var(self, vm, &node, gc, name, None, true)
		    };
		    match var {
			Err(e) => {
			    self.error(&node, format!("variable {} is undefined", e.name));
			    return None;
			}
			Ok(_) => {},
		    };
		    
		    unsafe {self.symbols.calls.last_mut().unwrap().closure}
		},
		Ast::Add(..) |
		Ast::Subtract(..) |
		Ast::Multiply(..) |
		Ast::Divide(..) |
		Ast::Modulo(..) |
		Ast::Power(..) => {
		    self.emit_math_op(&mut Some(node.clone()), vm, gc);
		    unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone()
		},
		Ast::Call(ref recv, ref args) => {
		    if let Some(last) = unsafe{self.symbols.calls.last_mut()} {
			if self.code_gen_c(&mut recv.clone(), vm, gc).is_none() {
			    return None;
			}
			for i in &*args {
			    self.code_gen_c(&mut Some(i.clone()), vm, gc);
			} 
			last.emit_byte_op(OpCode::Call, args.len() as u8);
			println!("closure = {:#?}", unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone());
			return unsafe{self.symbols.calls.last_mut()}.unwrap().closure.clone()
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
        ast: Option<Box<Ast>>,
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
			if let Some(last) = unsafe{self.symbols.calls.last_mut()} {
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
			let c = unsafe {self.symbols.calls.last_mut().unwrap().closure};
			if c.is_some() {
			    c.unwrap().get_core().finish(
				unsafe {self.symbols.calls.last_mut().unwrap().constant_arr.clone()},
				unsafe {self.symbols.calls.last_mut().unwrap().code.clone()}
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
	} else {
	    None
        }
    }*/
}




#[derive(Clone)]
struct SymbolTables<'a> {
    mods: HashMap<String, Module<'a>>,
    curr_mod: &'a Module<'a>,
    calls: Vec<FunCompiler>,
}

impl <'a>SymbolTables<'a> {
    fn new() -> Self {
        Self {
            mods: HashMap::<String, Module<'a>>::new(),
            calls: vec![],
        }
    }

    fn get_mod(&mut self, name: String) -> Result<&'a Module<'a>, String> {
        let res = self.mods.get(name);
        if res.is_some() {
            Ok(res.unwrap())
        } else {
            Err(format!("Module '{}' doesn't exist", name))
        }
    }
}

struct Module<'a> {
    mod_name: String,
    mod_index: usize,
    mod_parent: Option<&'a Module<'a>>,
    mod_vars: HashMap<String, VarEntry>,
}

impl<'a> Module<'a> {
    fn new(
        name: String,
        index: usize,
        parent: Option<&'a Module<'a>>,
    ) -> Self {
        Self {
            mod_name: name,
            mod_index: index,
            mod_parent: parent,
            mod_vars: HashMap::<String, VarEntry>::new(),
        }
    }

    fn name(&self) -> String {
        self.mod_name.clone()
    }

    fn index(&self) -> usize {
        self.mod_index
    }

    fn parent(&self) -> Option<&'a Module> {
        self.mod_parent.clone()
    }

    fn add_var(&mut self, name: String, var: VarEntry) {
    }

    fn get_var(
        &mut self, 
        name: String
    ) -> VarEntry {
    }
}


pub struct VarEntry {
    name: String,
    var_type: VarType,
    scope_type: ScopeType,
    access_type: AccessType,
    var_index: usize,
}

impl VarEntry {
    fn new(
    	name: String,
    	var_type: VarType,
    	scope_type: ScopeType,
    	access_type: AccessType,
    	var_index: usize
    ) -> Self {
        Self {
            name,
            var_type,
            scope_type,
            access_type,
            var_index,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_var_type(&self) -> VarType {
        self.var_type.clone()
    }
    
    pub fn get_scope_type(&self) -> ScopeType {
    	self.scope_type.clone()
    }
    
    pub fn get_access_type(&self) -> AccessType {
    	self.access_type.clone()
    }
    
    pub fn get_index(&self) -> usize {
        self.var_index
    }
}

// Begin FunCompiler Stuff

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
            parser: &mut Parser,
            compiler: &mut Compiler,
            ast: &Ast,
            gc: &Gc,
	) -> Self 
    {
        Self::function_setup(parser, ast, gc, "<main>", 0, 0, FunctionType::TopLevel)
    }

    fn new_named_closure
	(
            compiler: &mut Compiler,
            ast: &Ast,
            gc: &Gc,
            name: &str, 
            arity: usize, 
            scope: usize,
	) -> Self
    {
        
        Self::function_setup(compiler, ast, gc, name, arity, scope, FunctionType::NamedClosure)
    }

    fn new_anon_closure
	(
            compiler: &mut Compiler,
            ast: &Ast,
            gc: &Gc,
            arity: usize,
            scope: usize,
	) -> Self
    {
        Self::function_setup(compiler, ast, gc, "", arity, scope, FunctionType::AnonClosure)
    }

    /*fn new_function
    (
    compiler: &mut Compiler,
    name: &str,
    arity: usize,
) -> Self
    {
}
    
    fn new_constructor
    (
    compiler: &mut Compiler,
    arity: usize
) -> Self
    {
}
    
    fn new_method
    (
    	compiler: &mut Compiler,
    	name: &str,
    	arity: usize
	) -> Self 
    {
	}*/
    
    fn function_setup
	(
        compiler: &mut Compiler,
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
                compiler, ast, gc, "this".to_string(), 
                scope, false
            );
        } else {
            s.push_local(
                compiler, ast, gc, "".to_string(), 
                scope, false
            );
        } 
        s
    }

    fn resolve_local_internal
	(
        &mut self,
        compiler: *mut Compiler,
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
                        self.error(ast, "self-initialization of local variable '".to_string() + &mut self.locals[i].name)
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
        parser: &mut Parser,
        compiler: &mut Compiler,
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

            let local = fun_compiler.resolve_local_internal(compiler, ast, gc, name, arity, true);
            match local {
                Ok(var_info) => {
                    if let Some(fun_compiler) = fc {
                        fun_compiler.locals[var_info.index].upvalue = true;
                        return fun_compiler.add_upvalue(compiler, ast, gc, &name, arity, var_info.upvalue_local_index, true);
                    }
                },
                _ => {},
            }

            fc = iter.next();
            if fc.is_some() {
                if let Some(fun_compiler) = fc {
                    let upvalue = fun_compiler.resolve_upvalue_internal(compiler, parser.vm, ast, gc, name, arity, false, iter);
                    
                    match upvalue {
                        Ok(var_info) => {
                            if let Some(last) = self.symbols.calls.last_mut() {
                                return last.add_upvalue(compiler, ast, gc, name, arity, var_info.upvalue_local_index, false);
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
        compiler: *mut Compiler, 
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
            unsafe {self.error(ast, "too many upvalues".to_string())};
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
        compiler: &mut Compiler,
        ast: &Ast,
        _gc: &Gc,
        name: String,
        scope: usize,
        upvalue: bool
	) -> Option<&mut Local>
    {
        if self.locals.len() == MAX_LOCALS {
            unsafe {self.error(ast, "too many locals in function".to_string())};
            return None;
        }
        self.locals.push(Local::new(name.to_string(), scope, upvalue));

        self.locals.last_mut()
    }

    fn declare_local
	(
        &mut self, 
        compiler: &mut Compiler,
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
                            self.error(
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
        self.push_local(compiler, ast, gc, actual_name.clone(), scope, false);
        
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
        compiler: &mut Compiler,
        ast: &Ast,
        gc: &Gc,
        name: &String,
        arity: usize,
	    print_err: bool,
	) -> Result<VarInfo, VarError>
    {
        let fun_name = format!("{}(#{})", name, arity as u32);
        if self.local_fun_symtab.get(&fun_name).is_some() {
            unsafe {self.error(ast, "local function '".to_string() + &fun_name + "' already exists")};
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

    fn get_scope_depth(&mut self) -> usize {
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

#[derive(Clone, PartialEq)]
enum VarType {
	Unknown,
    Var,
    Function,
    Class,
    Const,
}

#[derive(Clone, PartialEq)]
enum ScopeType {
    Module,
    Local,
    Upvalue,
    Class,
}

#[derive(Clone, PartialEq)]
enum AccessType {
	Public,
	Private,
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

    fn get_name(&mut self) -> String {
        self.name.clone()
    }

    fn get_arity(&mut self) -> Option<usize> {
        self.arity
    }

    fn get_var_type(&mut self) -> VarType {
        self.var_type.clone()
    }

    fn get_scope_depth(&mut self) -> usize {
        self.scope
    }

    fn get_index(&mut self) -> usize {
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

    fn get_name(&mut self) -> String{
        self.name.clone()
    }

    fn get_error_type(&mut self) -> VarErrorType {
        self.error_type.clone()
    }
}

// End FunCompiler Stuff











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
struct VM {
    mod_vec: Vec<Vec<Value>>,
    mod_index: usize,
    main_fun: Option<Managed<Closure>>,
    calls: CallStack,
    stack: Vec<Value>,
    sp: usize,
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
    fn get_pc(&mut self) -> Option<usize>{
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

    fn check_arity
    (
        &self,
        fun: &Value,
        num_args: usize,
        stack: Vec<Value>,
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
        let fun = &mut self.stack[fun_index];
        let args = &mut self.stack[fun_index..fun_index + (num_args as usize) + 1];
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
                        match self.stack[0].get_uintsize() {
                            Ok(v) => {
                                return Err(format!("var index out of range: {}; limit: {}", arg, self.mod_vec[v].len()));
                            },
                            Err(e) => {
                                panic!("internal error: {}", e);
                            }
                        }
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
        root: Option<Box<Ast>>,
        compiler: &mut Compiler<'_>,
        gc: &Gc,
        debug: bool,
        compile: bool,
    ) -> Result<Value, InterpretErrorType>
    {
        self.calls.clear();
        self.stack.clear();
        let closure = compiler.compile(root, compiler, gc);
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
        compiler: &mut Compiler<'_>,
        gc: &Gc,
        debug: bool,
        compile: bool,
    ) -> Result<Value, InterpretErrorType> {
        self.execute(compiler, gc, debug, compile)
    }
    
    pub fn print_backtrace(&mut self) {
        println!("{}", self.calls.get_backtrace());
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



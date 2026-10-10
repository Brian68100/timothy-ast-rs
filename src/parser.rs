mod ast_printer;

use crate::ast::*;
use crate::closure::*;
use crate::function_core::*;
use crate::gc::{Gc, NO_GC};
use crate::lexer::*;
use crate::managed::*;
use crate::natives::*;
use crate::opcode::*;
use crate::parser::ast_printer::*;
use crate::token::*;
use crate::value::*;
use crate::vm::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::iter::Rev;

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
    Assignment, // = += -= *= /= &= |= ^= <<= >>=

    Ternary,    // ? :
    LOr,        // ||
    LAnd,       // &&
    Equality,   // == !=
    Comparison, // < > <= >=
    Bitwise,    // & ^ |
    Shift,      // << >>
    Term,       // + -
    Factor,     // * /
    Unary,      // ! ~ + -
    Exponent,   // **
    Call,       // . () []
    Primary,
}

impl From<u8> for BindingPower {
    fn from(val: u8) -> Self {
        match val {
            0 => BindingPower::NoBp,
            1 => BindingPower::Assignment, // = += -= *= /= &= |= ^= <<= >>=
            2 => BindingPower::Ternary,    // ? :
            3 => BindingPower::LOr,        // ||
            4 => BindingPower::LAnd,       // &&
            5 => BindingPower::Equality,   // == !=
            6 => BindingPower::Comparison, // < > <= >=
            7 => BindingPower::Bitwise,    // & ^ |
            8 => BindingPower::Shift,      // << >>
            9 => BindingPower::Term,       // + -
            10 => BindingPower::Factor,    // * /
            11 => BindingPower::Unary,     // ! ~ + -
            12 => BindingPower::Exponent,  // **
            13 => BindingPower::Call,      // . (| ( ) [ ]
            14 => BindingPower::Primary,
            _ => panic!("Invalid binding power value {}", val),
        }
    }
}

impl Into<u8> for BindingPower {
    fn into(self) -> u8 {
        match self {
            BindingPower::NoBp => 0,
            BindingPower::Assignment => 1, // = += -= *= /= &= |= ^= <<= >>=
            BindingPower::Ternary => 2,    // ? :
            BindingPower::LOr => 3,        // ||
            BindingPower::LAnd => 4,       // &&
            BindingPower::Equality => 5,   // == !=
            BindingPower::Comparison => 6, // < > <= >=
            BindingPower::Bitwise => 7,    // & ^ |
            BindingPower::Shift => 8,      // << >>
            BindingPower::Term => 9,       // + -
            BindingPower::Factor => 10,    // * /
            BindingPower::Unary => 11,     // ! ~ + -
            BindingPower::Exponent => 12,  // **
            BindingPower::Call => 13,      // . () []
            BindingPower::Primary => 14,
        }
    }
}

struct LexerData {
    lexer: Option<Lexer>,
    curr: Token,
    prev: Token,
}

pub struct Parser {
    lexer_stack: Vec<LexerData>,
    error_count: usize,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,
}

impl Parser {
    pub fn new() -> Self {
        let mut s = Self {
            lexer_stack: vec![],
            in_panic_mode: false,
            in_panic_lock_mode: false,
            error_count: 0,
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
        gc: &Gc,
        vm: &mut VM,
        print_flag: bool,
        compile_flag: bool,
        debug_flag: bool,
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
        self.advance(&gc);

        let root: Option<Box<Ast>> = Some(Box::new(Ast::ClosureDecl {
            name: " main".to_string(),
            fun_ty: FunctionType::TopLevel,
            params: vec![],
            listing: vec![],
            closure: None,
        }));
        if !self.parse(&mut root, gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            if print_flag {
                AstPrinter::new().run(&mut root);
            }
            vm.run(&mut root, gc, debug_flag, compile_flag)
        }
    }
    pub fn parse_file(
        &mut self,
        fname: String,
        gc: &Gc,
        vm: &mut VM,
        print_flag: bool,
        compile_flag: bool,
        debug_flag: bool,
    ) -> Result<Value, InterpretErrorType> {
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
        self.advance(&gc);

        let root: Option<Box<Ast>> = Some(Box::new(Ast::ClosureDecl {
            name: " main".to_string(),
            fun_ty: FunctionType::TopLevel,
            params: vec![],
            listing: vec![],
            closure: None,
        }));

        if !self.parse(&mut root, &gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            if print_flag {
                AstPrinter::new().run(&mut root);
            }
            vm.run(&mut root, &gc, debug_flag, compile_flag)
        }
    }

    pub fn store_fun(
        &mut self,
        name: &str,
        arity: usize,
        fun: fn(gc: &Gc, args: &[Value]) -> Result<Value, String>,
    ) -> bool {
        false
    }

    fn advance(&mut self, gc: &Gc) {
        self.lexer_stack.last_mut().unwrap().prev =
            self.lexer_stack.last_mut().unwrap().curr.clone();
        loop {
            self.lexer_stack.last_mut().unwrap().curr = self
                .lexer_stack
                .last_mut()
                .unwrap()
                .lexer
                .as_mut()
                .unwrap()
                .scan(gc);
            if self.current().kind != TokenKind::Err {
                break;
            }
            let curr = self.current();
            self.error_at_curr(curr.lexeme)
        }
    }

    fn check(&mut self, tk: TokenKind) -> bool {
        self.current().kind == tk
    }

    fn match_token(&mut self, tk: TokenKind, gc: &Gc) -> bool {
        if self.current().kind != tk {
            return false;
        }
        self.advance(gc);
        return true;
    }

    fn consume(&mut self, tk: TokenKind, err_message: String, gc: &Gc) -> bool {
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

        eprint!(
            "{}:{}:{}: error",
            self.lexer_stack
                .last_mut()
                .unwrap()
                .lexer
                .as_mut()
                .unwrap()
                .file_name(),
            token.line,
            token.col
        );

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
    fn parse(&mut self, root: &mut Option<Box<Ast>>, gc: &Gc) -> bool {
        while self.current().kind != TokenKind::Done {
            let _tk = self.current().kind;
            let left: Option<Box<Ast>> = self.parse_stmt(None, gc);
            println!("left = {:#?}", left);
            if left.is_none() {
                continue;
            }

            match root {
                Some(ref mut val) => {
                    match **val {
                        Ast::ClosureDecl {
                            ref mut listing, ..
                        } => {
                            listing.push(left.unwrap());
                        }
                        _ => {}
                    };
                }
                _ => {
                    unreachable!();
                }
            };
        }

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
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
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
    fn call_stmt_fn(
        &mut self,
        tk: TokenKind,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        match tk {
            TokenKind::LetKw => self.parse_let_stmt(left, gc),
            TokenKind::ReturnKw => self.parse_return_stmt(left, gc),
            TokenKind::LBrace => self.parse_block(left, gc),
            TokenKind::IfKw => self.parse_if_stmt(left, gc),
            _ => None,
        }
    }

    // Calls the corresponding prefix function
    fn call_prefix_fn(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
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
            _ => None,
        }
    }

    // Calls the corresponding infix function
    fn call_infix_fn(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
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
            _ => None,
        }
    }

    fn stmt_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::LetKw => Some(BindingPower::NoBp),
            TokenKind::ReturnKw => Some(BindingPower::NoBp),
            TokenKind::LBrace => Some(BindingPower::NoBp),
            _ => None,
        }
    }

    fn prefix_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::Int
            | TokenKind::Float
            | TokenKind::String
            | TokenKind::Ident
            | TokenKind::TrueKw
            | TokenKind::FalseKw
            | TokenKind::NilKw => Some(BindingPower::Primary),
            TokenKind::Add | TokenKind::Subtract | TokenKind::LNot | TokenKind::Not => {
                Some(BindingPower::Unary)
            }
            TokenKind::AnonClosure => Some(BindingPower::Call),
            TokenKind::FunKw => Some(BindingPower::Call),
            _ => None,
        }
    }

    fn infix_bp(&mut self, tk: TokenKind) -> Option<BindingPower> {
        match tk {
            TokenKind::Add | TokenKind::Subtract => Some(BindingPower::Term),
            TokenKind::Multiply | TokenKind::Divide | TokenKind::Modulo | TokenKind::Power => {
                Some(BindingPower::Factor)
            }
            TokenKind::LAnd => Some(BindingPower::LAnd),
            TokenKind::LOr => Some(BindingPower::LOr),
            TokenKind::Or | TokenKind::And | TokenKind::XOr => Some(BindingPower::Bitwise),
            TokenKind::LShift | TokenKind::RShift => Some(BindingPower::Shift),
            TokenKind::Assign
            | TokenKind::AddAssign
            | TokenKind::SubtractAssign
            | TokenKind::MultiplyAssign
            | TokenKind::DivideAssign
            | TokenKind::ModuloAssign
            | TokenKind::PowerAssign
            | TokenKind::OrAssign
            | TokenKind::AndAssign
            | TokenKind::XOrAssign
            | TokenKind::LShiftAssign
            | TokenKind::RShiftAssign => Some(BindingPower::Assignment),
            TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => Some(BindingPower::Call),
            _ => None,
        }
    }

    fn parse_stmt(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let mut node = None;
        let tk = self.current().kind;
        let stmt = self.stmt_bp(tk.clone());
        if stmt.is_some() {
            node = self.call_stmt_fn(tk.clone(), None, gc);
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

    fn parse_expr(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
        bp: BindingPower,
    ) -> Option<Box<Ast>> {
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
        while <BindingPower as Into<u8>>::into(bp.clone())
            <= <BindingPower as Into<u8>>::into(infix.unwrap())
        {
            tk = self.current().kind;
            infix = self.infix_bp(tk);
            if infix.is_none() {
                return l;
            }
            l = self.call_infix_fn(l, gc);
        }

        l
    }

    fn parse_let_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);
        self.consume(
            TokenKind::Ident,
            "identifier expected after 'let'".to_string(),
            gc,
        );
        let ident = self.previous().lexeme;
        //self.advance(gc);
        self.consume(
            TokenKind::Assign,
            "assignment expected after identifier".to_string(),
            gc,
        );

        let e = self.parse_expr(None, gc, BindingPower::Assignment);
        if e.is_none() {
            return None;
        }

        if !self.consume(
            TokenKind::Semicolon,
            "';' expected after let statement".to_string(),
            gc,
        ) {
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
        _gc: &Gc,
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_class_decl(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        _gc: &Gc,
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_macro_def(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_ns_decl(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        _gc: &Gc,
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_import_cmd(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>> {
        None
    }

    // control flow statements

    // IF Statement

    fn parse_if_stmt(&mut self, left: Option<Box<Ast>>, gc: &Gc) -> Option<Box<Ast>> {
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

    fn parse_anon_closure(&mut self, _left: Option<Box<Ast>>, gc: &Gc) -> Option<Box<Ast>> {
        self.advance(gc);
        let mut params: Vec<Box<Ast>> = vec![];
        let mut code: Vec<Box<Ast>> = vec![];

        loop {
            if !self.check(TokenKind::RParen) {
                if !self.consume(TokenKind::Ident, "parameter name expected".to_string(), gc) {
                    return None;
                }
                let param = Box::new(Ast::VariableId {
                    name: self.previous().lexeme.clone(),
                    method: false,
                    arity: None,
                });
                params.push(param);

                self.match_token(TokenKind::Comma, gc);
            } else {
                break;
            }
        }
        if !self.consume(
            TokenKind::RParen,
            "`)` expected after parameters".to_string(),
            gc,
        ) {
            return None;
        }

        self.consume(
            TokenKind::LBrace,
            "`{` expected at start of lambda function body".to_string(),
            gc,
        );

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            let mut l = None;
            l = self.parse_stmt(l, gc);
            if let Some(v) = l {
                code.push(v);
            } else {
            }
        }

        if !self.consume(
            TokenKind::RBrace,
            "`}` expected at end of lambda function body".to_string(),
            gc,
        ) {
            return None;
        }
        let closure = Closure::new(
            gc.manage(FunctionCore::new_anon_closure(params.len()), &NO_GC),
            gc.manage(Vec::<Value>::new(), &NO_GC),
        );

        Some(Box::new(Ast::ClosureDecl {
            name: closure.get_core().get_name().clone(),
            fun_ty: closure.get_core().get_type().clone(),
            params,
            listing: code,
            closure: Some(closure),
        }))
    }

    fn parse_named_closure(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);
        let mut params: Vec<Box<Ast>> = vec![];
        let mut code: Vec<Box<Ast>> = vec![];

        if !self.consume(
            TokenKind::Ident,
            "function name expected after 'fun'".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        let fun_name = self.previous().lexeme.clone();

        if !self.consume(
            TokenKind::LParen,
            "'(' expected after function name".to_string(),
            gc,
        ) {
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

                let param = Box::new(Ast::VariableId {
                    name: self.previous().lexeme.clone(),
                    method: false,
                    arity: None,
                });
                params.push(param);
            }
            if !self.match_token(TokenKind::Comma, gc) {
                break;
            }
        }

        if !self.consume(
            TokenKind::RParen,
            "')' expected after parameters".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        if !self.consume(
            TokenKind::LBrace,
            "'{' expected at start of function body".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            let mut l = None;
            l = self.parse_stmt(l, gc);
            if let Some(v) = l {
                code.push(v);
            } else {
            }
        }

        if !self.consume(
            TokenKind::RBrace,
            "'}' expected at end of function body".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        if self.error_count > 0 {
            return None;
        }

        let closure = Closure::new(
            gc.manage(
                FunctionCore::new_named_closure(fun_name.clone(), params.len()),
                &NO_GC,
            ),
            gc.manage(Vec::<Value>::new(), &NO_GC),
        );
        Some(Box::new(Ast::ClosureDecl {
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
    ) -> Option<Box<Ast>> {
        self.advance(gc);
        let mut ret_expr: Option<Box<Ast>> = None;
        if !self.match_token(TokenKind::Semicolon, gc) {
            ret_expr = self.parse_expr(left, gc, BindingPower::Assignment);
            if ret_expr.is_none() {
                return None;
            }
        }

        self.advance(gc);

        return Some(Box::new(Ast::ReturnStmt { expr: ret_expr }));
    }

    fn parse_break_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_continue_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_block(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);

        let mut list: Vec<Box<Ast>> = vec![];

        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::Done) {
            let mut l: Option<Box<Ast>> = None;
            l = self.parse_stmt(l, gc);
            if let Some(i) = l {
                list.push(i.clone());
            } else {
                return None;
            }
        }

        if !self.consume(
            TokenKind::RBrace,
            "'}' expected at end of block".to_string(),
            gc,
        ) {
            self.advance(gc);
        }

        if self.error_count > 0 {
            return None;
        }

        Some(Box::new(Ast::Block { listing: list }))
    }

    fn parse_expr_stmt(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        // generate expression nodes
        let node = self.parse_expr(left, gc, BindingPower::Assignment);

        if node.is_none() {
            return None;
        }

        let es = Box::new(Ast::ExprStmt {
            expr: node.unwrap(),
        });

        if !self.consume(
            TokenKind::Semicolon,
            "';' expected after expression".to_string(),
            gc,
        ) {
            return None;
        }

        Some(es)
    }

    fn parse_int(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value { val: v }))
    }

    fn parse_float(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value { val: v }))
    }

    fn parse_var(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let name = self.current().lexeme;
        self.advance(gc);

        Some(Box::new(Ast::VariableId {
            name: name,
            method: false,
            arity: None,
        }))
    }

    fn parse_true(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let v = Value::Bool(true);
        self.advance(gc);
        Some(Box::new(Ast::Value { val: v }))
    }

    fn parse_false(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let v = Value::Bool(false);
        self.advance(gc);
        Some(Box::new(Ast::Value { val: v }))
    }

    fn parse_nil(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value { val: v }))
    }

    fn parse_string(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let v = self.current().value;
        self.advance(gc);
        Some(Box::new(Ast::Value { val: v }))
    }

    fn parse_argument_list(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Vec<Box<Ast>>> {
        let mut arg_list: Vec<Box<Ast>> = vec![];
        if !self.check(TokenKind::RParen) {
            while !self.check(TokenKind::RParen) && !self.check(TokenKind::Done) {
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

        if !self.consume(
            TokenKind::RParen,
            "')' expected after arguments".to_string(),
            gc,
        ) {
            self.advance(gc);
            return None;
        }

        return Some(arg_list.clone());
    }

    fn parse_function_call(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);
        let arg_list = self.parse_argument_list(left.clone(), gc);

        if arg_list.is_none() {
            return None;
        }

        Some(Box::new(Ast::Call {
            recv: left.clone(),
            args: arg_list.clone().unwrap(),
        }))
    }

    fn parse_positive(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);
        self.parse_expr(left, gc, BindingPower::Assignment)
    }

    fn parse_negative(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);

        let right = self.parse_expr(left, gc, BindingPower::Assignment);

        Some(Box::new(Ast::Negate(right.clone())))
    }

    fn parse_lnot(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);

        let right = self.parse_expr(left, gc, BindingPower::Assignment);

        Some(Box::new(Ast::LNot(right.clone())))
    }

    fn parse_not(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        self.advance(gc);

        let right = self.parse_expr(left, gc, BindingPower::Unary);

        Some(Box::new(Ast::Not(right.clone())))
    }

    fn parse_addition(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Add(left.clone(), right.clone())))
    }

    fn parse_subtraction(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Subtract(left.clone(), right.clone())))
    }

    fn parse_multiplication(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Multiply(left.clone(), right.clone())))
    }

    fn parse_division(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Divide(left.clone(), right.clone())))
    }

    fn parse_modulo(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Modulo(left.clone(), right.clone())))
    }

    fn parse_power(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);

        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Power(left.clone(), right.clone())))
    }

    fn parse_land(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(
                left.clone(),
                gc,
                BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1),
            );
            if right.is_none() {
                return None;
            }
            if bp.is_some() {
                return Some(Box::new(Ast::TrueStmt));
            }
            return Some(Box::new(Ast::FalseStmt));
        }
        None
    }

    fn parse_lor(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        let mut right = None;
        if bp.is_none() {
            return None;
        }

        if bp.is_some() {
            self.advance(gc);

            right = self.parse_expr(
                left.clone(),
                gc,
                BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1),
            );
        }

        if right.is_none() {
            return None;
        }

        if bp.is_some() {
            self.advance(gc);

            let right = self.parse_expr(
                left.clone(),
                gc,
                BindingPower::from(<BindingPower as Into<u8>>::into(bp.clone().unwrap()) + 1),
            );
            if right.is_none() {
                return None;
            }
            if bp.is_some() {
                return Some(Box::new(Ast::TrueStmt));
            }
            return Some(Box::new(Ast::FalseStmt));
        }
        None
    }

    fn parse_and(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::And(left.clone(), right.clone())))
    }

    fn parse_or(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Or(left.clone(), right.clone())))
    }

    fn parse_xor(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::XOr(left.clone(), right.clone())))
    }

    fn parse_lshift(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::LShift(left.clone(), right.clone())))
    }

    fn parse_rshift(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::RShift(left.clone(), right.clone())))
    }

    fn parse_compound_addition(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::AddAssign(left.clone(), right.clone())))
    }

    fn parse_compound_subtraction(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::SubtractAssign(left.clone(), right.clone())))
    }

    fn parse_compound_multiplication(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::MultiplyAssign(left.clone(), right.clone())))
    }

    fn parse_compound_division(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::DivideAssign(left.clone(), right.clone())))
    }

    fn parse_compound_modulo(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::ModuloAssign(left.clone(), right.clone())))
    }

    fn parse_compound_power(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::PowerAssign(left.clone(), right.clone())))
    }

    fn parse_compound_and(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::AndAssign(left.clone(), right.clone())))
    }

    fn parse_compound_or(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::OrAssign(left.clone(), right.clone())))
    }

    fn parse_compound_xor(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::XOrAssign(left.clone(), right.clone())))
    }

    fn parse_compound_lshift(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::LShiftAssign(left.clone(), right.clone())))
    }

    fn parse_compound_rshift(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::RShiftAssign(left.clone(), right.clone())))
    }

    fn parse_assignment(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Rc<RefCell<Ast>>>,
        gc: &Gc,
    ) -> Option<Box<Ast>> {
        let tk = self.current().kind;
        let bp = self.infix_bp(tk);
        if bp.is_none() {
            return None;
        }

        self.advance(gc);

        let right = self.parse_expr(
            left.clone(),
            gc,
            BindingPower::from(<BindingPower as Into<u8>>::into(bp.unwrap()) + 1),
        );
        if right.is_none() {
            return None;
        }

        Some(Box::new(Ast::Assign(left.clone(), right.clone())))
    }

    // End Pratt Parser methods
}

#[cfg(test)]

mod tests {
    use super::*;
    use crate::gc::Gc;

    #[test]
    fn test_add_ns_var() {
        let mut deps = prep_for_test();

        let mut code = "let var = 10;";
        let mut parser = Parser::new();
        let mut result = parser.parse_text(
            &mut code.to_string(),
            &mut deps.1,
            &mut deps.0,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Int(10)));

        code = "var;";
        result = parser.parse_text(
            &mut code.to_string(),
            &mut deps.1,
            &mut deps.0,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Int(10)));
    }

    fn test_add_two_and_two() {
        let mut deps = prep_for_test();

        let code = "2 + 2";
        let mut parser = Parser::new();
        let result = parser.parse_text(
            &mut code.to_string(),
            &mut deps.1,
            &mut deps.0,
            false,
            false,
            false,
        );
        assert_eq!(result, Ok(Value::Int(4)))
    }

    fn prep_for_test() -> (VM, Gc) {
        (VM::new(), Gc::new())
    }
}




#[derive(Clone)]
struct Module {
    mod_name: String,
    mod_index: usize,
    mod_parent_index: Option<usize>,
    mod_vars: HashMap<String, usize>,
    mod_var_vec: Vec<VarInfo>,
}

impl Module {
    fn new(name: String, index: usize, parent_index: Option<usize>) -> Self {
        Self {
            mod_name: name,
            mod_index: index,
            mod_parent_index: parent_index,
            mod_vars: HashMap::<String, usize>::new(),
            mod_var_vec: vec![],
        }
    }

    fn name(&self) -> String {
        self.mod_name.clone()
    }

    fn index(&self) -> usize {
        self.mod_index
    }

    fn parent_index(&self) -> usize {
        self.mod_parent_index.clone()
    }

    /*fn add_mod(
        &mut self,
        compiler: &mut Compiler,
        module: &mut Module,
    ) -> Result<VarInfo, VarError> {
        self.mod_children
            .insert(module.name(), self.mod_children.len());
        let mut child = self
            .mod_child_vec
            .insert_mut(self.mod_children.len() - 1, *module);
        if let Some(entry) = self.mod_children.get(&module.name().clone()) {
            Ok(VarInfo::new(
                module.name().clone(),
                None,
                VarType::Module,
                ScopeType::None,
                AccessType::None,
                module.index(),
                0,
                0,
            ))
        } else {
            Err(VarError::new(
                module.name().clone(),
                VarType::Module,
                ScopeType::None,
                AccessType::None,
                VarErrorType::Undefined,
                0,
            ))
        }
    }*/

    fn add_var(
        &mut self,
        compiler: &mut Compiler,
        ast: Option<Box<Ast>>,
        name: &String,
        arity: Option<usize>,
        do_print_on_error: bool,
    ) -> Result<VarInfo, VarError> {
        if let Some(node) = ast {
            let var: Result<VarInfo, VarError> = if compiler.get_scope_depth() == 0 {
                compiler.resolve_mod_var(&*node, arity, do_print_on_error)
            } else {
                let mut last_mut = compiler.symbols.calls.last_mut().unwrap();
                last_mut.resolve_local(compiler, &*node, &name, arity, do_print_on_error)
            };
            var
        } else {
            Err(VarError::default())
        }
    }
}

impl Default for Module {
    fn default() -> Self {
        Self {
            mod_name: "".to_string(),
            mod_index: 0usize,
            mod_parent_index: None,
            mod_vars: HashMap::<String, usize>::new(),
            mod_var_vec: vec![],
        }
    }
}


#[derive(Clone)]
struct SymbolTables {
    mod_indexes: HashMap<String, usize>,
    mods: Vec<Module>,
    curr_mod: Option<usize>,
    calls: Vec<FunCompiler>,
    curr_scope: usize,
    //curr_class: Option<Class>,
}

impl SymbolTables {
    fn new() -> Self {
        Self {
            mod_indexes: HashMap::<String, usize>::new(),
            mods: vec![],
            curr_mod: None,
            calls: vec![],
            curr_scope: 0usize,
        }
    }

    fn curr_module(&self) -> Option<usize> {
        self.curr_mod.clone()
    }

    fn call_stack(&self) -> &[FunCompiler] {
        &self.calls
    }

    fn scope(&mut self) -> usize {
        self.calls.last_mut().unwrap().get_scope_depth()
    }

    fn get_mod(&mut self, name: &String) -> Result<Option<Module>, String> {
        let res = self.mod_indexes.get(name);
        if res.is_none() {
            return Err(format!("Module '{}' doesn't exist", name));
        } else {
            let res2 = self.mods.get(*res.unwrap());
            if res2.is_none() {
                Err(format!("Module '{}' doesn't exist", name))
            } else {
                Ok(Some(res2.unwrap()))
            }
        }
    }
}

#[derive(Clone)]
struct FunCompiler {
    name: String,
    ty: FunctionType,
    min_scope: usize,
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
    fn new_main(compiler: &mut Compiler, ast: &Ast, gc: &Gc) -> Self {
        if compiler.symbols.calls.len() > 0 {
            panic!("attempt to create new main with nonempty call stack");
        }
        Self::function_setup(compiler, ast, gc, "<main>", 0, 0, FunctionType::TopLevel)
    }

    fn new_named_closure(
        compiler: &mut Compiler,
        ast: &Ast,
        gc: &Gc,
        name: &str,
        arity: usize,
        scope: usize,
    ) -> Self {
        if compiler.symbols.calls.len() == 0 {
            panic!("no main function when named closure was declared");
        }
        Self::function_setup(
            compiler,
            ast,
            gc,
            name,
            arity,
            scope,
            FunctionType::NamedClosure,
        )
    }

    fn new_anon_closure(
        compiler: &mut Compiler,
        ast: &Ast,
        gc: &Gc,
        arity: usize,
        scope: usize,
    ) -> Self {
        if compiler.symbols.calls.len() == 0 {
            panic!("no main function when anonymous closure was declared");
        }
        Self::function_setup(
            compiler,
            ast,
            gc,
            "",
            arity,
            scope,
            FunctionType::AnonClosure,
        )
    }

    /*fn new_function
    (
        compiler: &mut Compiler,
        name: &str,
        arity: usize,
    ) -> Self
    {
        if compiler.symbols.calls.len() == 0 {
            panic!("no main function when function was declared");
        }
    }*/

    /*fn new_constructor
    (
        compiler: &mut Compiler,
        arity: usize
    ) -> Self
    {
        if compiler.symbols.calls.len() == 0 {
            panic!("no main function when constructor was declared");
        }
    }*/

    /*pub(crate) fn new_method
    (
        compiler: &mut Compiler,
        name: &str,
        arity: usize
    ) -> Self
    {
        if compiler.symbols.calls.len() == 0 {
            panic!("no main function when method was declared");
        }
    }*/

    fn function_setup(
        compiler: &mut Compiler,
        ast: &Ast,
        gc: &Gc,
        name: &str,
        arity: usize,
        scope: usize,
        ty: FunctionType,
    ) -> Self {
        let mut s = Self {
            name: name.to_string(),
            ty: ty,
            min_scope: scope,
            scope: scope,
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
            s.push_local(compiler, ast, gc, "this".to_string(), scope, false);
        } else {
            s.push_local(compiler, ast, gc, "".to_string(), scope, false);
        }
        compiler.symbols.calls.push(s.clone());
        s
    }

    fn resolve_local_or_upvalue(
        &mut self,
        compiler: Rc<Compiler>,
        token: Box<Token>,
        name: &String,
        arity: Option<usize>,
        do_print_on_err: bool,
    ) -> Result<VarInfo, VarError> {
        let result1 = match self.resolve_local(compiler, gc, token, &name, arity, do_print_on_err) {
            Err(e) => e,
            Ok(ok) => ok,
        };

        if let Err(_) = result1 {
            return result1;
        };

        let result2 = match self.resolve_upvalue(compiler, gc, token, name, arity, do_print_on_err)
        {
            Err(e) => e,
            Ok(ok) => ok,
        };

        if let Ok(local) = result1 {
            if let Ok(upvalue) = result2 {
                return result1;
            }
        }
        result2
    }

    fn resolve_local(
        &mut self,
        compiler: Rc<Compiler>,
        gc: &Gc,
        token: Box<Token>, // required for inner (*compiler).error()
        name: &String,
        arity: Option<usize>,
        do_print_on_err: bool,
    ) -> Result<VarInfo, VarError> {
        resolve_local_internal(compiler, name, arity, do_print_on_err)
    }

    pub(crate) fn resolve_local_internal(
        &mut self,
        compiler: Rc<Compiler>,
        gc: &Gc,
        token: Box<Token>, // required for (*compiler).error()
        name: &String,
        arity: Option<usize>,
        do_print_on_err: bool,
    ) -> Result<VarInfo, VarError> {
        for i in (0..self.locals.len()).rev() {
            if self.locals[i].name == *name {
                if !self.locals[i].defined && do_print_on_err {
                    unsafe {
                        (*compiler).error(
                            ast,
                            "self-initialization of local variable '".to_string()
                                + &mut self.locals[i].name,
                        )
                    };
                    return Err(VarError::new(
                        self.locals[i].name.clone(),
                        VarType::Var,
                        ScopeType::Local,
                        AccessType::None,
                        VarErrorType::SelfInit,
                        i,
                    ));
                } else {
                    return Ok(VarInfo::new(
                        self.locals[i].name.clone(),
                        arity,
                        VarType::Var,
                        ScopeType::Local,
                        AccessType::None,
                        self.locals[i].scope,
                        i,
                        0,
                    ));
                }
            }
        }

        Err(VarError::new(
            name.clone(),
            VarType::Var,
            ScopeType::Local,
            AccessType::None,
            VarErrorType::Undefined,
            0,
        ))
    }

    pub(crate) fn resolve_upvalue(
        &mut self,
        compiler:  Compiler,
        gc: &Gc,
        token: &Token,
        name: &String,
        arity: Option<usize>,
        do_print_on_error: bool,
    ) -> Result<VarInfo, VarError> {
        resolve_upvalue_internal(
            compiler,
            ast,
            gc,
            name.clone(),
            arity,
            do_print_on_error,
            compiler.symbols.calls.iter().rev(),
        )
    }

    pub(crate) fn resolve_upvalue_internal(
        &mut self,
        //parser: &mut Parser,
        compiler: *mut Compiler,
        gc: &Gc,
        token: &Token,
        name: &String,
        arity: Option<usize>,
        print_error: bool,
        mut iter: Rev<std::slice::IterMut<'_, FunCompiler>>,
    ) -> Result<VarInfo, VarError> {
        let mut fc = iter.next();
        if let Some(ref mut fun_compiler) = fc {
            let local = fun_compiler.resolve_local_internal(compiler, ast, gc, name, arity, true);
            match local {
                Ok(var_info) => {
                    if let Some(fun_compiler) = fc {
                        fun_compiler.locals[var_info.index].upvalue = true;
                        return fun_compiler.add_upvalue(
                            compiler,
                            ast,
                            gc,
                            &name,
                            arity,
                            var_info.upvalue_local_index,
                            true,
                        );
                    }
                }
                _ => {}
            }

            fc = iter.next();
            if fc.is_some() {
                if let Some(fun_compiler) = fc {
                    let upvalue = fun_compiler
                        .resolve_upvalue_internal(compiler, ast, gc, name, arity, false, iter);

                    match upvalue {
                        Ok(var_info) => {
                            if let Some(last) = compiler.symbols.calls.last_mut() {
                                return last.add_upvalue(
                                    compiler,
                                    ast,
                                    gc,
                                    name,
                                    arity,
                                    var_info.upvalue_local_index,
                                    false,
                                );
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }

        Err(VarError::new(
            name.clone(),
            VarType::Var,
            ScopeType::Upvalue,
            AccessType::None,
            VarErrorType::Undefined,
            0,
        ))
    }

    fn add_upvalue(
        &mut self,
        compiler: Rc<Compiler>,
        token: &Token,
        gc: &Gc,
        name: &String,
        arity: Option<usize>,
        index: usize,
        is_local: bool,
    ) -> Result<VarInfo, VarError> {
        for i in 0..self.locals.len() {
            if i == index && is_local {
                return Ok(VarInfo::new(
                    self.locals[i].name.clone(),
                    arity,
                    VarType::Var,
                    ScopeType::Upvalue,
                    AccessType::None,
                    self.locals[i].scope,
                    i,
                    0,
                ));
            }
        }

        if self.upvalues.len() >= MAX_UPVALUES {
            compiler.error(ast, "too many upvalues".to_string());
            return Err(VarError::new(
                name.clone(),
                VarType::Var,
                ScopeType::Upvalue,
                AccessType::None,
                VarErrorType::TooMany,
                MAX_UPVALUES,
            ));
        }

        let upvalue = Upvalue::new(is_local, index);
        self.upvalues.push(upvalue);

        Ok(VarInfo::new(
            self.locals[index].name.clone(),
            arity,
            VarType::Var,
            ScopeType::Upvalue,
            AccessType::None,
            self.locals[index].scope,
            self.upvalues.len() - 1,
            index,
        ))
    }

    fn push_local(
        &mut self,
        compiler: *mut Compiler,
        ast: &Ast,
        _gc: &Gc,
        name: String,
        scope: usize,
        upvalue: bool,
    ) -> Option<&mut Local> {
        if self.locals.len() == MAX_LOCALS {
            compiler.error(ast, "too many locals in function".to_string());
            return None;
        }
        self.locals
            .push(Local::new(name.to_string(), scope, upvalue));

        self.locals.last_mut()
    }

    fn declare_local(
        &mut self,
        compiler: &mut Compiler,
        ast: &Ast,
        gc: &Gc,
        name: &String,
        arity: Option<usize>,
        print_error: bool,
    ) -> Result<VarInfo, VarError> {
        let mut actual_name = "".to_string();
        if let Some(a) = arity {
            // format name, arity
            actual_name = format!("{}(#{})", name, a);
        } else {
            // just the name
            actual_name = name.to_string();
        }
        let mut iter = self.locals.iter_mut().rev();
        let mut scope = 0usize;
        loop {
            let i = iter.next();
            if i.is_none() {
                break;
            }
            if i.unwrap().scope < self.scope {
                scope = i.unwrap().scope;
                break;
            }
            if i.unwrap().name == *actual_name {
                if print_error {
                    compiler.error(
                        ast,
                        "local variable '".to_string() + &i.unwrap().name + "' already exists",
                    );
                }
                return Err(VarError::new(
                    actual_name.clone(),
                    VarType::Var,
                    ScopeType::Local,
                    AccessType::None,
                    VarErrorType::AlreadyExists,
                    i.unwrap().scope,
                ));
            }
        }
        self.push_local(compiler, ast, gc, actual_name.clone(), scope, false);

        Ok(VarInfo::new(
            actual_name.clone(),
            arity,
            VarType::Var,
            ScopeType::Local,
            AccessType::None,
            scope,
            self.locals.len() - 1,
            0,
        ))
    }

    fn declare_local_function(
        &mut self,
        compiler: &mut Compiler,
        ast: &Ast,
        gc: &Gc,
        name: &String,
        arity: usize,
        print_err: bool,
    ) -> Result<VarInfo, VarError> {
        let fun_name = format!("{}(#{})", name, arity as u32);
        if self.local_fun_symtab.get(&fun_name).is_some() {
            compiler.error(ast, format!("local function '{}' already exists", fun_name));
            Err(VarError::new(
                fun_name,
                VarType::Function,
                ScopeType::Local,
                AccessType::None,
                VarErrorType::AlreadyExists,
                self.get_scope_depth(),
            ))
        } else {
            let closure = self.closure.unwrap();
            let constant = self.get_constant(&Value::Closure(closure));
            let fun = VarInfo::new(
                fun_name.clone(),
                Some(arity),
                VarType::Var,
                ScopeType::Local,
                AccessType::None,
                self.get_scope_depth(),
                constant,
                0,
            );
            self.local_fun_symtab.insert(fun_name.clone(), fun.clone());

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
            return *pair.unwrap().1;
        }
        return len;
    }

    fn emit_op(&mut self, opcode: OpCode) {
        #[cfg(feature = "debug_opcode")]
        println!(
            "gen: {}: {}",
            self.closure.unwrap().get_core().get_name(),
            opcode
        );
        self.code.push(u8::from(opcode));
    }

    fn emit_byte_op(&mut self, opcode: OpCode, byte: u8) {
        #[cfg(feature = "debug_opcode")]
        println!(
            "gen: {}: {} byte {}",
            self.closure.unwrap().get_core().get_name(),
            opcode,
            byte
        );
        self.code.push(u8::from(opcode));
        self.code.push(byte);
    }

    fn emit_word_op(&mut self, opcode: OpCode, word: usize) {
        #[cfg(feature = "debug_opcode")]
        println!(
            "gen: {}: {} word {}",
            self.closure.unwrap().get_core().get_name(),
            opcode,
            word
        );
        self.code.push(u8::from(opcode));
        self.code.push(((word >> 8) & 0xff) as u8);
        self.code.push((word & 0xff) as u8);
    }

    fn emit_three_byte_op(&mut self, opcode: OpCode, three_byte: u32) {
        #[cfg(feature = "debug_opcode")]
        println!(
            "gen: {}: {} three byte {}",
            self.closure.unwrap().get_core().get_name(),
            opcode,
            three_byte
        );
        self.code.push(u8::from(opcode));
        self.code.push(((three_byte >> 16) & 0xff) as u8);
        self.code.push(((three_byte >> 8) & 0xff) as u8);
        self.code.push((three_byte & 0xff) as u8);
    }

    fn emit_math_op(&mut self, op: MathOp) {
        println!("gen: {} op {}", OpCode::MathOp, op);
        self.code.push(u8::from(OpCode::MathOp));
        self.code.push(u8::from(op));
    }
}

#[derive(Clone)]
struct LocalEntry {
    name: String,
    scope: usize,
    is_upvalue: bool,
    defined: bool,
}

impl LocalEntry {
    fn new(name: String, scope: usize, is_upvalue: bool) -> Self {
        Self {
            name,
            scope,
            is_upvalue,
            defined: false,
        }
    }
}

#[derive(Clone)]
pub(crate) struct UpvalueEntry {
    local: bool,
    index: usize,
}

impl UpvalueEntry {
    fn new(local: bool, index: usize) -> Self {
        Self { local, index }
    }
}





#[derive(Clone, PartialEq)]
pub enum VarType {
	None,
    Module,
    Var,
    Function,
    Class,
    Const,
}

#[derive(Clone, PartialEq)]
pub enum ScopeType {
    None,
    Module,
    Local,
    Upvalue,
    Class,
}

#[derive(Clone, PartialEq)]
pub enum AccessType {
    None,
	Public,
	Private,
}




#[derive(Clone)]
pub struct VarInfo {
    name: String,
    arity: Option<usize>,
    var_type: VarType,
    scope_type: ScopeType,
    access_type: AccessType,
    scope: usize,
    index: usize,
    upvalue_local_index: usize,
}

impl VarInfo {
    pub fn new
	(
        name: String,
        arity: Option<usize>,
        var_type: VarType,
        scope_type: ScopeType,
        access_type: AccessType,
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
            access_type,
            scope,
            index,
            upvalue_local_index,
        }
    }

    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }

    pub fn get_arity(&mut self) -> Option<usize> {
        self.arity
    }

    pub fn get_var_type(&mut self) -> VarType {
        self.var_type.clone()
    }

    pub fn get_scope_type(&mut self) -> ScopeType {
        self.scope_type.clone()
    }

    pub fn get_access_type(&self) -> AccessType {
        self.access_type.clone()
    }

    pub fn get_scope_level(&mut self) -> usize {
        self.scope
    }

    pub fn get_index(&mut self) -> usize {
        self.index
    }
}

#[derive(Clone, PartialEq)]
pub enum VarErrorType {
    None,
    Undefined,
    InvalidType,
    AlreadyExists,
    TooMany,
    SelfInit,
    IsFunction,
    MalformedName,
    Other,
}

#[derive(Clone)]
pub struct VarError {
    name: String,
    var_type: VarType,
    scope_type: ScopeType,
    access_type: AccessType,
    error_type: VarErrorType,
    scope_level: usize,
}

impl VarError {
    pub fn new(
        name: String,
        var_type: VarType,
        scope_type: ScopeType,
        access_type: AccessType,
        error_type: VarErrorType,
        scope_level: usize
    ) -> Self
    {
        Self {
            name,
            var_type,
            scope_type,
            access_type,
            error_type,
            scope_level,
        }
    }

    pub fn get_name(&mut self) -> String{
        self.name.clone()
    }

    pub fn get_var_type(&self) -> VarType {
        self.var_type.clone()
    }

    pub fn get_scope_type(&self) -> ScopeType {
        self.scope_type.clone()
    }

    pub fn get_error_type(&self) -> VarErrorType {
        self.error_type.clone()
    }

    pub fn get_scope_level(&self) -> usize {
        self.scope_level
    }
}

impl Default for VarError {
	fn default() -> Self {
		Self {
			name: "".to_string(),
			var_type: VarType::None,
			scope_type: ScopeType::None,
			access_type: AccessType::None,
			error_type: VarErrorType::None,
			scope_level: 0,
		}
	}
}




struct Compiler {
    symbols: SymbolTables,
    compile_flag: bool,
    error_count: u8,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,
}

impl Compiler {
    fn new(compile: bool, debug: bool) -> Self {
        Self {
            symbols: SymbolTables::new(),
            compile_flag: compile,
            error_count: 0u8,
            in_panic_mode: false,
            in_panic_lock_mode: false,
        }
    }

    fn resolve_var(
        &mut self,
        compiler: &mut Compiler,
        ast: &Ast,
        arity: Option<usize>,
        do_print_on_error: bool,
    ) -> Result<VarInfo, VarError> {
        let mut nm: String = "".to_string();
        match *ast {
            Ast::ClosureDecl { name, .. } => {
                nm = name.clone();
            }
            Ast::VariableId { name, .. } => {
                nm = name.clone();
            }
            Ast::Call { recv, .. } => {
                if let Some(rcv) = recv {
                    match *rcv {
                        Ast::ClosureDecl { name, .. } => {
                            nm = name.clone();
                        }
                        Ast::VariableId { name, .. } => {
                            nm = name.clone();
                        }
                        _ => todo!(),
                    }
                }
            },
            _ => todo!(),
        }
        match self.get_scope_depth() {
            0usize => self.resolve_mod_var(ast, arity, do_print_on_error),
            _ => {
                let mut last_mut = self.symbols.calls.last_mut().unwrap();
                last_mut.resolve_other_var(compiler, ast, &nm, arity, do_print_on_error)
            }
        }
    }

    // resolve fully qualified module, variable, or function name.
    fn resolve_mod_var(
        &mut self,
        ast: &Ast,
        arity: Option<usize>,
        do_print_on_error: bool,
    ) -> Result<VarInfo, VarError> {
        let result: Result<VarInfo, VarError> = Err(VarError::default());
        let mut name = "".to_string();
        let mut rest = "".to_string();

        self.build_name_result_from_ast(ast, None, arity, &mut name, &mut rest)
    }

    fn build_name_result_from_ast(
        &mut self,
        left: Option<Box<Ast>>,
        right: Option<Box<Ast>>,
        arity: Option<usize>,
        mut nm: &mut String,
        mut rest: &mut String,
    ) -> Result<VarInfo, VarError> {
        let first_l = true;
        if let Some(node_l) = left {
            match *node_l {
                Ast::DblColon { l, r, level } => {
                    if l.is_none() {
                        self.error(l, "nothing on left side of '::'".to_string());
                    }

                    if r.is_none() {
                        self.error(r, "nothing on right side of '::'".to_string());
                    }

                    self.build_name_result_from_ast(l, r, arity, nm, rest)
                }
                Ast::ModuleId { name, level } => {
                    return Ok(VarInfo::new(
                        *nm + name.as_str().clone(),
                        arity,
                        VarType::Module,
                        ScopeType::None,
                        AccessType::None,
                        0usize,
                        0usize,
                        0usize,
                    ));
                }
                Ast::VariableId {
                    name, arity, level, ..
                } => {
                    let rest = name.clone();
                    let mut access = AccessType::Public;
                    let mut ch = rest.chars();
                    ch.next();
                    if name.chars().nth(0) == Some('_') {
                        access = AccessType::Private;
                    }
                    return Ok(VarInfo::new(
                        *nm + &rest,
                        arity,
                        VarType::Var,
                        ScopeType::Module,
                        access,
                        0usize,
                        0usize,
                        0usize,
                    ));
                }
                _ => todo!(),
            }
        } else {
            if right.is_none() {
                self.error(right, "'nothing on right side of ::");
            }
            if right.is_some() {
                self.build_name_result_from_ast(left, right, arity, nm, rest)
            } else {
                Err(rror::new(
                    "::".to_string(),
                    VarType::None,
                    ScopeType::Module,
                    AccessType::None,
                    VarErrType::MalformeName,
                    0,
                ));
            }
        }
    }

    /*fn parse_var(
        &self,
        ast: &Ast,
        arity: Option<usize>,
        print_err: bool,
    ) -> Result<VarInfo, VarError> {
        let module_sections: Vec<&str> = name.split("::").collect();

        if module_sections[0] == "" && module_sections.len() == 1 {
            return Err(VarError::new(
                "::".to_string(),
                VarType::Module,
                ScopeType::None,
                AccessType::None,
                VarErrorType::MalformedName,
                0,
            ));
        }

        let mut module_name = "".to_string();

        for i in 0..module_sections.len() {
            if module_sections.len() > 1 {
                module_name.push_str("::");
            } else if i == 0 {
                module_name = name;
                break;
            }
            module_name.push_str(module_sections[i]);
            if let Some(nm) = module_sections.get(i) {
                if module_sections.len() > 1 {
                    if *nm == "" {
                        if i > 0 {
                            return Err(VarError::new(
                                name.clone(),
                                VarType::None,
                                ScopeType::None,
                                AccessType::None,
                                VarErrorType::MalformedName,
                                0,
                            ));
                        } else {
                            continue;
                        }
                    }
                    let mut mod_result = match self.resolve_module_chunk(
                        ast,
                        module_sections,
                        i,
                        arity,
                        print_err,
                    ) {
                        Ok(var) => {
                            if var.var_type == VarType::Module {
                                continue;
                            } else if i == 0
                                && var.var_type != VarType::Module
                                && var.var_type != VarType::None
                            {
                                if var.scope_type == ScopeType::Module
                                    || var.scope_type == ScopeType::Local
                                    || var.scope_type == ScopeType::Upvalue
                                {
                                    return Ok(var);
                                } else {
                                    return Err(VarError::new(
                                        var.name.clone(),
                                        var.var_type.clone(),
                                        var.scope_type.clone(),
                                        var.access_type.clone(),
                                        VarErrorType::InvalidType,
                                        0,
                                    ));
                                }
                            } else {
                                return Err(VarError::new(
                                    var.name.clone(),
                                    var.var_type.clone(),
                                    var.scope_type.clone(),
                                    var.access_type.clone(),
                                    VarErrorType::InvalidType,
                                    0,
                                ));
                            }
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    };
                }
            }
        }
        self.resolve_module_chunk(ast, module_sections, 0)
    }*/

    /*fn resolve_module_chunk(
        &mut self,
        ast: &Ast,
        module_sections: Vec<&str>,
        index: usize,
        arity: Option<usize>,
        do_print_on_err: bool,
    ) -> Result<VarInfo, VarError> {
        let ch = module_sections[index].chars().next();

        let mut curr_module_section: String = module_sections[index].to_string();
        if module_sections.len() == 1 && index == 0 {
            match self.symbols.calls.last_mut().unwrap().resolve_local(
                self,
                curr_module_section,
                do_print_on_err,
            ) {
                Ok(var) => Ok(var),
                Err(err) => {
                    match self.symbols.calls.last_mut().unwrap().resolve_upvalue(
                        self,
                        curr_module_section,
                        do_print_on_err,
                    ) {
                        Ok(var) => Ok(var),
                        Err(err) => self.symbols.curr_mod.get_mod_rel(curr_module_section),
                    }
                }
            }
        } else if index == module_sections.len() - 1 {
        }
    }*/

    /*fn resolve_var_from_info(
        &mut self,
        ast: Option<Box<Ast>>,
        name: String,
        mut count: &mut usize,
        arity: Option<usize>,
    ) -> Result<VarInfo, VarError> {
        if *count == 1 {
            if ast.is_none() {
                return self.resolve_var(name);
            } else {
            }
        }
        name.extend("::");
        if let Some(node) = ast {
            match *node {
                Ast::VariableId { name, next } => {
                    name.extend(name);
                    if let Some(nxt) = next {
                        if *count < 5 {
                            return self.resolve_var_from_info(
                                nxt, name, *count + 1, arity);
                        } else {
                            return Err(VarError::new(
                                name,
                                VarType::None,
                                ScopeType::None,
                                AccessType::None,
                                VarErrorType::PathTooLong,
                            ));
                        }
                    } else {
                        return self.resolve_var(name);
                    }
                }
                _ => {}
            }
        } else {
            if *count == 0 {
                panic!("ast node is none");
            }
        }
    }*/

    fn declare_var(
        &mut self,
        name: String,
        arity: usize,
        var_type: VarType,
        scope_type: ScopeType,
        access_type: AccessType,
        scope: usize,
    ) -> Result<VarInfo, VarError> {
        let ret = match resolve_var(name) {
            Ok(var_info) => Err(VarError::new(
                name,
                arity,
                var_type,
                scope_type,
                access_type,
                VarErrorType::AlreadyExists,
            )),
            Err(err_info) => {
                if err_info.err_type == VarErrorType::Undefined {
                    Ok(VarInfo::new(
                        name,
                        arity,
                        err_info.var_type,
                        err_info.scope_type,
                        err_info.access_type,
                        self.symbols.curr_mod.mod_vars.len(),
                        scope,
                    ))
                } else {
                    err_info
                }
            }
        };

        ret = match ret {
            Ok(var_info) => {
                self.add_var(name, arity, var_info.var_type, var_info.scope_type, scope)
            }
            Err(err_info) => err_info,
        };
        ret
    }

    fn add_local(
        &mut self,
        name: String,
        arity: usize,
        var_type: VarType,
        scope_type: ScopeType,
        scope: usize,
    ) -> Result<VarInfo, VarError> {
    }

    fn declare_mod(
        &mut self,
        name: String,
        parent: Option<&Module>,
    ) -> Result<(), String> {
        if self.symbols.mods.is_empty() {
            if name == ".".to_string() {
                self.symbols.mods.insert(
                    ".".to_string(),
                    Module::new("".to_string(), self.symbols.mods.len(), None),
                );

                if let Some(res) = self.symbols.mods.get_mut(&name) {
                    self.set_curr_mod(res.name());
                }
                return Ok(());
            }
        }
        match parent {
            Some(res) => {
                if res.name() == ".".to_string() {
                    return Err("global module cannot be a parent of any other module".to_string());
                }
                panic!("There should be a module here!");
            }
            _ => {
                self.symbols
                    .mods
                    .insert(name, Module::new(name, self.symbols.mods.len(), parent));
                if let Some(r) = self.symbols.mods.get_mut(&name) {
                    return self.set_curr_mod(name);
                } else {
                    panic!("Could not set current module");
                }
            }
        }
    }

    fn set_curr_mod(&mut self, name: String) -> Result<(), String> {
        if let Some(mut res) = self.symbols.mods.get_mut(&name) {
            self.symbols.curr_mod = Some(res);
            Ok(())
        } else {
            Err("cannot set current module to one which is invalid".to_string())
        }
    }

    fn get_curr_mod(&self) -> Option<&Module> {
        self.symbols.curr_mod
    }

    fn get_mod(&mut self, name: String) {}

    fn declare_curr_mod_var(
        &mut self,
        parser: &mut Parser,
        name: String,
        arity: Option<usize>,
    ) -> Result<VarInfo, VarError> {
        let curr_mod = self.get_curr_mod();
        let Some(res) = curr_mod else {
            return Err(VarInfo::new(
                name.clone(),
                arity,
                VarType::Var,
                ScopeType::Module,
                AccessType::None,
                VarErrorType::None,
                0,
            ));
        };
        let res2 = res.mod_vars.get_mut(&name);
        if res2.is_some() {
            Err(VarError::new(
                name.clone(),
                VarType::Var,
                ScopeType::Module,
                AccessType::None,
                VarErrorType::AlreadyExists,
            ))
        } else {
            let mut access_type = AccessType::Public;
            if name.chars().nth(0) == Some('_') {
                access_type = AccessType::Private;
            }
            let var = VarInfo::new(
                name.clone(),
                VarType::Var,
                ScopeType::Module,
                access_type.clone(),
                self.get_curr_mod().mod_vars.len(),
            );
        }
    }

    fn begin_scope(&mut self) {
        if let Some(last_mut) = self.symbols.calls.last_mut() {
            last_mut.begin_scope();
        } else {
            panic!("could not get topmost function information");
        }
    }

    fn end_scope(&mut self) {
        if let Some(last_mut) = self.symbols.calls.last_mut() {
            last_mut.begin_scope();
        } else {
            panic!("could not get topmost function information");
        }
    }

    fn get_scope_depth(&mut self) -> usize {
        if let Some(last_mut) = self.symbols.calls.last_mut() {
            last_mut.get_scope_depth()
        } else {
            panic!("could not get scope depth");
        }
    }

    fn compile(
        &mut self,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        //let vm_ptr : *mut VM = ptr::from_mut(vm);
        if !self.compile_flag {
            self.code_gen(ast, gc)
        } else {
            self.code_gen_c(ast, vm, gc);
            None
        }
    }

    fn error(&mut self, ast: Option<Box<Ast>>, err_message: String) {
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
            if self.in_panic_mode {
                eprintln!("error: too many errors");
                self.in_panic_lock_mode = true;
            }
        }
        self.in_panic_mode = true;
    }

    fn begin_function(&mut self, _ast: &mut Ast, _vm: *mut VM, _gc: &Gc) {}

    #[allow(unused_variables)]
    fn code_gen_param(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> bool {
        match *ast {
            Ast::VariableId { ref name, .. } => {
                if let Some(ref mut last) = self.symbols.calls.last_mut() {
                    let var = unsafe { add_local_var(self, ast, gc, name, None, true) };
                    match var {
                        Ok(var_info) => {
                            unsafe { define_var(vm, last, 0) };
                            return true;
                        }
                        Err(e) => {
                            return false;
                        }
                    }
                }
            }
            _ => {
                self.error(ast, "function parameter should be a variable".to_string());
                return false;
            }
        }
        false
    }

    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen(
        &mut self,
        compiler: Rc<Compiler>,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        let mut result: Option<Managed<Closure>> = None;

        if let Some(ref node) = ast {
            match **node {
                Ast::ClosureDecl {
                    ref name,
                    ref fun_ty,
                    ref params,
                    ref listing,
                    ref closure,
                } => self.code_gen_closure_decl(
                    ast,
                    name.to_string(),
                    fun_ty.clone(),
                    params,
                    listing.to_vec(),
                    closure,
                ),
                Ast::LetDecl { ref name, ref expr } => {
                    self.code_gen_let_decl(name.to_string(), expr.clone())
                }
                Ast::ReturnStmt { ref expr, .. } => {
                    let last = unsafe { self.symbols.calls.last_mut().unwrap() };

                    last.closure
                }
                Ast::Call(ref recv, ref args) => {
                    self.code_gen_call(&*ast.clone().unwrap(), gc, &recv, args)
                }
                Ast::Block { ref listing, .. } => {
                    for i in listing {
                        self.code_gen(&mut Some(i.clone()), vm, gc);
                    }
                    self.symbols.calls.last_mut().unwrap().closure
                }
                Ast::ExprStmt { ref expr } => self.code_gen(&mut Some(expr.clone()), vm, gc),
                Ast::Value { ref val, .. } => self.code_gen_value(val.clone()),
                Ast::VariableId {
                    ref name,
                    ref arity,
                } => self.code_gen_var(self.gc, name.to_string(), arity.clone()),
                Ast::Add(..)
                | Ast::Subtract(..)
                | Ast::Multiply(..)
                | Ast::Divide(..)
                | Ast::Modulo(..)
                | Ast::Power(..) => {
                    unsafe { (*self).emit_math_op(&mut Some(node.clone()), vm, gc) };
                    unsafe { (*self).symbols.calls.last_mut().unwrap().closure.clone() };
                }
                Ast::Assign(ref left, ref right) => {
                    unsafe { (*self).code_gen(right, vm, gc) };
                    if let Some(l) = left {
                        match **l {
                            Ast::VariableId {
                                ref name,
                                ref arity,
                            } => {
                                let var = unsafe {
                                    (*self).resolve_var(self, vm, &**l, gc, name, *arity, true);
                                };
                                match var {
                                    Ok(v) => {
                                        if let Some(last) = self.symbols.calls.last_mut() {
                                            last.emit_word_op(OpCode::StoreVar, v.index as usize);
                                            last.closure
                                        } else {
                                            panic!("function compiler stack is empty");
                                        }
                                    }
                                    Err(e) => {
                                        if e.error_type == VarErrorType::Undefined {
                                            self.error(
                                                Some(l),
                                                format!("variable {} is undefined", e.name),
                                            )
                                        }
                                    }
                                }
                                None
                            }
                            _ => {
                                self.error(
                                    Some(l),
                                    "only variables and fields can be on left side of assignment"
                                        .to_string(),
                                );
                                None
                            }
                        }
                    } else {
                        panic!("left node of assignment doesn't exist");
                    }
                }
            }
        } else {
            eprintln!("ast node doesn't exist");
            None
        }
    }

    fn code_gen_closure_decl(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        let mut result: Option<Managed<Closure>> = None;
        if let Some(node) = ast {
            match *ast {
                Ast::ClosureDecl {
                    name,
                    fun_ty,
                    params,
                    listing,
                    closure,
                } => {
                    let ty = node.fun_ty.clone();
                    let mut fun = match ty {
                        FunctionType::TopLevel => {
                            FunCompiler::new_main(self);
                            self.symbols.calls.last_mut().unwrap().closure = Some(closure.clone());
                        }
                        FunctionType::NamedClosure => {
                            FunCompiler::new_named_closure(self);
                            self.symbols.calls.last_mut().unwrap().closure = Some(closure.clone());
                        }
                        FunctionType::AnonClosure => {
                            FunCompiler::new_anon_closure(self);
                            self.symbols.calls.last_mut().unwrap().closure = Some(closure.clone());
                        }
                        _ => {
                            panic!("invalid function type for Ast::ClosureDecl node");
                        }
                    };

                    result = self.symbols.calls.last_mut().unwrap().closure.clone();

                    if ty != FunctionType::TopLevel {
                        self.begin_scope();
                    }

                    for i in params {
                        let ret = self.code_gen_param(&i.clone(), gc);
                        if !ret {
                            result = None;
                        }
                    }

                    for i in listing {
                        println!("here");
                        let ret = self.code_gen(Some(i.clone()), vm, gc);
                        println!("ret = {:#?}", ret);
                        if ret.is_none() {
                            result = None;
                        }
                    }

                    if ty != FunctionType::TopLevel {
                        unsafe { self.end_scope() };
                    }

                    if self.error_count == 0 {
                        if ty == FunctionType::NamedClosure || ty == FunctionType::AnonClosure {
                            let fun_com = &mut self.symbols.calls.last_mut().unwrap();

                            let prev_fun_com =
                                &mut (&mut self.symbols.calls)[self.symbols.calls.len() - 2];
                            if let Some(closure) = fun_com.closure {
                                let constant = prev_fun_com.get_constant(&Value::Closure(closure));
                                prev_fun_com.emit_word_op(OpCode::LoadValue, constant as usize);
                            } else {
                                panic!("internal closure doesn't exist");
                            }

                            if ty == FunctionType::NamedClosure {
                                let last = self.symbols.calls.last_mut().unwrap();

                                let scope = self.get_scope_depth();
                                let mut scope_type = ScopeType::Module;
                                if scope > 0 {
                                    scope_type = ScopeType::Local;
                                }
                                let mut var: Result<VarInfo, VarError> = Err(VarError::new(
                                    name.clone(),
                                    VarType::Var,
                                    scope_type,
                                    AccessType::None,
                                    VarErrorType::Undefined,
                                    scope,
                                ));
                                let fun = self.add_var(node.name, Some(node.params.len()), true);

                                match fun {
                                    Ok(var_info) => {
                                        println!("here!");
                                        self.define_var(vm, prev_fun_com, var_info.index)
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if result.is_some() {
                            result = self.code_gen_return(ast, vm, gc);
                        }
                        self.symbols.calls.pop();
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
        } else {
            None
        }
    }

    fn code_gen_let_decl(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        let last = self.symbols.calls.last_mut().unwrap();
        let scope = self.get_scope_depth();

        let mut var: Result<VarInfo, VarError> = Err(VarError::new(
            "".to_string(),
            VarType::None,
            ScopeType::None,
            AccessType::None,
            VarErrorType::Undefined,
            0,
        ));

        let mut arity: Option<usize> = None;
        if let Some(expr) = ast {
            let function = match *expr {
                Ast::ClosureDecl {
                    ref name,
                    ref fun_ty,
                    ref params,
                    ref listing,
                    ref closure,
                } => {
                    arity = Some(params.len());
                    var = self.add_var(name, arity);
                    true
                }
                _ => false,
            };

            if let Some(node) = ast {}

            if self.code_gen(Some(expr.clone()), vm, gc).is_none() {
                return None;
            }
        }
        match var {
            Ok(var_info) => {
                self.define_var(vm, last, var_info.index);
                return last.closure;
            }
            Err(var_error) => None,
        }
    }

    fn code_gen_value(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        if let Some(last) = unsafe { self.symbols.calls.last_mut() } {
            if let Some(node) = ast {
                match *node {
                    Ast::Value { val } => match val {
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
                                last.emit_op(OpCode::LoadReg(
                                    self.curr_reg(),
                                    self.next_avail_reg(),
                                ));
                            }
                            self.symbols.calls.last_mut().unwrap().closure.clone()
                        }
                        Value::Float(_) => {
                            let constant = last.get_constant(&val);
                            last.emit_word_op(OpCode::LoadValue, constant as usize);
                            self.symbols.calls.last_mut().unwrap().closure.clone()
                        }
                        Value::Bool(b) => {
                            if b == true {
                                last.emit_op(OpCode::LoadTrue);
                            } else {
                                last.emit_op(OpCode::LoadFalse);
                            }
                            self.symbols.calls.last_mut().unwrap().closure.clone()
                        }
                        Value::Nil => {
                            last.emit_op(OpCode::LoadNil);
                            self.symbols.calls.last_mut().unwrap().closure.clone()
                        }
                        Value::String(_) => {
                            let constant = last.get_constant(&val);
                            last.emit_word_op(OpCode::LoadValue, constant as usize);
                            self.symbols.calls.last_mut().unwrap().closure.clone()
                        }
                        _ => {
                            self.error(&node, "value is not a valid constant".to_string());
                            None
                        }
                    },
                }
            } else {
                panic!("not an AST node");
            }
        } else {
            eprintln!("function compiler stack is empty");
            None
        }
    }

    fn code_gen_var(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        if let Some(node) = ast {
            let var =
                unsafe { (*compiler).resolve_var_from_info(ast, "".to_string(), &mut 0, None) };
            match var {
                Err(e) => {
                    if e.error_type == VarErrorType::Undefined {
                        unsafe {
                            (*compiler).error(ast, format!("variable {} is undefined", e.name))
                        };
                    }
                    return None;
                }
                Ok(v) => {
                    if let Some(last) = unsafe { (*compiler).symbols.calls.last_mut() } {
                        unsafe { (*compiler).emit_var_load(last, v) };
                        return unsafe { (*compiler).symbols.calls.last_mut().unwrap().closure };
                    };
                }
            };
        }
        None
    }

    fn code_gen_call(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        recv: Option<Box<Ast>>,
        args: &Vec<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        if let Some(last) = unsafe { (*compiler).symbols.calls.last_mut() } {
            match *recv.clone().unwrap() {
                Ast::VariableId {
                    ref name,
                    ref arity,
                } => {
                    let mut var: Result<VarInfo, VarError> = Err(VarError::new(
                        "".to_string(),
                        VarType::None,
                        ScopeType::None,
                        AccessType::None,
                        VarErrorType::None,
                        0,
                    ));

                    if let Some(last_mut) = unsafe { (*self).symbols.calls.last_mut() } {
                        var = unsafe { (*self).resolve_var(name.to_string()) };
                    } else {
                        panic!("could not get variable/error info");
                    }

                    match var {
                        Ok(v) => {
                            unsafe { (*self).emit_var_load(last, v) };
                        }
                        Err(e) => {
                            if e.error_type == VarErrorType::Undefined {
                                unsafe {
                                    (*compiler).error(
                                        &*recv.clone().unwrap(),
                                        format!("variable {} is undefined", e.name),
                                    )
                                };
                            }
                            return None;
                        }
                    }
                }
                _ => {}
            }
            for i in &*args {
                if unsafe { (*compiler).code_gen(Some(i.clone()), vm, gc).is_none() } {
                    return None;
                }
            }
            #[cfg(not(feature = "reg_mach"))]
            last.emit_byte_op(OpCode::Call, args.len() as u8);

            #[cfg(feature = "reg_mach")]
            last.emit_op(OpCode::Call, args.len() as u8);

            unsafe {
                (*compiler)
                    .symbols
                    .calls
                    .last_mut()
                    .unwrap()
                    .closure
                    .clone()
            }
        } else {
            panic!("no function compilers left on stack");
        }
    }

    #[allow(unused_assignments)]
    #[allow(unused_variables)]
    fn code_gen_return(
        &mut self,
        compiler: *mut Compiler,
        ast: Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        if let Some(node) = ast {
            match *node {
                Ast::ClosureDecl {
                    ref name,
                    ref fun_ty,
                    ref params,
                    ref listing,
                    ref closure,
                } => {
                    if let last = unsafe { (*self).symbols.calls.last_mut().unwrap() } {
                        if unsafe { (*compiler).error_count == 0 } {
                            if listing.is_empty() {
                                last.emit_op(OpCode::LoadNil);
                                last.emit_op(OpCode::Return);
                            }
                            if let Some(mut l) = listing.last() {
                                match **l {
                                    Ast::ExprStmt { .. } => {
                                        if let Some(last_mut) =
                                            unsafe { (*compiler).symbols.calls.last_mut() }
                                        {
                                            last_mut.emit_op(OpCode::Return);
                                        } else {
                                            panic!("could not emit return opcode");
                                        }
                                    }
                                    Ast::ReturnStmt { ref expr, .. } => {
                                        if let Some(e) = expr {
                                            unsafe {
                                                (*compiler).code_gen(
                                                    Some::<Box<Ast>>(e.clone()),
                                                    vm,
                                                    gc,
                                                )
                                            };
                                        } else {
                                            last.emit_op(OpCode::LoadNil);
                                        }
                                        last.emit_op(OpCode::Return);
                                    }
                                    _ => {}
                                }
                            }
                            let c =
                                unsafe { (*compiler).symbols.calls.last_mut().unwrap().closure };
                            if c.is_some() {
                                c.unwrap().get_core().finish(
                                    unsafe {
                                        (*compiler)
                                            .symbols
                                            .calls
                                            .last_mut()
                                            .unwrap()
                                            .constant_arr
                                            .clone()
                                    },
                                    unsafe {
                                        (*compiler).symbols.calls.last_mut().unwrap().code.clone()
                                    },
                                );
                                return c;
                            }
                            return None;
                        } else {
                            return None;
                        }
                    }
                }
                _ => {
                    unreachable!();
                }
            }
        }
        None
    }

    fn add_mod(&mut self, compiler: *mut Compiler, fun_com: &mut FunCompiler) {
        fun_com.emit_op(OpCode::AddMod);
    }

    fn switch_mod(
        &mut self,
        compiler: *mut Compiler,
        fun_com: &mut FunCompiler,
        module: usize,
    ) {
        fun_com.emit_word_op(OpCode::SwitchMod, module);
    }

    fn emit_var_load(
        &mut self,
        compiler: *mut Compiler,
        fun_com: &mut FunCompiler,
        var: VarInfo,
    ) {
        match var.scope_type {
            ScopeType::Module => {
                fun_com.emit_word_op(OpCode::LoadVar, var.index as usize);
            }
            ScopeType::Local => {
                if var.index < 9 {
                    fun_com.emit_op(OpCode::from(u8::from(OpCode::LoadLocal0) + var.index as u8));
                } else {
                    fun_com.emit_byte_op(OpCode::LoadLocal, var.index as u8);
                }
            }
            ScopeType::Upvalue => {
                fun_com.emit_byte_op(OpCode::LoadUpvalue, var.index as u8);
            }
            _ => todo!(),
        }
    }

    fn emit_math_op(
        &mut self,
        compiler: *mut Compiler,
        ast: &mut Option<Box<Ast>>,
        vm: &mut VM,
        gc: &Gc,
    ) -> Option<Managed<Closure>> {
        if let Some(node) = ast {
            if let Some(last_mut) = unsafe { (*compiler).symbols.calls.last_mut() } {
                match **node {
                    Ast::Add(ref left, ref right) => {
                        self.code_gen(left.clone(), vm, gc);
                        self.code_gen(right.clone(), vm, gc);
                        last_mut.emit_math_op(MathOp::Add);
                        last_mut.closure
                    }
                    Ast::Subtract(ref left, ref right) => {
                        self.code_gen(left.clone(), vm, gc);
                        self.code_gen(right.clone(), vm, gc);
                        last_mut.emit_math_op(MathOp::Subtract);
                        last_mut.closure
                    }
                    Ast::Multiply(ref left, ref right) => {
                        self.code_gen(left.clone(), vm, gc);
                        self.code_gen(right.clone(), vm, gc);
                        last_mut.emit_math_op(MathOp::Multiply);
                        last_mut.closure
                    }
                    Ast::Divide(ref left, ref right) => {
                        self.code_gen(left.clone(), vm, gc);
                        self.code_gen(right.clone(), vm, gc);
                        last_mut.emit_math_op(MathOp::Divide);
                        last_mut.closure
                    }
                    Ast::Modulo(ref left, ref right) => {
                        self.code_gen(left.clone(), vm, gc);
                        self.code_gen(right.clone(), vm, gc);
                        last_mut.emit_math_op(MathOp::Modulo);
                        last_mut.closure
                    }
                    Ast::Power(ref left, ref right, ..) => {
                        self.code_gen(left.clone(), vm, gc);
                        self.code_gen(right.clone(), vm, gc);
                        last_mut.emit_math_op(MathOp::Power);
                        last_mut.closure
                    }
                    _ => {
                        unreachable!();
                    }
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
        Ast::VariableId{ref name, ..} => {
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
        Ast::ClosureDecl {
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
                prev_fun_com.emit_word_op(OpCode::LoadValue, constant as usize);
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
        Ast::VariableId{
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
        Ast::ClosureDecl {
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

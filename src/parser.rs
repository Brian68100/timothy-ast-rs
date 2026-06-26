use crate::lexer::*;
use crate::token::*;
use crate::function_core::*;
use crate::closure::*;
use crate::ast::*;
use crate::value::*;
use crate::vm::*;
use crate::gc::{Gc, NO_GC};
use crate::natives::*;

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

pub fn interpret_string
(
    source: &mut String,
    natives: Option<&NativeMap>,
    vm: &mut VM,
    gc: &mut Gc,
    print_ast: bool,
    compile: bool,
    debug: bool,
) -> Result<Value, InterpretErrorType>
{
    let mut parser = Parser::new_from_string(source, gc);
    let mut root: Option<Box<Ast>> = Some(Box::new(Ast::Program {
        listing: vec![],
    }));
 
    if !parser.parse(&mut root, vm, gc) {
        Err(InterpretErrorType::ParseError)
    } else {
        println!("root = {:#?}", root.clone());
        if print_ast {
            let ast_printer = AstPrinter::new();
            ast_printer.run(&mut root);
        }
        
        vm.run(&mut root, gc, debug, compile)
    }
}

pub fn interpret_file
(
    file_name: String,
    natives: Option<&NativeMap>,
    vm: &mut VM,
    gc: &mut Gc,
    print_ast: bool,
    compile: bool,
    debug: bool,
) -> Result<Value, InterpretErrorType>
{
    let vm_ptr: *mut VM = vm;
    let parser = Parser::new_from_file(file_name, gc);
    if parser.is_none() {
        return Err(InterpretErrorType::OtherError);
    }
    let mut p = parser.unwrap();
    let mut root: Option<Box<Ast>> = Some(Box::new(Ast::Program {
        listing: vec![],
    }));
     
    if !p.parse(&mut root, vm_ptr, gc) {
        Err(InterpretErrorType::ParseError)
    } else {
        if print_ast {
            let ast_printer = AstPrinter::new(); 
            ast_printer.run(&mut root);
        }

        vm.run(&mut root, gc, debug, compile) 
    }
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
            1  => BindingPower::Assignment,     // = += -= *= /* &= |= ^= <<= >>=

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
            BindingPower::Assignment => 1,     // = += -= *= /* &= |= ^= <<= >>=

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
    lexer: Lexer,
    curr: Token,
    prev: Token,
}

struct Parser {
    lexer_stack: Vec<LexerData>,
    error_count: usize,
    //compiler: Compiler,
    in_panic_mode: bool,
    in_panic_lock_mode: bool,
}

impl<'a> Parser { 
   fn new(gc: &Gc, debug: bool) -> Self {
        let mut s = Self {
            gc,
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

    pub fn parse_text(&self, text: String) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Lexer::new_from_text(text);

        if lexer.is_err() {
            return Err(InterpretErrorType::ParseError)
        }

        self.lexer_stack.clear();
        self.lexer_stack.push(LexerData {
            lexer: lexer.unwrap(),
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
        self.advance(self.gc);
        if !self.parse(self.gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            vm.run(&mut root, gc, debug, compile)
        }
    }
    pub fn parse_file(&self, fname: String) -> Result<Value, InterpretErrorType> {
        self.lexer_stack = vec![];
        self.error_count = 0;
        self.in_panic_mode = false;
        self.in_panic_lock_mode = false;

        let lexer = Some(Lexer::new_from_file(fname));

        if lexer.is_err() {
            return Err(InterpretErrorType::ParseError)
        }

        self.lexer_stack.push(LexerData {
            lexer: lexer.unwrap(),
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
        self.advance(self.gc);
        if !self.parse(self.gc) {
            Err(InterpretErrorType::ParseError)
        } else {
            vm.run(gc, debug, compile)
        }
    }

    fn advance(&mut self, gc: &Gc) {
        self.lexer_stack.last_mut().unwrap().prev = self.lexer_stack.last_mut().unwrap().curr.clone();
        loop {
            self.lexer_stack.last_mut().unwrap().curr = self.lexer_stack.last_mut()
                .unwrap().lexer.scan(gc);
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
        
        eprint!("{}:{}:{}: error", self.lexer_stack.last_mut().unwrap().lexer.file_name(), token.line, token.col);

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
	    root: &mut Option<Box<Ast>>,
        gc: &Gc,
    ) -> bool {
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
                        Ast::Program {
                            ref mut listing,
                            ..} => {

                            listing.push(left.unwrap());
                        },
                        /*Ast::FunDecl {
                            ref mut listing, 
                            ..} => {
                            
                            listing.push(left.unwrap());
                        },*/
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
        _gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        None
    }

    fn parse_class_decl(&mut self,
        _left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
        _gc: &Gc
    ) -> Option<Box<Ast>> 
    {
        None
    }

    fn parse_macro_def(&mut self,
        _left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
    ) -> Option<Box<Ast>> 
    {
        None
    }

    fn parse_ns_decl
    (
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
        _gc: &Gc
    ) -> Option<Box<Ast>> {
        None
    }

    fn parse_import_cmd(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
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
            Some(closure),
        }))
    } 

    fn parse_named_closure
    (
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
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
            Some(closure),
        }))
    } 

    fn parse_return_stmt(
        &mut self,
        left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
    ) -> Option<Box<Ast>>
    {
        None
    }

    fn parse_continue_stmt(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
    ) -> Option<Box<Ast>>
    {
        None
    }

    fn parse_block(
        &mut self,
        _left: Option<Box<Ast>>,
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,
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
        //: Option<Weak<RefCell<Ast>>>,        
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
        //: Option<Weak<RefCell<Ast>>>,        
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
        //: Option<Weak<RefCell<Ast>>>,
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

    fn run(&self, root: &mut Option<Box<Ast>>) {
        let ret = self.print(root, 0);
        match ret {
            PrinterState::Done => {},
            PrinterState::NotDoneYet  => {
                panic!("\nAST printer is not done even though it should be"); 
            },
        }
    }

    fn print(&self, node: &mut Option<Box<Ast>>, level: usize) -> PrinterState {
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

    fn indent(&self, level: usize) {
        for _i in 0..level * 2 {
            print!(" ");
        }
    }
}


#[derive(Clone)]
struct SymbolTables {
    mods: HashMap<String, Module>,
    calls: Vec<FuncEntry>,
}

impl SymbolTables {
    fn new() -> Self {
        Self {
            mods: HashMap::<String, Module>::new(),
            calls: vec![],
        }
    }

    fn get_mod(&self, name: String) -> Result<&Module, String> {
        let res = self.mods.get(name);
        if res.is_some() {
            Ok(res.unwrap())
        } else {
            Err(format!("Module '{}' doesn't exist", name))
        }
    }
}

struct Module {
    mod_name: String,
    mod_index: usize,
    mod_vars: HashMap<String, VarEntry>,
    mod_env: Environment,
}

#[derive(Clone)]
struct FuncEntry {
    locals: Vec<LocalEntry>,
    upvalues: Vec<UpvalueEntry>,
    enclosing: Option<&FuncEntry>,
}



pub struct VarEntry {
    name: String,
    var_type: VarType,
    var_index: usize,
}

impl VarEntry {
    fn new(name: String, var_type: VarType, value: Option<Value>) -> Self {
        Self {
            name,
            var_type,
            var_index,
        }
    }

    pub fn get_name() -> &String {
        &name
    }

    pub fn get_type() -> &VarType {
        &var_type
    }
    
    pub fn get_index() -> &usize {
        &var_index
    }

    pub fn get value() -> &Value {
        &var_value
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



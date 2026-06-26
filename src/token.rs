use crate::value::*;
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    Err,
    None,
    Done,
    Int,
    Float,
    String,
    Ident,

    TrueKw,
    FalseKw,
    NilKw,
    LetKw,
    FunKw,
    StaticKw,
    MacroKw,
    ClassKw,
    NewKw,
    ThisKw,
    SuperKw,
    IfKw,
    ElseKw,
    WhileKw,
    DoKw,
    ForKw,
    ContinueKw,
    BreakKw,
    ReturnKw,
    

    Field,
    StaticField,

    AnonClosure,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Question,
    Dot,
    Comma,
    Colon,
    Semicolon,
    
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    And,
    Or,
    XOr,
    Not,
    LShift,
    RShift,
    LAnd,
    LOr,
    LNot,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    Hash,

    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    PowerAssign,
    AndAssign,
    OrAssign,
    XOrAssign,
    NotAssign,
    LShiftAssign,
    RShiftAssign,

    FieldAssign,
    IndexMethod,
    IndexAssignMethod,
    
}

#[derive(Clone)]
#[derive(Debug)]
pub enum TokenType {
    Err,
    None,
    Ident,
    Keyword,
    Operator,
    Other,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Token {
   pub kind: TokenKind,
   pub ty: TokenType,
   pub lexeme: String,
   pub line: usize,
   pub col: usize,
   pub value: Value,
}

impl Token {
    pub fn new(tk: TokenKind, tt: TokenType, lex: String, line: usize, col: usize, val: Value) -> Self {
        Self {
            kind: tk,
            ty: tt,
            lexeme: lex,
            line,
            col,
            value: val,
        }
    }
}

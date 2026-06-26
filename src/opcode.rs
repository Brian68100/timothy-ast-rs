use std::fmt::Display;

#[derive(Debug)]
#[allow(dead_code)]
pub enum MathOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Invalid,
}

impl From<u8> for MathOp {
    fn from(v: u8) -> MathOp {
        match v {
            0 => MathOp::Add,
            1 => MathOp::Subtract,
            2 => MathOp::Multiply,
            3 => MathOp::Divide,
            4 => MathOp::Modulo,
            5 => MathOp::Power,
            _ => MathOp::Invalid,
        }
    }
}

impl From<MathOp> for u8 {
    fn from(v: MathOp) -> u8 {
        match v {
            MathOp::Add      => 0,
            MathOp::Subtract => 1,
            MathOp::Multiply => 2,
            MathOp::Divide   => 3,
            MathOp::Modulo   => 4,
            MathOp::Power    => 5,
            MathOp::Invalid  => 6,
        }
    }
}

impl Display for MathOp {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            MathOp::Add => {
                write!(f, "ADD")
            },
            MathOp::Subtract => {
                write!(f, "SUBTRACT")
            },
            MathOp::Multiply => {
                write!(f, "MULTIPLY")
            },
            MathOp::Divide => {
                write!(f, "DIVIDE")
            },
            MathOp::Modulo => {
                write!(f, "MODULO")
            },
            MathOp::Power => {
                write!(f, "POWER")
            }, 
            MathOp::Invalid => {
                write!(f, "<INVALID>")
            },
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum BitwiseOp {
    And,
    Or,
    Xor,
    Not,
    LShift,
    RShift,
    Invalid,
}

impl From<u8> for BitwiseOp {
    fn from(v: u8) -> BitwiseOp {
        match v {
            0 => BitwiseOp::And,
            1 => BitwiseOp::Or,
            2 => BitwiseOp::Xor,
            3 => BitwiseOp::Not,
            4 => BitwiseOp::LShift,
            5 => BitwiseOp::RShift,
            _ => BitwiseOp::Invalid,
        }
    }
}

impl From<BitwiseOp> for u8 {
    fn from(v: BitwiseOp) -> u8 {
        match v {
            BitwiseOp::And     => 0,
            BitwiseOp::Or      => 1,
            BitwiseOp::Xor     => 2,
            BitwiseOp::Not     => 3,
            BitwiseOp::LShift  => 4,
            BitwiseOp::RShift  => 5,
            BitwiseOp::Invalid => 6,
        }
    }
}

impl Display for BitwiseOp {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self {
            BitwiseOp::And => {
                write!(f, "AND")
            },
            BitwiseOp::Or => {
                write!(f, "OR")
            },
            BitwiseOp::Xor => {
                write!(f, "XOR")
            },
            BitwiseOp::Not => {
                write!(f, "NOT")
            },
            BitwiseOp::LShift => {
                write!(f, "LSHIFT")
            },
            BitwiseOp::RShift => {
                write!(f, "RSHIFT")
            }, 
            BitwiseOp::Invalid => {
                write!(f, "<INVALID>")
            },
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum OpCode {
    PushN,
    Pop,
    PopN,
    Dup,
    LoadTrue,
    LoadFalse,
    LoadNil,
    Load0,
    Load1,
    Load2,
    Load3,
    Load4,
    Load5,
    Load6,
    Load7,
    Load8,
    Load9,
    Load10,
    LoadValue,
    Array,
    Dict,
    DefVar,
    LoadVar,
    StoreVar,
    LoadLocal,
    LoadLocal0,
    LoadLocal1,
    LoadLocal2,
    LoadLocal3,
    LoadLocal4,
    LoadLocal5,
    LoadLocal6,
    LoadLocal7,
    LoadLocal8,
    StoreLocal,
    StoreLocal0,
    StoreLocal1,
    StoreLocal2,
    StoreLocal3,
    StoreLocal4,
    StoreLocal5,
    StoreLocal6,
    StoreLocal7,
    StoreLocal8,
    LoadUpvalue,
    StoreUpvalue,
    LoadField,
    StoreField,
    LoadStatic,
    StoreStatic,
    LoadMethod,
    LoadStaticMethod,
    LoadSuperMethod,
    JumpFwd,
    JumpBack,
    JumpTrue,
    JumpFalse,
    DefStatic,
    Method,
    StaticMethod,
    Instance,
    Closure,
    CloseUpvalue,
    Call,
    Return,
    Ternary,
    Neg,
    Print,
    Println,
    Input,
    MathOp,
    BitwiseOp,
    MathAssignOp,
    BitwiseAssignOp,
    Invalid,
}

impl From<u8> for OpCode {
    fn from(v: u8) -> OpCode {
        match v {
            0 => OpCode::PushN,
            1 => OpCode::Pop,
            2 => OpCode::PopN,
            3 => OpCode::Dup,
            4 => OpCode::LoadTrue,
            5 => OpCode::LoadFalse,
            6 => OpCode::LoadNil,
            7 => OpCode::Load0,
            8 => OpCode::Load1,
            9 => OpCode::Load2,
            10 => OpCode::Load3,
            11 => OpCode::Load4,
            12 => OpCode::Load5,
            13 => OpCode::Load6,
            14 => OpCode::Load7,
            15 => OpCode::Load8,
            16 => OpCode::Load9,
            17 => OpCode::Load10,
            18 => OpCode::LoadValue,
            19 => OpCode::Array,
            20 => OpCode::Dict,
            21 => OpCode::DefVar,
            22 => OpCode::LoadVar,
            23 => OpCode::StoreVar,
            24 => OpCode::LoadLocal,
            25 => OpCode::LoadLocal0,
            26 => OpCode::LoadLocal1,
            27 => OpCode::LoadLocal2,
            28 => OpCode::LoadLocal3,
            29 => OpCode::LoadLocal4,
            30 => OpCode::LoadLocal5,
            31 => OpCode::LoadLocal6,
            32 => OpCode::LoadLocal7,
            33 => OpCode::LoadLocal8,
            34 => OpCode::StoreLocal,
            35 => OpCode::StoreLocal0,
            36 => OpCode::StoreLocal1,
            37 => OpCode::StoreLocal2,
            38 => OpCode::StoreLocal3,
            39 => OpCode::StoreLocal4,
            40 => OpCode::StoreLocal5,
            41 => OpCode::StoreLocal6,
            42 => OpCode::StoreLocal7,
            43 => OpCode::StoreLocal8,
            44 => OpCode::LoadUpvalue,
            45 => OpCode::StoreUpvalue,
            46 => OpCode::LoadField,
            47 => OpCode::StoreField,
            48 => OpCode::LoadStatic,
            49 => OpCode::StoreStatic,
            50 => OpCode::LoadMethod,
            51 => OpCode::LoadStaticMethod,
            52 => OpCode::LoadSuperMethod,
            53 => OpCode::JumpFwd,
            54 => OpCode::JumpBack,
            55 => OpCode::JumpTrue,
            56 => OpCode::JumpFalse,
            57 => OpCode::DefStatic,
            58 => OpCode::Method,
            59 => OpCode::StaticMethod,
            60 => OpCode::Instance,
            61 => OpCode::Closure,
            62 => OpCode::CloseUpvalue,
            63 => OpCode::Call,
            64 => OpCode::Return,
            65 => OpCode::Ternary,
            66 => OpCode::Neg,
            67 => OpCode::Print,
            68 => OpCode::Println,
            69 => OpCode::Input,
            70 => OpCode::MathOp,
            71 => OpCode::BitwiseOp,
            72 => OpCode::MathAssignOp,
            73 => OpCode::BitwiseAssignOp,
            _ => OpCode::Invalid,
        }
    }
}

impl From<OpCode> for u8 {
    fn from(v: OpCode) -> u8 {
        match v {
            OpCode::PushN => 0,
            OpCode::Pop => 1,
            OpCode::PopN => 2,
            OpCode::Dup => 3,
            OpCode::LoadTrue => 4,
            OpCode::LoadFalse => 5,
            OpCode::LoadNil => 6,
            OpCode::Load0 => 7,
            OpCode::Load1 => 8,
            OpCode::Load2 => 9,
            OpCode::Load3 => 10,
            OpCode::Load4 => 11,
            OpCode::Load5 => 12,
            OpCode::Load6 => 13,
            OpCode::Load7 => 14,
            OpCode::Load8 => 15,
            OpCode::Load9 => 16,
            OpCode::Load10 => 17,
            OpCode::LoadValue => 18,
            OpCode::Array => 19,
            OpCode::Dict => 20,
            OpCode::DefVar => 21,
            OpCode::LoadVar => 22,
            OpCode::StoreVar => 23,
            OpCode::LoadLocal => 24,
            OpCode::LoadLocal0 => 25,
            OpCode::LoadLocal1 => 26,
            OpCode::LoadLocal2 => 27,
            OpCode::LoadLocal3 => 28,
            OpCode::LoadLocal4 => 29,
            OpCode::LoadLocal5 => 30,
            OpCode::LoadLocal6 => 31,
            OpCode::LoadLocal7 => 32,
            OpCode::LoadLocal8 => 33,
            OpCode::StoreLocal => 34,
            OpCode::StoreLocal0 => 35,
            OpCode::StoreLocal1 => 36,
            OpCode::StoreLocal2 => 37,
            OpCode::StoreLocal3 => 38,
            OpCode::StoreLocal4 => 39,
            OpCode::StoreLocal5 => 40,
            OpCode::StoreLocal6 => 41,
            OpCode::StoreLocal7 => 42,
            OpCode::StoreLocal8 => 43,
            OpCode::LoadUpvalue => 44,
            OpCode::StoreUpvalue => 45,
            OpCode::LoadField => 46,
            OpCode::StoreField => 47,
            OpCode::LoadStatic => 48,
            OpCode::StoreStatic => 49,
            OpCode::LoadMethod => 50,
            OpCode::LoadStaticMethod => 51,
            OpCode::LoadSuperMethod => 52,
            OpCode::JumpFwd => 53,
            OpCode::JumpBack => 54,
            OpCode::JumpTrue => 55,
            OpCode::JumpFalse => 56,
            OpCode::DefStatic => 57,
            OpCode::Method => 58,
            OpCode::StaticMethod => 59,
            OpCode::Instance => 60,
            OpCode::Closure => 61,
            OpCode::CloseUpvalue => 62,
            OpCode::Call => 63,
            OpCode::Return => 64,
            OpCode::Ternary => 65,
            OpCode::Neg => 66,
            OpCode::Print => 67,
            OpCode::Println => 68,
            OpCode::Input => 69,
            OpCode::MathOp => 70,
            OpCode::BitwiseOp => 71,
            OpCode::MathAssignOp => 72,
            OpCode::BitwiseAssignOp => 73,
            OpCode::Invalid => 74,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        match self { 
            OpCode::PushN => {
                write!(f, "PUSHN")
            },
            OpCode::Pop => {
                write!(f, "POP")
            },
            OpCode::PopN => {
                write!(f, "POPN")
            },
            OpCode::Dup => {
                write!(f, "DUP")
            },
            OpCode::LoadTrue => {
                write!(f, "LOAD_TRUE")
            },
            OpCode::LoadFalse => {
                write!(f, "LOAD_FALSE")
            },
            OpCode::LoadNil => {
                write!(f, "LOAD_NIL")
            },
            OpCode::Load0 => {
                write!(f, "LOAD0")
            },
            OpCode::Load1 => {
                write!(f, "LOAD1")
            },
            OpCode::Load2 => {
                write!(f, "LOAD2")
            },
            OpCode::Load3 => {
                write!(f, "LOAD3")
            },
            OpCode::Load4 => {
                write!(f, "LOAD4")
            },
            OpCode::Load5 => {
                write!(f, "LOAD5")
            },
            OpCode::Load6 => {
                write!(f, "LOAD6")
            },
            OpCode::Load7 => {
                write!(f, "LOAD7")
            },
            OpCode::Load8 => {
                write!(f, "LOAD8")
            },
            OpCode::Load9 => {
                write!(f, "LOAD9")
            },
            OpCode::Load10 => {
                write!(f, "LOAD10")
            },
            OpCode::LoadValue => {
                write!(f, "LOAD_VALUE")
            },
            OpCode::Array => {
                write!(f, "ARRAY")
            },
            OpCode::Dict => {
                write!(f, "DICT")
            },
            OpCode::DefVar => {
                write!(f, "DEF_VAR")
            },
            OpCode::LoadVar => {
                write!(f, "LOAD_VAR")
            },
            OpCode::StoreVar => {
                write!(f, "STORE_VAR")
            },
            OpCode::LoadLocal => {
                write!(f, "LOAD_LOCAL")
            },
            OpCode::LoadLocal0 => {
                write!(f, "LOAD_LOCAL0")
            },
            OpCode::LoadLocal1 => {
                write!(f, "LOAD_LOCAL1")
            },
            OpCode::LoadLocal2 => {
                write!(f, "LOAD_LOCAL2")
            },
            OpCode::LoadLocal3 => {
                write!(f, "LOAD_LOCAL3")
            },
            OpCode::LoadLocal4 => {
                write!(f, "LOAD_LOCAL4")
            },
            OpCode::LoadLocal5 => {
                write!(f, "LOAD_LOCAL5")
            },
            OpCode::LoadLocal6 => {
                write!(f, "LOAD_LOCAL6")
            },
            OpCode::LoadLocal7 => {
                write!(f, "LOAD_LOCAL7")
            },
            OpCode::LoadLocal8 => {
                write!(f, "LOAD_LOCAL8")
            },
            OpCode::StoreLocal => {
                write!(f, "STORE_LOCAL")
            },
            OpCode::StoreLocal0 => {
                write!(f, "STORE_LOCAL0")
            },
            OpCode::StoreLocal1 => {
                write!(f, "STORE_LOCAL1")
            },
            OpCode::StoreLocal2 => {
                write!(f, "STORE_LOCAL2")
            },
            OpCode::StoreLocal3 => {
                write!(f, "STORE_LOCAL3")
            },
            OpCode::StoreLocal4 => {
                write!(f, "STORE_LOCAL4")
            },
            OpCode::StoreLocal5 => {
                write!(f, "STORE_LOCAL5")
            },
            OpCode::StoreLocal6 => {
                write!(f, "STORE_LOCAL6")
            },
            OpCode::StoreLocal7 => {
                write!(f, "STORE_LOCAL7")
            },
            OpCode::StoreLocal8 => {
                write!(f, "STORE_LOCAL8")
            },
            OpCode::LoadUpvalue => {
                write!(f, "LOAD_UPVALUE")
            },
            OpCode::StoreUpvalue => {
                write!(f, "STORE_UPVALUE")
            },
            OpCode::LoadField => {
                write!(f, "LOAD_FIELD")
            },
            OpCode::StoreField => {
                write!(f, "STORE_FIELD")
            },
            OpCode::LoadStatic => {
                write!(f, "LOAD_STATIC")
            },
            OpCode::StoreStatic => {
                write!(f, "STORE_STATIC")
            },
            OpCode::LoadMethod => {
                write!(f, "LOAD_METHOD")
            },
            OpCode::LoadStaticMethod => {
                write!(f, "LOAD_STATIC_METHOD")
            },
            OpCode::LoadSuperMethod => {
                write!(f, "LOAD_SUPER_METHOD")
            },
            OpCode::JumpFwd => {
                write!(f, "JUMP_FWD")
            },
            OpCode::JumpBack => {
                write!(f, "JUMP_BACK")
            },
            OpCode::JumpTrue => {
                write!(f, "JUMP_TRUE")
            },
            OpCode::JumpFalse => {
                write!(f, "JUMP_FALSE")
            },
            OpCode::DefStatic => {
                write!(f, "DEF_STATIC")
            },
            OpCode::Method => {
                write!(f, "METHOD")
            },
            OpCode::StaticMethod => {
                write!(f, "STATIC_METHOD")
            },
            OpCode::Instance => {
                write!(f, "INSTANCE")
            },
            OpCode::Closure => {
                write!(f, "CLOSURE")
            },
            OpCode::CloseUpvalue => {
                write!(f, "CLOSE_UPVALUE")
            },
            OpCode::Call => {
                write!(f, "CALL")
            }, 
            OpCode::Return => {
                write!(f, "RETURN")
            },
            OpCode::Ternary => {
                write!(f, "TERNARY")
            },
            OpCode::Neg => {
                write!(f, "NEG")
            },
            OpCode::Print => {
                write!(f, "PRINT")
            },
            OpCode::Println => {
                write!(f, "PRINTLN")
            },
            OpCode::Input => {
                write!(f, "INPUT")
            },
            OpCode::MathOp => {
                write!(f, "MATH")
            },
            OpCode::MathAssignOp => {
                write!(f, "MATH_ASSIGN")
            },
            OpCode::BitwiseOp => {
                write!(f, "BITWISE")
            },
            OpCode::BitwiseAssignOp => {
                write!(f, "BITWISE_ASSIGN")
            },
            OpCode::Invalid => {
                write!(f, "<INVALID>")
            },
        }
    }
}

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
    AddMod,
    SwitchMod,
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
            0  => OpCode::PushN,
            1  => OpCode::Pop,
            2  => OpCode::PopN,
            3  => OpCode::Dup,
            4  => OpCode::AddMod,
            5  => OpCode::SwitchMod,
            6  => OpCode::LoadTrue,
            7  => OpCode::LoadFalse,
            8  => OpCode::LoadNil,
            9  => OpCode::Load0,
            10 => OpCode::Load1,
            11 => OpCode::Load2,
            12 => OpCode::Load3,
            13 => OpCode::Load4,
            14 => OpCode::Load5,
            15 => OpCode::Load6,
            16 => OpCode::Load7,
            17 => OpCode::Load8,
            18 => OpCode::Load9,
            19 => OpCode::Load10,
            20 => OpCode::LoadValue,
            21 => OpCode::Array,
            22 => OpCode::Dict,
            23 => OpCode::DefVar,
            24 => OpCode::LoadVar,
            25 => OpCode::StoreVar,
            26 => OpCode::LoadLocal,
            27 => OpCode::LoadLocal0,
            28 => OpCode::LoadLocal1,
            29 => OpCode::LoadLocal2,
            30 => OpCode::LoadLocal3,
            31 => OpCode::LoadLocal4,
            32 => OpCode::LoadLocal5,
            33 => OpCode::LoadLocal6,
            34 => OpCode::LoadLocal7,
            35 => OpCode::LoadLocal8,
            36 => OpCode::StoreLocal,
            37 => OpCode::StoreLocal0,
            38 => OpCode::StoreLocal1,
            39 => OpCode::StoreLocal2,
            40 => OpCode::StoreLocal3,
            41 => OpCode::StoreLocal4,
            42 => OpCode::StoreLocal5,
            43 => OpCode::StoreLocal6,
            44 => OpCode::StoreLocal7,
            45 => OpCode::StoreLocal8,
            46 => OpCode::LoadUpvalue,
            47 => OpCode::StoreUpvalue,
            48 => OpCode::LoadField,
            49 => OpCode::StoreField,
            50 => OpCode::LoadStatic,
            51 => OpCode::StoreStatic,
            52 => OpCode::LoadMethod,
            53 => OpCode::LoadStaticMethod,
            54 => OpCode::LoadSuperMethod,
            55 => OpCode::JumpFwd,
            56 => OpCode::JumpBack,
            57 => OpCode::JumpTrue,
            58 => OpCode::JumpFalse,
            59 => OpCode::DefStatic,
            60 => OpCode::Method,
            61 => OpCode::StaticMethod,
            62 => OpCode::Instance,
            63 => OpCode::Closure,
            64 => OpCode::CloseUpvalue,
            65 => OpCode::Call,
            66 => OpCode::Return,
            67 => OpCode::Ternary,
            68 => OpCode::Neg,
            69 => OpCode::Print,
            70 => OpCode::Println,
            71 => OpCode::Input,
            72 => OpCode::MathOp,
            73 => OpCode::BitwiseOp,
            74 => OpCode::MathAssignOp,
            75 => OpCode::BitwiseAssignOp,
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
            OpCode::AddMods => 4,
            OpCode::SwitchMod => 5,
            OpCode::LoadTrue => 6,
            OpCode::LoadFalse => 7,
            OpCode::LoadNil => 8,
            OpCode::Load0 => 9,
            OpCode::Load1 => 10,
            OpCode::Load2 => 11,
            OpCode::Load3 => 12,
            OpCode::Load4 => 13,
            OpCode::Load5 => 14,
            OpCode::Load6 => 15,
            OpCode::Load7 => 16,
            OpCode::Load8 => 17,
            OpCode::Load9 => 18,
            OpCode::Load10 => 19,
            OpCode::LoadValue => 20,
            OpCode::Array => 21,
            OpCode::Dict => 22,
            OpCode::DefVar => 23,
            OpCode::LoadVar => 24,
            OpCode::StoreVar => 25,
            OpCode::LoadLocal => 26,
            OpCode::LoadLocal0 => 27,
            OpCode::LoadLocal1 => 28,
            OpCode::LoadLocal2 => 29,
            OpCode::LoadLocal3 => 30,
            OpCode::LoadLocal4 => 31,
            OpCode::LoadLocal5 => 32,
            OpCode::LoadLocal6 => 33,
            OpCode::LoadLocal7 => 34,
            OpCode::LoadLocal8 => 35,
            OpCode::StoreLocal => 36,
            OpCode::StoreLocal0 => 37,
            OpCode::StoreLocal1 => 38,
            OpCode::StoreLocal2 => 39,
            OpCode::StoreLocal3 => 40,
            OpCode::StoreLocal4 => 41,
            OpCode::StoreLocal5 => 42,
            OpCode::StoreLocal6 => 43,
            OpCode::StoreLocal7 => 44,
            OpCode::StoreLocal8 => 45,
            OpCode::LoadUpvalue => 46,
            OpCode::StoreUpvalue => 47,
            OpCode::LoadField => 48,
            OpCode::StoreField => 49,
            OpCode::LoadStatic => 50,
            OpCode::StoreStatic => 51,
            OpCode::LoadMethod => 52,
            OpCode::LoadStaticMethod => 53,
            OpCode::LoadSuperMethod => 54,
            OpCode::JumpFwd => 55,
            OpCode::JumpBack => 56,
            OpCode::JumpTrue => 57,
            OpCode::JumpFalse => 58,
            OpCode::DefStatic => 59,
            OpCode::Method => 60,
            OpCode::StaticMethod => 61,
            OpCode::Instance => 62,
            OpCode::Closure => 63,
            OpCode::CloseUpvalue => 64,
            OpCode::Call => 65,
            OpCode::Return => 66,
            OpCode::Ternary => 67,
            OpCode::Neg => 68,
            OpCode::Print => 69,
            OpCode::Println => 70,
            OpCode::Input => 71,
            OpCode::MathOp => 72,
            OpCode::BitwiseOp => 73,
            OpCode::MathAssignOp => 74,
            OpCode::BitwiseAssignOp => 75,
            OpCode::Invalid => 76,
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
            OpCode::AddMod => {
                write!(f, "ADD_MODS")
            },
            OpCode::ChangeMod => {
                write!(f, "SWITCH_MOD")
            }
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

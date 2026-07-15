use crate::value::Value;
use std::{fmt, mem};
use crate::utils::do_if_some;
use crate::managed::*;
use std::fmt::{Display, Formatter, Error};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum FunctionType {
    TopLevel, 
    NamedClosure,
    AnonClosure,
    Constructor,
    Method,
}

impl Display for FunctionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            FunctionType::TopLevel => {
                write!(f, "Top Level")
            }, 
            FunctionType::NamedClosure => {
                write!(f, "Named Closure")
            },
            FunctionType::AnonClosure => {
                write!(f, "Anonymous Closure")
            },
            FunctionType::Constructor => {
                write!(f, "Constructor")
            },
            FunctionType::Method => {
                write!(f, "Method")
            },
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FunctionCore {
    name: String,
    ty: FunctionType,
    arity: usize,
    code: Vec<u8>,
    constants: Vec<Value>,
    upvalue_count: usize,
}
	

	
impl FunctionCore {
    pub fn new_main(
    ) -> Self
    {
        FunctionCore::new(
            "".to_string(),
            FunctionType::TopLevel,
            0,
            vec![],
            vec![],
            0,
        )
    }

    pub fn new_named_closure(
        name: String, 
        arity: usize,
    ) -> Self
    {
        FunctionCore::new(
            name,
            FunctionType::NamedClosure, 
            arity,
            vec![],
            vec![],
            0,
        )
    }

    pub fn new_anon_closure(
        arity: usize,
    ) -> Self
    {
        FunctionCore::new(
            "".to_string(), 
            FunctionType::AnonClosure, 
            arity,
            vec![],
            vec![],
            0,
        )
    }

    pub fn new_default_constructor(
    ) -> Self
    {
        FunctionCore::new(
            "".to_string(),
            FunctionType::Constructor,
            0,
            vec![],
            vec![],
            0,
        )
    }

    pub fn new_constructor(
        name: String,
        arity: usize,
    ) -> Self
    {
        FunctionCore::new(
            name,
            FunctionType::Constructor,
            arity,
            vec![],
            vec![],
            0,
        )
    }

    fn new(name: String, ty: FunctionType, arity: usize, code: Vec<u8>, constants: Vec<Value>, upvalue_count: usize) -> Self
    { 
        Self {
            name,
            ty,
            arity,
            code: code.to_vec(),
            constants: constants.to_vec(),
            upvalue_count: 0,
        }
    }
    
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_type(&self) -> FunctionType {
        self.ty.clone()
    }
    
    pub fn get_arity(&self) -> usize {
        self.arity
    }

    pub fn get_upvalue_count(&self) -> usize {
        self.upvalue_count
    }

    pub fn get_byte(&self, index: usize) -> Option<&u8> {
        if index < self.code.len() {
            Some(&self.code[index])
        } else {
            None
        }
    }

    pub fn finish(&mut self, constants: Vec<Value>, code: Vec<u8>) -> bool {
        if self.code.is_empty() {
            self.constants = constants.clone();
            self.code = code.clone();
            return true;
        }
        return false;
    } 

    #[inline]
    pub fn len(&self) -> usize {
        self.code.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
    #[inline]
    pub fn get_code(&self) -> &[u8] {
        &self.code[0..self.code.len()]
    }
    pub(crate) fn get_constants(&self) -> &Vec<Value> {
        &self.constants
    }
}

impl GcTrace for FunctionCore {
    fn trace(&self, mark: &mut dyn FnMut(Managed<dyn Manage>)) -> bool {
        self
            .get_constants()
            .iter()
            .for_each(|constant| do_if_some(constant.get_dyn_managed(), 
                |obj| mark(obj)));

        true
    }
}

impl Manage for FunctionCore {
    fn type_name(&self) -> &str {
        "function core"
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl fmt::Debug for FunctionCore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FunctionCore")
            .field("arity", &self.arity)
            .field("upvalue_count", &self.upvalue_count)
            .field("name", &self.name)
            .finish()
    }
}

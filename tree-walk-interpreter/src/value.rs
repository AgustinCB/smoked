use crate::class::{LoxClass, LoxObject};
use crate::function::LoxFunction;
use parser::types::{DataKeyword, FunctionHeader, Literal, ProgramError, SourceCodeLocation};
use std::cell::RefCell;
use std::convert::{TryInto, TryFrom};
use std::fmt::{Display, Error, Formatter, Debug};
use std::ops::{Neg, Not};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct LoxTrait<'a> {
    pub name: &'a str,
    pub methods: Vec<FunctionHeader<'a>>,
    pub getters: Vec<FunctionHeader<'a>>,
    pub setters: Vec<FunctionHeader<'a>>,
    pub static_methods: Vec<FunctionHeader<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct LoxArray<'a> {
    pub capacity: usize,
    pub elements: Vec<Box<Value<'a>>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'a> {
    Nil,
    Uninitialized,
    Boolean {
        value: bool,
    },
    Integer {
        value: i64,
    },
    Float {
        value: f32,
    },
    String {
        value: String,
    },
    Function(Rc<LoxFunction<'a>>),
    Method(Rc<LoxFunction<'a>>, Rc<LoxObject<'a>>),
    Class(Rc<LoxClass<'a>>),
    Object(Rc<LoxObject<'a>>),
    Trait(Rc<LoxTrait<'a>>),
    Array(Rc<RefCell<LoxArray<'a>>>),
    Module(&'a str),
}

impl<'a> Value<'a> {
    pub fn is_number(&self) -> bool {
        match self {
            Value::Integer { .. } => true,
            Value::Float { .. } => true,
            _ => false,
        }
    }

    pub fn is_class(&self) -> bool {
        match self {
            Value::Class { .. } => true,
            _ => false,
        }
    }

    pub fn is_trait(&self) -> bool {
        match self {
            Value::Trait { .. } => true,
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Uninitialized => false,
            Value::Boolean { value: false } => false,
            Value::Float { value } if *value == 0f32 => false,
            Value::Integer { value } if *value == 0 => false,
            _ => true,
        }
    }
}

impl<'a> Neg for Value<'a> {
    type Output = Value<'a>;

    fn neg(self) -> Value<'a> {
        match self {
            Value::Integer { value } => Value::Integer { value: -value },
            Value::Float { value } => Value::Float { value: -value },
            _ => panic!("Only numbers can change sign"),
        }
    }
}

impl<'a> Not for Value<'a> {
    type Output = Value<'a>;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean { value } => Value::Boolean { value: !value },
            _ => Value::Boolean {
                value: !self.is_truthy(),
            },
        }
    }
}

pub enum ValueError {
    ExpectingDouble,
    ExpectingInteger,
    ExpectingNumber,
    ExpectingString,
}

impl ValueError {
    pub fn into_program_error<'a>(self, location: &SourceCodeLocation<'a>) -> ProgramError<'a> {
        ProgramError {
            location: location.clone(),
            message: self.to_string(),
        }
    }
}

impl ToString for ValueError {
    fn to_string(&self) -> String {
        match self {
            ValueError::ExpectingDouble => "Type error! Expecting a double!".to_owned(),
            ValueError::ExpectingInteger => "Type error! Expecting an integer!".to_owned(),
            ValueError::ExpectingNumber => "Type error! Expecting a number!".to_owned(),
            ValueError::ExpectingString => "Type error! Expecting a string!".to_owned(),
        }
    }
}

impl<'a> TryFrom<Value<'a>> for i64 {
    type Error = ValueError;
    fn try_from(value: Value<'a>) -> Result<i64, Self::Error> {
        match value {
            Value::Integer { value } => Ok(value),
            Value::Float { value } => Ok(value as _),
            _ => Err(ValueError::ExpectingDouble),
        }
    }
}

impl<'a> TryFrom<Value<'a>> for f32 {
    type Error = ValueError;
    fn try_from(value: Value<'a>) -> Result<f32, Self::Error> {
        match value {
            Value::Float { value } => Ok(value),
            Value::Integer { value } => Ok(value as _),
            _ => Err(ValueError::ExpectingDouble),
        }
    }
}

impl<'a> TryInto<String> for Value<'a> {
    type Error = ValueError;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            Value::String { value } => Ok(value),
            _ => Err(ValueError::ExpectingString),
        }
    }
}

impl<'a> Into<Value<'a>> for &Literal<'a> {
    fn into(self) -> Value<'a> {
        match self {
            Literal::Float(value) => Value::Float { value: *value },
            Literal::Integer(value) => Value::Integer { value: *value as _ },
            Literal::QuotedString(value) => Value::String {
                value: (*value).to_owned(),
            },
            Literal::Keyword(DataKeyword::Nil) => Value::Nil,
            Literal::Keyword(DataKeyword::True) => Value::Boolean { value: true },
            Literal::Keyword(DataKeyword::False) => Value::Boolean { value: false },
        }
    }
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Value::Float { value } => f.write_str(value.to_string().as_str()),
            Value::Integer { value } => f.write_str(value.to_string().as_str()),
            Value::String { value } => f.write_str(value.as_str()),
            Value::Boolean { value } => f.write_str(value.to_string().as_str()),
            Value::Uninitialized => f.write_str("Uninitialized"),
            Value::Nil => f.write_str("Nil"),
            Value::Function(lf) => f.write_str(format!("{:?}", *lf).as_str()),
            Value::Class(c) => f.write_str(format!("{}", c.name).as_str()),
            Value::Object(c) => f.write_str(format!("{} instance", c.class_name).as_str()),
            Value::Method(lf, o) => f.write_str(format!("Method {:?} of {}", lf, o.class_name).as_str()),
            Value::Trait(t) => f.write_str(t.name),
            Value::Array(a) => {
                f.write_str("[ ")?;
                for e in a.borrow().elements.iter() {
                    f.write_str(format!("{}, ", e).as_str())?;
                }
                f.write_str("]")
            }
            Value::Module(_) => f.write_str("[Module]"),
        }
    }
}

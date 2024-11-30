use std::{collections::HashMap, str::FromStr};

use crate::{error::ValueError, opt::Value};

#[derive(Debug, PartialEq)]
pub struct Matches {
    exec_name: String, 
    positional: Vec<String>, 
    named: HashMap<String, Value>,
}

impl Matches {
    pub fn new(exec_name: String, positional: Vec<String>, named: HashMap<String, Value>) -> Self {
        Self {exec_name, positional, named}
    }
    
    pub fn flag(&self, name: &str) -> Result<Option<bool>, ValueError> {
        match self.named.get(name) {
            Some(Value::Flag(b)) => Ok(Some(*b)),
            Some(_) => Err(ValueError::WrongOptionType),
            None => Ok(None),
        }
    }

    pub fn one<T: FromStr>(&self, name: &str) -> Result<Option<T>, ValueError> {
        match self.named.get(name) {
            Some(Value::Single(val)) => {
                let v = T::from_str(val.as_str())
                    .map_err(|_| ValueError::ConversionError(val.clone()))?;
                Ok(Some(v))
            }
            Some(_) => Err(ValueError::WrongOptionType),
            None => Ok(None),
        }
    }

    pub fn all<T: FromStr>(&self, name: &str) -> Result<Vec<T>, ValueError> {
        match self.named.get(name) {
            Some(Value::Multi(val)) => {
                let r: Result<Vec<T>, _> = val.iter().map(|v| T::from_str(v.as_str())).collect();
                r.map_err(|_| ValueError::ConversionError(format!("{:?}", val)))
            }
            Some(_) => Err(ValueError::WrongOptionType),
            None => Ok(vec![]),
        }
    }

    pub fn positional(&self) -> &[String] {
        &self.positional
    }
}

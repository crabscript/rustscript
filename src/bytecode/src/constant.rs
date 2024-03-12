#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub enum Type {
    #[default]
    Int,
    Float,
    Bool,
}

pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub type RawValue = u64;

pub fn get_value(t: &Type, v: RawValue) -> Value {
    match t {
        Type::Int => Value::Int(v as i64),
        Type::Float => Value::Float(f64::from_bits(v)),
        Type::Bool => Value::Bool(v != 0),
    }
}

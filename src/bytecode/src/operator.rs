use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BinOp {
    /// Addition of two values of the same type (int or float or string)
    Add,
    /// Subtraction of two values of the same type (int or float)
    Sub,
    /// Multiplication of two values of the same type (int or float)
    Mul,
    /// Division of two values of the same type (int or float)
    Div,
    /// Modulo of two values of the same type (int)
    Mod,
    /// Greater than comparison of two values of the same type (int or float)
    Gt,
    /// Less than comparison of two values of the same type (int or float)
    Lt,
    /// Equality comparison of two values of the same type (bool or int or float or string)
    Eq,
    /// Logical AND of two values of the same type (bool)
    And,
    /// Logical OR of two values of the same type (bool)
    Or,
}

impl From<&str> for BinOp {
    fn from(s: &str) -> Self {
        match s {
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            "%" => BinOp::Mod,
            ">" => BinOp::Gt,
            "<" => BinOp::Lt,
            "==" => BinOp::Eq,
            "&&" => BinOp::And,
            "||" => BinOp::Or,
            _ => panic!("Invalid binary operator: {}", s),
        }
    }
}

impl From<BinOp> for String {
    fn from(op: BinOp) -> Self {
        match op {
            BinOp::Add => "+".to_string(),
            BinOp::Sub => "-".to_string(),
            BinOp::Mul => "*".to_string(),
            BinOp::Div => "/".to_string(),
            BinOp::Mod => "%".to_string(),
            BinOp::Gt => ">".to_string(),
            BinOp::Lt => "<".to_string(),
            BinOp::Eq => "==".to_string(),
            BinOp::And => "&&".to_string(),
            BinOp::Or => "||".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum UnOp {
    /// Negation of a value of the same type (int or float)
    Neg,
    /// Logical negation of a value of the same type (bool)
    Not,
}

impl From<&str> for UnOp {
    fn from(s: &str) -> Self {
        match s {
            "-" => UnOp::Neg,
            "!" => UnOp::Not,
            _ => panic!("Invalid unary operator: {}", s),
        }
    }
}

impl From<UnOp> for String {
    fn from(op: UnOp) -> Self {
        match op {
            UnOp::Neg => "-".to_string(),
            UnOp::Not => "!".to_string(),
        }
    }
}

use parser::{Type, UnOpType};
use std::{collections::HashMap, fmt::Display};

use parser::{BlockSeq, Decl, Expr};

#[derive(Debug, PartialEq)]
pub struct TypeErrors {
    errs: Vec<String>,
}

impl TypeErrors {
    pub fn new() -> TypeErrors {
        TypeErrors { errs: vec![] }
    }

    pub fn new_err(err: &str) -> TypeErrors {
        TypeErrors {
            errs: vec![err.to_string()],
        }
    }

    pub fn add(&mut self, err: &str) {
        self.errs.push(err.to_string());
    }

    pub fn append(&mut self, errs: &mut TypeErrors) {
        self.errs.append(&mut errs.errs)
    }

    pub fn is_ok(&self) -> bool {
        self.errs.is_empty()
    }
}

impl Display for TypeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .errs
            .iter()
            .map(|x| format!("[TypeError]: {}", x))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", string)
    }
}

impl std::error::Error for TypeErrors {}

/// Struct to enable type checking by encapsulating type environment.
pub struct TypeChecker<'prog> {
    program: &'prog BlockSeq,
}

impl<'prog> TypeChecker<'prog> {
    pub fn new(program: &BlockSeq) -> TypeChecker<'_> {
        TypeChecker { program }
    }

    fn check_unop(
        op: &UnOpType,
        expr: &Expr,
        ty_env: &mut HashMap<String, Type>,
    ) -> Result<Type, TypeErrors> {
        match op {
            UnOpType::Negate => {
                // Return err imm if operand itself is not well typed
                let ty = TypeChecker::check_expr(expr, ty_env)?;
                match ty {
                    Type::Int | Type::Float => Ok(ty),
                    _ => {
                        let e = format!("Can't negate type {}", ty);
                        Err(TypeErrors::new_err(&e))
                    }
                }
            }
            UnOpType::Not => {
                let ty = TypeChecker::check_expr(expr, ty_env)?;
                match ty {
                    Type::Bool => Ok(ty),
                    _ => {
                        let e = format!("Can't apply logical NOT to type {}", ty);
                        Err(TypeErrors::new_err(&e))
                    }
                }
            }
        }
    }

    /// Return the type errors out instead of using mutable ref
    // because for nested errors in the expr we want to propagate those
    fn check_expr(expr: &Expr, ty_env: &mut HashMap<String, Type>) -> Result<Type, TypeErrors> {
        let local_errs = TypeErrors::new();
        let ty = match expr {
            Expr::Integer(_) => Type::Int,
            Expr::Float(_) => Type::Float,
            Expr::Bool(_) => Type::Bool,
            Expr::Symbol(ident) => {
                let get_type = ty_env.get(ident);
                if get_type.is_none() {
                    let e = format!("Identifier '{}' not declared", ident);
                    return Err(TypeErrors::new_err(&e));
                }

                get_type.expect("Should have type").to_owned()
            }
            Expr::UnOpExpr(op, expr) => {
                return TypeChecker::check_unop(op, expr, ty_env);
            }
            _ => todo!(),
        };

        if local_errs.is_ok() {
            Ok(ty)
        } else {
            Err(local_errs)
        }
    }

    /// Type check declaration and add errors if any
    fn check_decl(decl: &Decl, ty_env: &mut HashMap<String, Type>) -> Result<(), TypeErrors> {
        // dbg!("Type checking decl:", decl);
        match decl {
            Decl::LetStmt(stmt) => {
                let expr_type = TypeChecker::check_expr(&stmt.expr, ty_env)?;
                if let Some(ty_ann) = stmt.type_ann {
                    // annotation and assigned type not equal
                    if !ty_ann.eq(&expr_type) {
                        let string = format!(
                            "'{}' has declared type {} but assigned type {}",
                            stmt.ident, ty_ann, expr_type
                        );
                        return Err(TypeErrors::new_err(&string));
                    }
                }

                // assign type to ident
                ty_env.insert(stmt.ident.to_owned(), expr_type);
            }
            // Type check the expr and return any errors
            Decl::ExprStmt(expr) => {
                TypeChecker::check_expr(expr, ty_env)?;
            }
            _ => todo!(),
        }

        Ok(())
    }

    pub fn type_check(self) -> Result<Type, TypeErrors> {
        let mut errs = TypeErrors::new();
        // map bindings to types
        let mut ty_env: HashMap<String, Type> = HashMap::new();

        for decl in self.program.decls.iter() {
            if let Err(mut decl_errs) = TypeChecker::check_decl(decl, &mut ty_env) {
                errs.append(&mut decl_errs)
            }
        }

        // Return type of last expr if any. If errs, add to err list
        if let Some(last) = &self.program.last_expr {
            let res = TypeChecker::check_expr(last, &mut ty_env);
            match res {
                Ok(ty) => return Ok(ty),
                Err(mut expr_errs) => errs.append(&mut expr_errs),
            };
        }

        if errs.is_ok() {
            Ok(Type::Unit)
        } else {
            Err(errs)
        }
    }
}

impl Default for TypeErrors {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use parser::Parser;
    use parser::Type;

    use super::TypeChecker;

    fn expect_pass(inp: &str, exp_type: Type) {
        let prog = Parser::new_from_string(inp).parse().expect("Should parse");
        let ty = TypeChecker::new(&prog).type_check();
        assert_eq!(Ok(exp_type), ty)
    }

    // contains true means check if input contains exp_err. else check full equals
    fn expect_err(inp: &str, exp_err: &str, contains: bool) {
        let prog = Parser::new_from_string(inp).parse().expect("Should parse");
        let ty_err = TypeChecker::new(&prog)
            .type_check()
            .expect_err("Should err");

        if contains {
            dbg!(ty_err.to_string());
            assert!(ty_err.to_string().contains(exp_err))
        } else {
            assert_eq!(ty_err.to_string(), exp_err)
        }
    }

    #[test]
    fn test_type_check_basic() {
        // Primitives
        expect_pass("2", Type::Int);
        expect_pass("2.33", Type::Float);
        expect_pass("true", Type::Bool);

        // // Let
        expect_pass("let x : int = 2;", Type::Unit);
        expect_pass("let x : bool = false;", Type::Unit);
        expect_pass("let x : float = 3.4;", Type::Unit);

        expect_err(
            "let x : int = true;",
            "declared type int but assigned type bool",
            true,
        );
        expect_err(
            "let x : bool = 2.33;",
            "declared type bool but assigned type float",
            true,
        );
        expect_err(
            "let x : float = 20;",
            "declared type float but assigned type int",
            true,
        );

        // Multiple errors: collects them
        expect_err("let x : float = 20; let x : int = true; let x : float = 20;",
         "[TypeError]: 'x' has declared type float but assigned type int\n[TypeError]: 'x' has declared type int but assigned type bool\n[TypeError]: 'x' has declared type float but assigned type int", false);
    }

    #[test]
    fn test_type_check_sym() {
        expect_pass("let x : int = 2; x", Type::Int);
        // // variable shadowing
        expect_pass("let x : int = 2; let x : bool = true; x", Type::Bool);
        expect_pass("let x : int = 2; let y : bool = true; x;", Type::Unit);
    }

    #[test]
    fn test_type_check_unops() {
        // Negation
        expect_err("-true;", "Can't negate type bool", true);
        expect_err("let x : bool = true; -x", "Can't negate type bool", true);
        expect_pass("let x : int = 20; -x", Type::Int);
        expect_pass("let x : int = 20; -x;", Type::Unit);
        expect_pass("let x : float = 2.33; -x", Type::Float);
        expect_pass("let x : float = 2.33; -x;", Type::Unit);

        // Not
        expect_pass("let x : bool = true; !x", Type::Bool);
        expect_err("let x : int = 20; !x", "NOT to type int", true);
        expect_err("let x : float = 20.36; !x", "NOT to type float", true);
    }
}

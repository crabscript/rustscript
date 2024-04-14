use parser::{structs::*, Parser};
use std::{collections::HashMap, fmt::Display};

use parser::structs::{BlockSeq, Decl, Expr, Type};

#[derive(Debug, PartialEq)]
pub struct TypeErrors {
    pub(crate) errs: Vec<String>,
    pub(crate) cont: bool,
}

impl TypeErrors {
    pub fn new() -> TypeErrors {
        TypeErrors {
            errs: vec![],
            cont: true,
        }
    }

    pub fn new_err(err: &str) -> TypeErrors {
        TypeErrors {
            errs: vec![err.to_string()],
            cont: true,
        }
    }

    pub fn set_cont(&mut self, cont: bool) {
        self.cont = cont
    }

    pub fn add(&mut self, err: &str) {
        self.errs.push(err.to_string());
    }

    /// Move errors from the other into this one, leaving the other empty
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

type Env = HashMap<String, Type>;

pub fn new_env_with_syms(syms: Vec<String>) -> Env {
    let mut env: Env = HashMap::new();
    for sym in syms.iter() {
        env.insert(sym.to_owned(), Type::Unitialised);
    }

    env
}

/// Struct to enable type checking by encapsulating type environment.
pub struct TypeChecker<'prog> {
    program: &'prog BlockSeq,
    pub(crate) envs: Vec<Env>,
}

impl<'prog> TypeChecker<'prog> {
    pub fn new(program: &BlockSeq) -> TypeChecker<'_> {
        TypeChecker {
            program,
            envs: vec![],
        }
    }

    /// Return type of identifier by looking up nested scopes, or error if not there.
    pub(crate) fn get_type(&self, ident: &str) -> Result<Type, TypeErrors> {
        for env in self.envs.iter().rev() {
            let ty = env.get(ident);
            if let Some(ty) = ty {
                return Ok(ty.to_owned());
            }
        }

        let e = format!("Identifier '{}' not declared", ident);
        Err(TypeErrors::new_err(&e))
    }

    /// Returns type of identifier if initialised. If identifier doesn't exist or still uninit, returns Error.
    /// For use in AssignStmt e.g x = 10;
    pub(crate) fn get_type_if_init(&self, ident: &str) -> Result<Type, TypeErrors> {
        let ty = self.get_type(ident)?;
        if ty.eq(&Type::Unitialised) {
            let e = format!("Identifier '{}' assigned before declaration", ident);
            Err(TypeErrors::new_err(&e))
        } else {
            Ok(ty)
        }
    }

    /// Assign type to identifier if exists (either Unit or actual type). Else, error
    /// Only for LetStmt so we only assign in the last env (e.g x = 2; means x already declared with let)
    pub(crate) fn assign_ident(&mut self, ident: &str, ty: Type) -> Result<(), TypeErrors> {
        self.get_type(ident)?; // actually we should only check last env?
        if let Some(env) = self.envs.last_mut() {
            env.insert(ident.to_string(), ty);
        }

        Ok(())
    }

    pub(crate) fn check_unop(&mut self, op: &UnOpType, expr: &Expr) -> Result<Type, TypeErrors> {
        match op {
            UnOpType::Negate => {
                // Return err imm if operand itself is not well typed
                let ty = self.check_expr(expr)?;
                match ty {
                    Type::Int | Type::Float => Ok(ty),
                    _ => {
                        let e = format!("Can't negate type {}", ty);
                        Err(TypeErrors::new_err(&e))
                    }
                }
            }
            UnOpType::Not => {
                let ty = self.check_expr(expr)?;
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

    pub(crate) fn check_binop(
        &mut self,
        op: &BinOpType,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<Type, TypeErrors> {
        let mut ty_errs = TypeErrors::new();
        let mut l_type = self.check_expr(lhs);
        let mut r_type = self.check_expr(rhs);

        if let Err(ref mut errs) = l_type {
            ty_errs.append(errs);
        }

        if let Err(ref mut errs) = r_type {
            ty_errs.append(errs);
        }

        // have errs from lhs and/or rhs: return out, no types to check
        if !ty_errs.is_ok() {
            return Err(ty_errs);
        }

        // let x = matches!((l_type, r_type), (Ok(Type::Int), Ok(Type::Int)));
        let l_type = l_type?;
        let r_type = r_type?;

        match (l_type, r_type) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Float, Type::Float) => Ok(Type::Float),
            _ => {
                let e = format!(
                    "Can't apply '{}' to types '{}' and '{}'",
                    op, l_type, r_type
                );
                Err(TypeErrors::new_err(&e))
            }
        }
    }

    /// Return the type errors out instead of using mutable ref
    // because for nested errors in the expr we want to propagate those
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeErrors> {
        let local_errs = TypeErrors::new();
        let ty = match expr {
            Expr::Integer(_) => Type::Int,
            Expr::Float(_) => Type::Float,
            Expr::Bool(_) => Type::Bool,
            Expr::Symbol(ident) => {
                // self.ty_env.borrow().get(ident)?
                self.get_type(ident)?
            }
            Expr::UnOpExpr(op, expr) => {
                return self.check_unop(op, expr);
            }
            Expr::BinOpExpr(op, lhs, rhs) => {
                return self.check_binop(op, lhs, rhs);
            }
            Expr::BlockExpr(blk) => self.check_block(blk)?,
            Expr::IfElseExpr(if_else) => self.check_if_else(if_else)?,
        };

        if local_errs.is_ok() {
            Ok(ty)
        } else {
            Err(local_errs)
        }
    }

    /// Type check declaration and add errors if any
    pub(crate) fn check_decl(&mut self, decl: &Decl) -> Result<(), TypeErrors> {
        // dbg!("Type checking decl:", decl);
        match decl {
            Decl::LetStmt(stmt) => {
                self.check_let(stmt)?;
            }
            // Type check the expr and return any errors
            Decl::ExprStmt(expr) => {
                self.check_expr(expr)?;
            }
            // Check if sym is declared already. Then check expr matches type at decl
            Decl::AssignStmt(stmt) => {
                let sym_ty = self.get_type_if_init(&stmt.ident.to_owned())?;
                let exp_ty = self.check_expr(&stmt.expr)?;

                if !sym_ty.eq(&exp_ty) {
                    let e = format!(
                        "'{}' declared with type {} but assigned type {}",
                        stmt.ident, sym_ty, exp_ty
                    );
                    return Err(TypeErrors::new_err(&e));
                }
            }
            Decl::IfOnlyStmt(if_else) => {
                self.check_if_else(if_else)?;
            }
        }

        Ok(())
    }

    pub fn type_check(mut self) -> Result<Type, TypeErrors> {
        self.check_block(self.program)
    }
}

impl Default for TypeErrors {
    fn default() -> Self {
        Self::new()
    }
}

pub fn expect_pass(inp: &str, exp_type: Type) {
    let prog = Parser::new_from_string(inp).parse().expect("Should parse");
    let ty = TypeChecker::new(&prog).type_check();
    assert_eq!(Ok(exp_type), ty)
}

// contains true means check if input contains exp_err. else check full equals
pub fn expect_err(inp: &str, exp_err: &str, contains: bool) {
    let prog = Parser::new_from_string(inp).parse().expect("Should parse");
    dbg!(&prog);
    let ty_err = TypeChecker::new(&prog)
        .type_check()
        .expect_err("Should err");

    if contains {
        dbg!(ty_err.to_string());
        assert!(ty_err.to_string().contains(exp_err))
    } else {
        dbg!(ty_err.to_string());
        assert_eq!(ty_err.to_string(), exp_err)
    }
}

#[cfg(test)]
mod tests {
    use super::{expect_err, expect_pass};
    use parser::structs::Type;

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

    #[test]
    fn test_type_check_binops() {
        expect_pass("2+2", Type::Int);
        expect_pass("let x : int = 2; let y : int = 3; x + y", Type::Int);
        expect_pass("let x : int = 2; let y : int = 3; x + y;", Type::Unit);

        expect_pass(
            "let x : float = 2.36; let y : float = 3.2; x + y",
            Type::Float,
        );
        expect_pass(
            "let x : float = 2.36; let y : float = 3.2; x + y;",
            Type::Unit,
        );

        expect_err("let x : float = 2.36; 2 + 3*x", "apply", true);
        expect_err(
            "let x : int = 20 * 3 + 6 / 2; let y : float = 3.2; x + y",
            "apply",
            true,
        );
        expect_err("let x : bool = true +2;", "apply", true);
    }

    #[test]
    fn test_type_check_binops_collect() {
        // Collect errors from lhs/rhs
        let t = "x+y";
        expect_err(
            t,
            "[TypeError]: Identifier 'x' not declared\n[TypeError]: Identifier 'y' not declared",
            true,
        );

        // blks - can't get types from the blks since they have errs but those are collected
        let t = "{ 2+false; 3} - {3+3.5; true}";
        expect_err(t,  "[TypeError]: Can't apply '+' to types 'int' and 'bool'\n[TypeError]: Can't apply '+' to types 'int' and 'float'", true);

        let t = "x+y+z";
        expect_err(t, "[TypeError]: Identifier 'x' not declared\n[TypeError]: Identifier 'y' not declared\n[TypeError]: Identifier 'z' not declared", false);
    }
}

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

// type, must_break, must_return
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub ty: Type,
    pub must_break: bool,
    pub must_return: bool,
}

impl CheckResult {
    /// Combines two CheckResults into one CheckResult with or on the must values.
    /// Resulting has Type::Unit
    pub fn combine(res1: &CheckResult, res2: &CheckResult) -> CheckResult {
        CheckResult {
            ty: Type::Unit,
            must_break: res1.must_break || res2.must_break,
            must_return: res1.must_return || res2.must_return,
        }
    }
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
        if TypeChecker::is_builtin_fn(ident) {
            return Ok(Type::BuiltInFn);
        }

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

    /// Put param string and type into last env without checking if it's there
    // For use in fn_decl
    pub(crate) fn assign_param_types(&mut self, params: Vec<FnParam>) -> Result<(), TypeErrors> {
        let mut ty_errs = TypeErrors::new();

        for param in params.iter() {
            if let Some(env) = self.envs.last_mut() {
                match &param.type_ann {
                    Some(ty) => {
                        env.insert(param.name.clone(), ty.to_owned());
                    }
                    None => {
                        let e = format!("Parameter '{}' has no type annotation", param.name);
                        ty_errs.add(&e);
                    }
                };
            }
        }

        if !ty_errs.is_ok() {
            return Err(ty_errs);
        }

        Ok(())
    }

    pub(crate) fn check_unop(
        &mut self,
        op: &UnOpType,
        expr: &Expr,
    ) -> Result<CheckResult, TypeErrors> {
        match op {
            UnOpType::Negate => {
                // Return err imm if operand itself is not well typed
                let check_res = self.check_expr(expr)?;
                match check_res.ty {
                    Type::Int | Type::Float => {
                        let res = CheckResult {
                            ty: check_res.ty,
                            must_break: check_res.must_break,
                            must_return: check_res.must_return,
                        };

                        Ok(res)
                    }
                    _ => {
                        let e = format!("Can't negate type {}", check_res.ty);
                        Err(TypeErrors::new_err(&e))
                    }
                }
            }
            UnOpType::Not => {
                let check_res = self.check_expr(expr)?;
                match check_res.ty {
                    Type::Bool => {
                        let res = CheckResult {
                            ty: check_res.ty,
                            must_break: check_res.must_break,
                            must_return: check_res.must_return,
                        };

                        Ok(res)
                    }
                    _ => {
                        let e = format!("Can't apply logical NOT to type {}", check_res.ty);
                        Err(TypeErrors::new_err(&e))
                    }
                }
            }
        }
    }

    // Add, Sub, Mul, Div where allowed are (int, int) and (float, float)
    fn check_math_ops(
        op: &BinOpType,
        left_ty: &CheckResult,
        right_ty: &CheckResult,
    ) -> Result<CheckResult, TypeErrors> {
        match op {
            BinOpType::Add | BinOpType::Sub | BinOpType::Div | BinOpType::Mul => {
                match (&left_ty.ty, &right_ty.ty) {
                    (Type::Int, Type::Int) => {
                        let res = CheckResult {
                            ty: Type::Int,
                            must_break: left_ty.must_break || right_ty.must_break,
                            must_return: left_ty.must_return || right_ty.must_return,
                        };

                        Ok(res)
                    }
                    (Type::Float, Type::Float) => {
                        let res = CheckResult {
                            ty: Type::Float,
                            must_break: left_ty.must_break || right_ty.must_break,
                            must_return: left_ty.must_return || right_ty.must_return,
                        };

                        Ok(res)
                    }
                    _ => {
                        let e = format!(
                            "Can't apply '{}' to types '{}' and '{}'",
                            op, left_ty.ty, right_ty.ty
                        );
                        Err(TypeErrors::new_err(&e))
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn check_binop(
        &mut self,
        op: &BinOpType,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<CheckResult, TypeErrors> {
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

        let err = format!(
            "Can't apply '{}' to types '{}' and '{}'",
            op, l_type.ty, r_type.ty
        );

        let err: Result<_, TypeErrors> = Err(TypeErrors::new_err(&err));

        match op {
            BinOpType::Add | BinOpType::Sub | BinOpType::Div | BinOpType::Mul => {
                TypeChecker::check_math_ops(op, &l_type, &r_type)
            }
            // (num, num) => bool
            BinOpType::Gt | BinOpType::Lt => {
                if matches!(
                    (l_type.ty, r_type.ty),
                    (Type::Int, Type::Int) | (Type::Float, Type::Float)
                ) {
                    // Ok(Type::Bool)
                    let res = CheckResult {
                        ty: Type::Bool,
                        must_break: l_type.must_break || r_type.must_break,
                        must_return: l_type.must_return || r_type.must_return,
                    };

                    Ok(res)
                } else {
                    err
                }
            }
            // (bool, bool) => bool
            BinOpType::LogicalOr | BinOpType::LogicalAnd => {
                if matches!((l_type.ty, r_type.ty), (Type::Bool, Type::Bool)) {
                    // Ok(Type::Bool)
                    let res = CheckResult {
                        ty: Type::Bool,
                        must_break: l_type.must_break || r_type.must_break,
                        must_return: l_type.must_return || r_type.must_return,
                    };

                    Ok(res)
                } else {
                    err
                }
            }
            // (t, t) => bool
            BinOpType::LogicalEq => {
                if l_type.ty.eq(&r_type.ty) {
                    let res = CheckResult {
                        ty: Type::Bool,
                        must_break: l_type.must_break || r_type.must_break,
                        must_return: l_type.must_return || r_type.must_return,
                    };

                    Ok(res)
                } else {
                    err
                }
            }
        }
    }

    /// Return the type errors out instead of using mutable ref
    // because for nested errors in the expr we want to propagate those
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Result<CheckResult, TypeErrors> {
        let local_errs = TypeErrors::new();
        let ty: CheckResult = match expr {
            Expr::Integer(_) => CheckResult {
                ty: Type::Int,
                must_break: false,
                must_return: false,
            },
            Expr::Float(_) => CheckResult {
                ty: Type::Float,
                must_break: false,
                must_return: false,
            },
            Expr::Bool(_) => CheckResult {
                ty: Type::Bool,
                must_break: false,
                must_return: false,
            },
            Expr::Symbol(ident) => {
                // self.ty_env.borrow().get(ident)?
                let sym_ty = self.get_type(ident)?;

                CheckResult {
                    ty: sym_ty,
                    must_break: false,
                    must_return: false,
                }
            }
            Expr::UnOpExpr(op, expr) => {
                return self.check_unop(op, expr);
            }
            Expr::BinOpExpr(op, lhs, rhs) => {
                return self.check_binop(op, lhs, rhs);
            }
            Expr::BlockExpr(blk) => return self.check_block(blk, vec![]),
            Expr::IfElseExpr(if_else) => return self.check_if_else(if_else),
            Expr::FnCallExpr(fn_call) => return self.check_fn_call(fn_call),
            Expr::SpawnExpr(fn_call) => {
                self.check_fn_call(fn_call)?;
                CheckResult {
                    ty: Type::ThreadId,
                    must_break: false,
                    must_return: false,
                }
            }
            // TODO: return join type based on function that was called
            // Need to track spawn / join calls at compile time
            Expr::JoinExpr(_) => CheckResult {
                ty: Type::Unit,
                must_break: false,
                must_return: false,
            },
        };

        if local_errs.is_ok() {
            Ok(ty)
        } else {
            Err(local_errs)
        }
    }

    /// Type check declaration and add errors if any
    pub(crate) fn check_decl(&mut self, decl: &Decl) -> Result<CheckResult, TypeErrors> {
        // dbg!("Type checking decl:", decl);
        match decl {
            Decl::LetStmt(stmt) => self.check_let(stmt),
            // Type check the expr and return any errors
            Decl::ExprStmt(expr) => self.check_expr(expr),
            // Check if sym is declared already. Then check expr matches type at decl
            Decl::AssignStmt(stmt) => {
                let sym_ty = self.get_type_if_init(&stmt.ident.to_owned())?;
                let exp_ty = self.check_expr(&stmt.expr)?;

                if !sym_ty.eq(&exp_ty.ty) {
                    let e = format!(
                        "'{}' declared with type {} but assigned type {}",
                        stmt.ident, sym_ty, exp_ty.ty
                    );
                    return Err(TypeErrors::new_err(&e));
                }

                let res = CheckResult {
                    ty: Type::Unit,
                    must_break: exp_ty.must_break,
                    must_return: exp_ty.must_return,
                };

                Ok(res)
            }
            Decl::IfOnlyStmt(if_else) => self.check_if_else(if_else),
            Decl::LoopStmt(lp) => self.check_loop(lp),
            Decl::BreakStmt => {
                // must_break base case
                Ok(CheckResult {
                    ty: Type::Unit,
                    must_break: true,
                    must_return: false,
                })
            }
            Decl::FnDeclStmt(fn_decl) => self.check_fn_decl(fn_decl),
            // TODO: check nested returns with fn stack
            Decl::ReturnStmt(_) => Ok(CheckResult {
                ty: Type::Unit,
                must_break: true,
                must_return: true,
            }),
            Decl::WaitStmt(_) => Ok(CheckResult {
                ty: Type::Unit,
                must_break: false,
                must_return: false,
            }),
            Decl::PostStmt(_) => Ok(CheckResult {
                ty: Type::Unit,
                must_break: false,
                must_return: false,
            }),
        }

        // Ok(())
    }

    pub fn type_check(mut self) -> Result<Type, TypeErrors> {
        let ty = self.check_block(self.program, vec![])?;
        // dbg!(&ty);
        Ok(ty.ty)
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
    dbg!(&ty);
    assert_eq!(Ok(exp_type), ty)
}

/// To expect type str
pub fn expect_pass_str(inp: &str, exp_type_str: &str) {
    let prog = Parser::new_from_string(inp).parse().expect("Should parse");
    let ty = TypeChecker::new(&prog)
        .type_check()
        .expect("Type check should pass");
    dbg!(&ty);
    assert_eq!(ty.to_string(), exp_type_str)
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
    fn test_type_check_binops_math() {
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

    #[test]
    fn test_type_check_binops_logical() {
        // &&, ||
        expect_pass("true && false", Type::Bool);
        expect_err(
            "2 && false",
            "Can't apply '&&' to types 'int' and 'bool'",
            true,
        );
        expect_err(
            "false && 2",
            "Can't apply '&&' to types 'bool' and 'int'",
            true,
        );

        expect_pass("true || false", Type::Bool);
        expect_err(
            "2 || false",
            "Can't apply '||' to types 'int' and 'bool'",
            true,
        );

        // chaining ok
        expect_pass("let x = true; true && false && x", Type::Bool);
        expect_pass("let x = true; true && false || x", Type::Bool);
    }

    #[test]
    fn test_type_check_binops_cmp() {
        // ==, >, <

        // eq
        expect_pass("true == false", Type::Bool);
        expect_pass("23 == 56", Type::Bool);
        expect_pass("23.5 == 56.2", Type::Bool);
        expect_err(
            "true == {2;3 }",
            "Can't apply '==' to types 'bool' and 'int'",
            true,
        );

        // >
        expect_pass("2 > 3", Type::Bool);
        expect_pass("2.5 > 3.2", Type::Bool);
        expect_err(
            "true > false",
            "Can't apply '>' to types 'bool' and 'bool'",
            true,
        );

        // <
        expect_pass("2 < 3", Type::Bool);
        expect_pass("2.5 < 3.2", Type::Bool);
        expect_err(
            "true < false",
            "Can't apply '<' to types 'bool' and 'bool'",
            true,
        );

        // mix
        expect_pass("false == (3 > 5)", Type::Bool);
        expect_err(
            "(5 == 3) < 5",
            "[TypeError]: Can't apply '<' to types 'bool' and 'int'",
            false,
        );
    }

    #[test]
    fn test_type_check_binops_log_cmp() {
        // mix ==, >, <, &&, ||
        expect_pass(r"2 == 2 && !true == false || 1 > 3 && 2 < 5 ", Type::Bool);

        expect_pass(
            r"
        let x : int = 2;
        let y : int = 3;
        let z : bool = 2 == 3;
        x == y && !z || y > x && x+2 < 5
        ",
            Type::Bool,
        );

        expect_err(
            r"
        let x : int = 2;
        let y : int = 3;
        let z : bool = 2 == 3;
        x == y && !z || y > x && z < 5
        ",
            "Can't apply '<' to types 'bool' and 'int'",
            true,
        );
    }
}

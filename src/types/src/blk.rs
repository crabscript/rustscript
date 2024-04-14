use crate::type_checker::{new_env_with_syms, TypeChecker, TypeErrors};
use parser::structs::{BlockSeq, Type};

impl<'prog> TypeChecker<'prog> {
    pub(crate) fn check_block(&mut self, program: &BlockSeq) -> Result<Type, TypeErrors> {
        let mut errs = TypeErrors::new();
        // map bindings to types
        // let mut ty_env: HashMap<String, Type> = HashMap::new();
        // let mut ty_env = TyEnv::new();
        let env = new_env_with_syms(program.symbols.clone());
        self.envs.push(env);

        for decl in program.decls.iter() {
            if let Err(mut decl_errs) = self.check_decl(decl) {
                errs.append(&mut decl_errs);

                // if this err means we can't proceed, stop e.g let x = -true; let y = x + 3; - we don't know type of x since invalid
                if !decl_errs.cont {
                    break;
                }
            }
        }

        // return errors for decls first if any, without checking expr
        // because expr may be dependent
        if !errs.is_ok() {
            self.envs.pop();
            return Err(errs);
        }

        // Return type of last expr if any. If errs, add to err list
        if let Some(last) = &program.last_expr {
            let res = self.check_expr(last);
            match res {
                Ok(ty) => {
                    self.envs.pop();
                    return Ok(ty);
                }
                Err(mut expr_errs) => errs.append(&mut expr_errs),
            };
        }

        self.envs.pop();

        if errs.is_ok() {
            Ok(Type::Unit)
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use parser::structs::Type;

    use crate::type_checker::{expect_err, expect_pass};

    #[test]
    fn test_type_check_blk_simple() {
        let t = "{ 2 }";
        expect_pass(t, Type::Int);

        let t = "{ 2; true }";
        expect_pass(t, Type::Bool);

        let t = "{ let x : float = 2.4; x }";
        expect_pass(t, Type::Float);

        let t = "{ let x = 2.4; x; }";
        expect_pass(t, Type::Unit);

        let t = "let y = { let x = true; x }; y";
        expect_pass(t, Type::Bool);

        let t = "let y : int = { let x = true; x }; y";
        expect_err(t, "has declared type int but assigned type bool", true);
    }

    #[test]
    fn test_type_check_blk_scope() {
        let t = r"
        let x : int = 2;
        {
            let y : int = 3;
        }
        y
        ";
        expect_err(t, "'y' not declared", true);

        let t = r"
        let x : int = 2;
        {
            let x : bool = true;
            let y : bool = x;
        }
        x
        ";
        expect_pass(t, Type::Int);

        let t = r"
        let x : int = 2;
        {
            let x : bool = true;
            let y : int = x;
        }
        x
        ";
        expect_err(t, "has declared type int but assigned type bool", true);

        let t = r"
        let x : int = 2;
        let z = {
            let x : bool = true;
            let y : bool = x;
            y
        };
        x;
        z
        ";
        expect_pass(t, Type::Bool);
    }

    #[test]
    fn test_type_check_blk_more() {
        let t = r"
        let x = 2; 
        let y = 0; 
        { 
            let x = 3; 
            y = 4; 
        } 
        x+y
        ";

        expect_pass(t, Type::Int);

        // gets type from parent scope correctly during assign
        let t = r"
        let x = 2; 
        let y : bool = true;
        { 
            let x = 3; 
            y = 4; 
        } 
        x+y
        ";

        expect_err(t, "'y' declared with type bool but assigned type int", true);

        // doesn't matter that shadowed var has diff type
        let t = r"
        let x : int = 20; 
        let y = 0; 
        let z : bool = { 
            let x : bool = true; 
            y = 4; 
            x
        };
        x+y
        ";
        expect_pass(t, Type::Int);

        // blk with no last expr has unit type
        let t = r"
        let x : int = 20; 
        let y = 0; 
        let z : bool = { 
            let x : bool = true; 
            y = 4; 
            x;
        };
        x+y
        ";
        expect_err(t, "'z' has declared type bool but assigned type ()", true);

        // x declared again in block, so the assign looks for closest decl of x which is Uninit at time of x = true;
        let t = r"
        let x = 10;
        {
            x = true;
            let x = false;
        };
        ";

        expect_err(
            t,
            "[TypeError]: Identifier 'x' assigned before declaration",
            false,
        );
    }

    #[test]
    fn test_type_check_blk_errs() {
        let t = r"
        let x : int = true;
        let y = 20;
        {
            let y : bool = 20;
        }
        x + false
        ";

        expect_err(t, "[TypeError]: 'x' has declared type int but assigned type bool\n[TypeError]: 'y' has declared type bool but assigned type int", true);
    }
}

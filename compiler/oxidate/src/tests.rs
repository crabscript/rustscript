#[cfg(test)]
mod tests {

    use std::vec;

    use bytecode::ByteCode;
    use bytecode::ByteCode::*;
    use bytecode::Value::*;
    use parser::Parser;

    use crate::compiler::Compiler;

    fn exp_compile_str(inp: &str) -> Vec<ByteCode> {
        let parser = Parser::new_from_string(inp);
        let parsed = parser.parse().expect("Should parse");
        dbg!("parsed:", &parsed);
        let comp = Compiler::new(parsed);
        comp.compile().expect("Should compile")
    }

    fn test_comp(inp: &str, exp: Vec<ByteCode>) {
        let res = exp_compile_str(inp);
        dbg!(&res);
        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_simple() {
        let res = exp_compile_str("42;");
        assert_eq!(res, vec![ByteCode::ldc(42), POP, DONE]);

        let res = exp_compile_str("42; 45; 30");
        assert_eq!(
            res,
            vec![
                ByteCode::ldc(42),
                POP,
                ByteCode::ldc(45),
                POP,
                ByteCode::ldc(30),
                DONE
            ]
        );

        let res = exp_compile_str("42; true; 2.36;");
        assert_eq!(
            res,
            vec![
                ByteCode::ldc(42),
                POP,
                ByteCode::ldc(true),
                POP,
                ByteCode::ldc(2.36),
                POP,
                DONE
            ]
        )
    }

    #[test]
    fn test_compile_binop() {
        let res = exp_compile_str("2+3*2-4;");
        let exp = vec![
            LDC(Int(2)),
            LDC(Int(3)),
            LDC(Int(2)),
            BINOP(bytecode::BinOp::Mul),
            BINOP(bytecode::BinOp::Add),
            LDC(Int(4)),
            BINOP(bytecode::BinOp::Sub),
            POP,
            DONE,
        ];

        assert_eq!(res, exp);

        let res = exp_compile_str("2+3*4-5/5");

        let exp = [
            LDC(Int(2)),
            LDC(Int(3)),
            LDC(Int(4)),
            BINOP(bytecode::BinOp::Mul),
            BINOP(bytecode::BinOp::Add),
            LDC(Int(5)),
            LDC(Int(5)),
            BINOP(bytecode::BinOp::Div),
            BINOP(bytecode::BinOp::Sub),
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_binop_cmp() {
        // >, <, ==
        test_comp(
            "2+2 < 3",
            vec![
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                LDC(Int(3)),
                ByteCode::binop("<"),
                DONE,
            ],
        );

        // >
        test_comp(
            "2+2 > 3",
            vec![
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                LDC(Int(3)),
                ByteCode::binop(">"),
                DONE,
            ],
        );

        // ==
        test_comp(
            "2+2 == 3",
            vec![
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                LDC(Int(3)),
                ByteCode::binop("=="),
                DONE,
            ],
        );

        // mix
        let exp = vec![
            LDC(Int(4)),
            LDC(Int(6)),
            ByteCode::binop("<"),
            LDC(Bool(false)),
            LDC(Int(3)),
            LDC(Int(3)),
            ByteCode::binop(">"),
            ByteCode::binop("=="),
            ByteCode::binop("=="),
            DONE,
        ];
        test_comp("(4 < 6) == (false == (3 > 3))", exp);
    }

    #[test]
    fn test_compile_let() {
        let res = exp_compile_str("let x = 2;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);

        // stmt last
        let res = exp_compile_str("let x = 2; let y = 3; ");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string(), "y".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(3)),
            ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);

        // many
        let res = exp_compile_str("let x = 2; let y = 3; 40");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string(), "y".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(3)),
            ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(40)),
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_sym() {
        let res = exp_compile_str("let x = 2; -x+2;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LD("x".to_string()),
            UNOP(bytecode::UnOp::Neg),
            LDC(Int(2)),
            BINOP(bytecode::BinOp::Add),
            POP,
            EXITSCOPE,
            DONE,
        ];
        assert_eq!(res, exp);

        let res = exp_compile_str("let x = 2; let y = x; x*5+2");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string(), "y".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LD("x".to_string()),
            ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LD("x".to_string()),
            LDC(Int(5)),
            BINOP(bytecode::BinOp::Mul),
            LDC(Int(2)),
            BINOP(bytecode::BinOp::Add),
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_not() {
        let res = exp_compile_str("!true");
        let exp = [LDC(Bool(true)), UNOP(bytecode::UnOp::Not), DONE];
        assert_eq!(res, exp);

        let res = exp_compile_str("!!false");
        let exp = [
            LDC(Bool(false)),
            UNOP(bytecode::UnOp::Not),
            UNOP(bytecode::UnOp::Not),
            DONE,
        ];
        assert_eq!(res, exp);

        let res = exp_compile_str("!!!true;");
        let exp = [
            LDC(Bool(true)),
            UNOP(bytecode::UnOp::Not),
            UNOP(bytecode::UnOp::Not),
            UNOP(bytecode::UnOp::Not),
            POP,
            DONE,
        ];
        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_assign() {
        let res = exp_compile_str("let x = 2; x = 3;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(3)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];
        assert_eq!(res, exp);

        // diff types
        let res = exp_compile_str("let x = 2; x = true;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Bool(true)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];
        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_blk_simple() {
        let t = "{ 2 }";
        let exp = vec![ByteCode::ldc(2), DONE];
        test_comp(t, exp);

        let t = "{ 2; 3 }";
        let exp = vec![ByteCode::ldc(2), ByteCode::POP, ByteCode::ldc(3), DONE];
        test_comp(t, exp);

        let t = "{ 2; 3; }";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            LDC(Unit),
            DONE,
        ];
        test_comp(t, exp);

        let t = "{ 2; 3; 4 }";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            ByteCode::ldc(4),
            DONE,
        ];
        test_comp(t, exp);

        // // like doing just 4;
        let t = "{ 2; 3; 4 };";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            ByteCode::ldc(4),
            ByteCode::POP,
            DONE,
        ];
        test_comp(t, exp);

        let t = "{ 2; 3; 4; };";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            ByteCode::ldc(4),
            ByteCode::POP,
            ByteCode::ldc(Unit),
            ByteCode::POP,
            DONE,
        ];
        test_comp(t, exp);
    }

    #[test]
    fn test_compile_blk_cases() {
        test_comp("{ 2 }", vec![ByteCode::ldc(2), DONE]);
        // blk with no last expr or none_like returns Unit
        test_comp("{ 2; }", vec![ByteCode::ldc(2), POP, LDC(Unit), DONE]);

        // // since we pop after every stmt, if the block ends in expr we just rely on that
        test_comp("{ 2 };", vec![ByteCode::ldc(2), POP, DONE]);

        // // we pop after every stmt, but since this blk has no last expr we push unit before blk ends so the pop doesn't
        test_comp(
            "{ 2; };",
            vec![ByteCode::ldc(2), POP, ByteCode::ldc(Unit), POP, DONE],
        );

        // nested
        test_comp(
            r"
        {
            2;
            {
                {

                }
            }
        }
        ",
            vec![LDC(Int(2)), POP, LDC(Unit), DONE],
        );

        // nested
        test_comp(
            r"
        {
            2;
            {
                {

                }
            }
        };
        ",
            vec![LDC(Int(2)), POP, LDC(Unit), POP, DONE],
        );

        // nested with stmt inside
        test_comp(
            r"
        {
            2;
            {
                { 
                    {

                    };
                }
            }
        }
        ",
            vec![LDC(Int(2)), POP, LDC(Unit), POP, LDC(Unit), DONE],
        );
    }

    #[test]
    fn test_compile_blk_let() {
        // empty blk
        let t = r"
        let x = {
            {}
        };
        ";

        // last LDC Unit if from compiling let. last POP is from automatic pop after decl
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                LDC(Unit),
                ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );

        let t = r"
        let x = 2;
        {
            let y = 3;
            x+y
        }
        ";
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                ByteCode::ldc(2),
                ASSIGN("x".to_string()),
                ByteCode::ldc(Unit),
                POP,
                ENTERSCOPE(vec!["y".to_string()]),
                LDC(Int(3)),
                ASSIGN("y".to_string()),
                LDC(Unit),
                POP,
                LD("x".to_string()),
                LD("y".to_string()),
                ByteCode::binop("+"),
                EXITSCOPE,
                EXITSCOPE,
                DONE,
            ],
        );

        let t = r"
        let x = 2; { {2+2;} };
        ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                ByteCode::ldc(2),
                ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                POP,
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );

        // nested none-like
        let t = r"
        let x = 2; { 

            {
                {
                    2+2;
                }
            } 
        
        };
        ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                ByteCode::ldc(2),
                ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                POP,
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_if_only() {
        // if only with nothing after
        let t = r"
        if !true {
            2
        }
        200
        ";

        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                ByteCode::unop("!"),
                JOF(5),
                LDC(Int(2)),
                GOTO(6),
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // ifonly-blk has value
        let t = r"
        if !true {
            2
        }
        200
        ";

        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                ByteCode::unop("!"),
                JOF(5),
                LDC(Int(2)),
                GOTO(6),
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // if only-blk none like
        let t = r"
        if true {
            2;
            3;
        }
        200
        ";

        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                JOF(8),
                LDC(Int(2)),
                POP,
                LDC(Int(3)),
                POP,
                LDC(Unit),
                GOTO(9),
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // consec
        let t = r"
        let y = true;
        if false {
           2; 3 
        }

        if y {  
            y = false;
        }

        y
        ";

        let exp = vec![
            ENTERSCOPE(vec!["y".to_string()]),
            LDC(Bool(true)),
            ByteCode::ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LDC(Bool(false)),
            JOF(11),
            LDC(Int(2)),
            POP,
            LDC(Int(3)),
            GOTO(12),
            LDC(Unit),
            POP,
            ByteCode::ld("y"),
            JOF(21),
            LDC(Bool(false)),
            ByteCode::ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LDC(Unit),
            GOTO(22),
            LDC(Unit),
            POP,
            ByteCode::ld("y"),
            EXITSCOPE,
            DONE,
        ];

        test_comp(t, exp);
    }

    #[test]
    fn test_compile_if_else() {
        // ifelse as stmt, blks return val
        let t = r"
        if true {
            2
        } else {
            3
        }
        200
        ";
        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                JOF(4),
                LDC(Int(2)),
                GOTO(5),
                LDC(Int(3)),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // ifelse as stmt, blks return unit
        let t = r"
         if true {
             2;
             true;
         } else {
             3;
             false;
         }
         200
         ";
        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                JOF(8),
                LDC(Int(2)),
                POP,
                LDC(Bool(true)),
                POP,
                LDC(Unit),
                GOTO(13),
                LDC(Int(3)),
                POP,
                LDC(Bool(false)),
                POP,
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // ifelse as expr, blks return val
        let t = r"
         let y = true;
         let x = if y {
            2;
            true
        } else {
            3;
            false
        };

        x
         ";
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["y".to_string(), "x".to_string()]),
                LDC(Bool(true)),
                ByteCode::ASSIGN("y".to_string()),
                LDC(Unit),
                POP,
                ByteCode::ld("y".to_string()),
                JOF(11),
                LDC(Int(2)),
                POP,
                LDC(Bool(true)),
                GOTO(14),
                LDC(Int(3)),
                POP,
                LDC(Bool(false)),
                ByteCode::ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                ByteCode::ld("x".to_string()),
                EXITSCOPE,
                DONE,
            ],
        );

        // if-else expr, blks return unit
        let t = r"
         let x = if true {
            2;
        } else {
            3;
        };

        x
         ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                LDC(Bool(true)),
                JOF(7),
                LDC(Int(2)),
                POP,
                LDC(Unit),
                GOTO(10),
                LDC(Int(3)),
                POP,
                LDC(Unit),
                ByteCode::assign("x".to_string()),
                LDC(Unit),
                POP,
                ByteCode::ld("x".to_string()),
                EXITSCOPE,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_logical_ops() {
        // &&
        test_comp(
            "true && false",
            vec![
                LDC(Bool(true)),
                JOF(4),
                LDC(Bool(false)),
                GOTO(5),
                LDC(Bool(false)),
                DONE,
            ],
        );
        test_comp(
            "true && false && true",
            vec![
                LDC(Bool(true)),
                JOF(4),
                LDC(Bool(false)),
                GOTO(5),
                LDC(Bool(false)),
                JOF(8),
                LDC(Bool(true)),
                GOTO(9),
                LDC(Bool(false)),
                DONE,
            ],
        );
        test_comp(
            "2 < 3 && true",
            vec![
                LDC(Int(2)),
                LDC(Int(3)),
                BINOP(bytecode::BinOp::Lt),
                JOF(6),
                LDC(Bool(true)),
                GOTO(7),
                LDC(Bool(false)),
                DONE,
            ],
        );

        // ||
        test_comp(
            "true || false",
            vec![
                LDC(Bool(true)),
                JOF(4),
                LDC(Bool(true)),
                GOTO(5),
                LDC(Bool(false)),
                DONE,
            ],
        );
        test_comp(
            "true || false || false",
            vec![
                LDC(Bool(true)),
                JOF(4),
                LDC(Bool(true)),
                GOTO(5),
                LDC(Bool(false)),
                JOF(8),
                LDC(Bool(true)),
                GOTO(9),
                LDC(Bool(false)),
                DONE,
            ],
        );

        // mix
        test_comp(
            "true || false && false",
            vec![
                LDC(Bool(true)),
                JOF(4),
                LDC(Bool(true)),
                GOTO(9),
                LDC(Bool(false)),
                JOF(8),
                LDC(Bool(false)),
                GOTO(9),
                LDC(Bool(false)),
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_loop() {
        // inf loop
        let t = r"
        200;
        loop {
            2;
        }
        ";
        test_comp(
            t,
            vec![
                LDC(Int(200)),
                POP,
                LDC(Int(2)),
                POP,
                LDC(Unit),
                POP,
                GOTO(2),
                LDC(Unit),
                POP,
                DONE,
            ],
        );

        // with break, no cond
        let t = r"
        200;

        loop {
            2;
            break;
        }

        300;
        ";
        test_comp(
            t,
            vec![
                LDC(Int(200)),
                POP,
                LDC(Int(2)),
                POP,
                GOTO(9),
                POP,
                LDC(Unit),
                POP,
                GOTO(2),
                LDC(Unit),
                POP,
                LDC(Int(300)),
                POP,
                DONE,
            ],
        );

        // with cond, no break

        let t = r"
        let x = 0;
        loop x < 3 {
            x = x + 1;
        }
        x
        ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                LDC(Int(0)),
                ByteCode::assign("x"),
                LDC(Unit),
                POP,
                ByteCode::ld("x"), // 5 - loop cond (start)
                LDC(Int(3)),
                ByteCode::binop("<"),
                JOF(18),
                ByteCode::ld("x"),
                LDC(Int(1)),
                ByteCode::binop("+"),
                ByteCode::assign("x"),
                LDC(Unit),
                POP,
                LDC(Unit),
                POP,
                GOTO(5),
                LDC(Unit), // 18 - loop end (load unit as value)
                POP,
                ByteCode::ld("x"),
                EXITSCOPE,
                DONE,
            ],
        );

        // cond and break
        let t = r"
        let x = 0;
        loop x < 3 {
            x = x + 1;
            
            if x == 2 {
                break;
            }
        }
        x
        ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                LDC(Int(0)),
                ByteCode::assign("x"),
                LDC(Unit),
                POP,
                LD("x".to_string()),
                LDC(Int(3)),
                ByteCode::binop("<"),
                JOF(28),
                LD("x".to_string()),
                LDC(Int(1)),
                ByteCode::binop("+"),
                ByteCode::assign("x"),
                LDC(Unit),
                POP,
                LD("x".to_string()),
                LDC(Int(2)),
                ByteCode::binop("=="),
                JOF(23),
                GOTO(28),
                POP,
                LDC(Unit),
                GOTO(24),
                LDC(Unit),
                POP,
                LDC(Unit),
                POP,
                GOTO(5),
                LDC(Unit),
                POP,
                LD("x".to_string()),
                EXITSCOPE,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_fn_call() {
        let t = "print(2, 3)";
        test_comp(
            t,
            vec![
                ByteCode::ld("print"),
                LDC(Int(2)),
                LDC(Int(3)),
                CALL(2),
                LDC(Unit),
                DONE,
            ],
        );

        let t = "print(2, 3);";
        test_comp(
            t,
            vec![
                ByteCode::ld("print"),
                LDC(Int(2)),
                LDC(Int(3)),
                CALL(2),
                LDC(Unit),
                POP,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_fn_decl() {
        let t = r"
        300;
        fn f() {
            2
        }
        ";
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["f".to_string()]),
                ByteCode::ldc(300),
                POP,
                LDF(5, vec![]),
                GOTO(7),
                ByteCode::ldc(2),
                RESET(bytecode::FrameType::CallFrame),
                ByteCode::assign("f"),
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );

        // explicit return - doesn't skip rest of block yet
        let t = r"
        fn f() {
            return 2;
        }
        ";
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["f".to_string()]),
                LDF(3, vec![]),
                GOTO(8),
                ByteCode::ldc(2),
                RESET(bytecode::FrameType::CallFrame),
                POP,
                LDC(Unit),
                RESET(bytecode::FrameType::CallFrame),
                ByteCode::assign("f"),
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_fn_decl_more() {
        // fn with params
        let t = r"
        fn fac(n: int) {
            2 + n
        }
        ";
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["fac".to_string()]),
                LDF(3, vec!["n".to_string()]),
                GOTO(7),
                ByteCode::ldc(2),
                ByteCode::ld("n"),
                ByteCode::binop("+"),
                RESET(bytecode::FrameType::CallFrame),
                ByteCode::assign("fac"),
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_spawn() {
        let t = r"
        2;
        spawn func(1);
        3;
        ";
        test_comp(
            t,
            vec![
                ByteCode::ldc(2),
                POP,
                SPAWN(4),
                GOTO(9),
                POP,
                LD("func".to_string()),
                ByteCode::ldc(1),
                CALL(1),
                DONE,
                POP,
                ByteCode::ldc(3),
                POP,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_wait_post() {
        let t = r"
        wait sem;
        2;
        post sem;
        ";
        // test_comp(t, vec![]);
    }
}

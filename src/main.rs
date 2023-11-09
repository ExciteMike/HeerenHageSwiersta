#![warn(clippy::pedantic)]

use std::collections::HashSet;

use heeren_hage_swierstra::{fresh_type_id, infer_type, Ast, Environment, Type};

/// create the AST to test with
fn mk_ast() -> Ast {
    // fn f0 x =
    //   fn f1 x =
    //     x = x + 1
    //     x = x + (len "test")
    //   f1 x
    // print "test"
    // print (f0 10)
    let f1 = Ast::Fn {
        fn_name: "f1",
        parameter: "x",
        body: Ast::Add(
            Ast::Id("x").into(),
            Ast::Add(
                Ast::LiteralInt(1).into(),
                Ast::App {
                    e1: Ast::Id("len").into(),
                    e2: Ast::LiteralStr("test").into(),
                }
                .into(),
            )
            .into(),
        )
        .into(),
    };
    let f0: Ast = Ast::Fn {
        fn_name: "f0",
        parameter: "x",
        body: Ast::Do(vec![
            f1,
            Ast::App {
                e1: Ast::Id("f1").into(),
                e2: Ast::Id("x").into(),
            },
        ])
        .into(),
    };
    Ast::Do(vec![
        f0,
        Ast::App {
            e1: Ast::Id("print").into(),
            e2: Ast::LiteralStr("test").into(),
        },
        Ast::App {
            e1: Ast::Id("print").into(),
            e2: Ast::App {
                e1: Ast::Id("f0").into(),
                e2: Ast::LiteralInt(10).into(),
            }
            .into(),
        },
    ])
}

/// create an environment to test with
fn mk_env() -> Environment {
    let print_ty_id = fresh_type_id();
    [
        (
            "len",
            (HashSet::new(), Type::F(Type::Str.into(), Type::Int.into())),
        ),
        (
            "print",
            (
                HashSet::from([print_ty_id]),
                Type::F(Type::Unknown(print_ty_id).into(), Type::Nothing.into()),
            ),
        ),
    ]
    .into()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = mk_env();
    let ast = mk_ast();
    let ir = ast.desugar();
    let (_, typed_tree) = infer_type(&env, ir);
    println!("{}", typed_tree.to_string()?);
    Ok(())
}

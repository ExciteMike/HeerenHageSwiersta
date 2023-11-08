#![warn(clippy::pedantic)]

use heeren_hage_swierstra::{infer_type, Ast};

fn dump(ast: Ast) -> Result<(), Box<dyn std::error::Error>> {
    let (_, tree) = infer_type(ast.desugar());
    println!("{}", tree.to_string()?);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // fn f0 x =
    //   fn f1 x =
    //     x = x + 1
    //     x = x + (len "test")
    //   f1 x
    // f0 10
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
    let expr = Ast::Do(vec![
        f0,
        Ast::App {
            e1: Ast::Id("f0").into(),
            e2: Ast::LiteralInt(10).into(),
        },
    ]);

    if false {
        dump(Ast::App {
            e1: Ast::Id("len").into(),
            e2: Ast::LiteralStr("abc").into(),
        })
    } else {
        dump(expr)
    }
}

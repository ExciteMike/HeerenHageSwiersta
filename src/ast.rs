use crate::Ir;

#[derive(Debug, Clone)]
pub enum Ast {
    LiteralStr(&'static str),
    LiteralInt(i64),
    Id(&'static str),
    App {
        e1: Box<Ast>,
        e2: Box<Ast>,
    },
    Lam {
        binding: &'static str,
        body: Box<Ast>,
    },
    Let {
        e1: Box<Ast>,
        binding: &'static str,
    },
    Fn {
        fn_name: &'static str,
        parameter: &'static str,
        body: Box<Ast>,
    },
    Add(Box<Ast>, Box<Ast>),
    Do(Vec<Ast>),
}

impl Ast {
    pub fn display_tree(&self) {
        self.display_tree_("");
    }
    fn display_tree_(&self, prefix: &str) {
        use Ast::*;
        match self {
            LiteralStr(s) => println!("{prefix}+-\"{s}\""),
            LiteralInt(i) => println!("{prefix}+-{i}"),
            Id(s) => println!("{prefix}+-ID `{s}`"),
            App { e1, e2 } => {
                println!("{prefix}+-App");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Lam { binding, body } => {
                println!("{prefix}+-λ {binding} → ⋯");
                println!("{prefix}  |");
                body.display_tree_(&format!("{prefix}  "));
            }
            Let { e1, binding } => {
                println!("{prefix}+-Let {binding} = ⋯");
                println!("{prefix}  |");
                e1.display_tree_(&format!("{prefix}  "));
            }
            Add(e1, e2) => {
                println!("{prefix}+-Add");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Fn {
                fn_name,
                parameter,
                body,
            } => {
                println!("{prefix}+-Fn {fn_name} {parameter} = ⋯");
                println!("{prefix}  |");
                body.display_tree_(&format!("{prefix}  "));
            }
            Do(vec) => {
                println!("{prefix}+-Do");
                for expr in vec {
                    println!("{prefix}  |");
                    expr.display_tree_(&format!("{prefix}  |"));
                }
            }
        }
    }

    #[must_use]
    pub fn desugar(self) -> Ir {
        use Ast::*;
        match self {
            LiteralInt(x) => Ir::LiteralInt(x),
            LiteralStr(x) => Ir::LiteralStr(x),
            Id(x) => Ir::Id(x),
            App { e1, e2 } => Ir::App {
                e1: (*e1).desugar().into(),
                e2: (*e2).desugar().into(),
            },
            Lam { binding, body } => Ir::Lam {
                binding,
                body: (*body).desugar().into(),
            },
            Let { e1, binding } => Ir::Let {
                e1: (*e1).desugar().into(),
                binding,
                e2: Ir::Nop.into(),
            },
            Add(e1, e2) => Ir::Add((*e1).desugar().into(), (*e2).desugar().into()),
            Fn {
                fn_name,
                parameter,
                body,
            } => Ir::Let {
                e1: Ir::Lam {
                    binding: parameter,
                    body: (*body).desugar().into(),
                }
                .into(),
                binding: fn_name,
                e2: Ir::Nop.into(),
            },
            Do(vec) => desugar_statements(vec.into_iter()),
        }
    }
}

/// helper for `Do` case of `Ast::desugar`
fn desugar_statements<I>(stmts: I) -> Ir
where
    I: Iterator<Item = Ast> + DoubleEndedIterator,
{
    stmts.rfold(Ir::Nop, |init, ast| match ast {
        Ast::Let { e1, binding } => Ir::Let {
            e1: e1.desugar().into(),
            binding,
            e2: init.into(),
        },
        Ast::Fn {
            fn_name,
            parameter,
            body,
        } => Ir::Let {
            e1: Ir::Lam {
                binding: parameter,
                body: (*body).desugar().into(),
            }
            .into(),
            binding: fn_name,
            e2: init.into(),
        },
        _ => {
            let ir = ast.desugar();
            if matches!(init, Ir::Nop) {
                ir
            } else {
                Ir::Seq(ir.into(), init.into())
            }
        }
    })
}

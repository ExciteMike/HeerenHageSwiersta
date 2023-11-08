#[derive(Debug)]
pub enum Ir {
    Nop,
    LiteralStr(&'static str),
    LiteralInt(i64),
    Id(&'static str),
    App {
        e1: Box<Ir>,
        e2: Box<Ir>,
    },
    Lam {
        binding: &'static str,
        body: Box<Ir>,
    },
    Let {
        e1: Box<Ir>,
        binding: &'static str,
        e2: Box<Ir>,
    },
    Add(Box<Ir>, Box<Ir>),
    Seq(Box<Ir>, Box<Ir>),
}

impl Ir {
    pub fn display_tree(&self) {
        self.display_tree_("");
    }
    fn display_tree_(&self, prefix: &str) {
        use Ir::*;
        match self {
            Nop => println!("{prefix}+-NOP"),
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
                println!("{prefix}+-Lambda {binding} -> ...");
                println!("{prefix}  |");
                body.display_tree_(&format!("{prefix}  "));
            }
            Let { e1, binding, e2 } => {
                println!("{prefix}+-Let {binding} = ... in ...");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Add(e1, e2) => {
                println!("{prefix}+-Add");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Seq(e1, e2) => {
                println!("{prefix}+-Seq");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
        }
    }
}

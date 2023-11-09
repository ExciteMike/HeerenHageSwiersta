#![allow(clippy::missing_errors_doc)]
use crate::{ApplySubst, Substitutions, Type};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum TypedIr {
    Nop(Type),
    LiteralInt(i64, Type),
    LiteralStr(&'static str, Type),
    Id(&'static str, Type),
    App {
        e1: Box<TypedIr>,
        e2: Box<TypedIr>,
        ty: Type,
    },
    Lam {
        binding: &'static str,
        body: Box<TypedIr>,
        ty: Type,
    },
    Let {
        e1: Box<TypedIr>,
        binding: &'static str,
        e2: Box<TypedIr>,
        ty: Type,
    },
    Add(Box<TypedIr>, Box<TypedIr>),
    Seq(Box<TypedIr>, Box<TypedIr>),
}
impl TypedIr {
    #[must_use]
    pub fn ty(&self) -> &Type {
        use TypedIr::*;
        match self {
            Nop(ty)
            | LiteralInt(_, ty)
            | LiteralStr(_, ty)
            | Id(_, ty)
            | App { ty, .. }
            | Lam { ty, .. }
            | Let { ty, .. } => ty,
            Add(inner, _) => inner.ty(),
            Seq(_, inner) => inner.ty(),
        }
    }

    pub fn display_tree(&self) {
        self.display_tree_("");
    }
    fn display_tree_(&self, prefix: &str) {
        use TypedIr::*;
        match self {
            Nop(ty) => println!("{prefix}+-NOP {ty:?}"),
            LiteralStr(s, ty) => println!("{prefix}+-\"{s}\" : {ty:?}"),
            LiteralInt(i, ty) => println!("{prefix}+-{i} : {ty:?}"),
            Id(s, ty) => println!("{prefix}+-ID `{s}` : {ty:?}"),
            App { e1, e2, ty } => {
                println!("{prefix}+-App {ty:?}");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Lam { binding, body, ty } => {
                println!("{prefix}+-Lambda {binding} -> ... : {ty:?}");
                println!("{prefix}  |");
                body.display_tree_(&format!("{prefix}  "));
            }
            Let {
                e1,
                binding,
                e2,
                ty,
            } => {
                println!("{prefix}+-Let {binding} = ... in ... : {ty:?}");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Add(e1, e2) => {
                let ty = e1.ty();
                println!("{prefix}+-Add : {ty:?}");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
            Seq(e1, e2) => {
                let ty = e2.ty();
                println!("{prefix}+-Seq : {ty:?}");
                println!("{prefix}  |  |");
                e1.display_tree_(&format!("{prefix}  |  "));
                println!("{prefix}  |");
                e2.display_tree_(&format!("{prefix}  "));
            }
        }
    }
    pub fn to_string(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.to_string_("")
    }
    fn to_string_(&self, indent: &str) -> Result<String, Box<dyn std::error::Error>> {
        use TypedIr::*;
        let mut buf = String::new();
        let increased_indent = format!("{indent}    ");
        match self {
            Nop(ty) => write!(buf, "NOP:{ty:?}")?,
            LiteralStr(s, ty) => write!(buf, "\"{s}\":{ty:?}")?,
            LiteralInt(i, ty) => write!(buf, "{i}:{ty:?}")?,
            Id(s, ty) => write!(buf, "{s}:{ty:?}")?,
            App { e1, e2, ty } => {
                write!(buf, "{}({}):{ty:?}", e1.to_string()?, e2.to_string()?,)?;
            }
            Lam { binding, body, ty } => {
                let body = body.to_string_(&increased_indent)?;
                write!(
                    buf,
                    "lambda {binding} -> {{\n{increased_indent}{body}\n{indent}}} : {ty:?}"
                )?;
            }
            Let {
                e1,
                binding,
                e2,
                ty,
            } => {
                let e1 = e1.to_string_(&increased_indent)?;
                let e2 = e2.to_string_(&increased_indent)?;
                write!(buf,"let {binding} = {{\n{increased_indent}{e1}\n{indent}}} in {{\n{increased_indent}{e2}\n{indent}}} : {ty:?}")?;
            }
            Add(e1, e2) => {
                let ty = e1.ty();
                let e1 = e1.to_string()?;
                let e2 = e2.to_string()?;
                write!(buf, "({e1} + {e2} : {ty:?})")?;
            }
            Seq(e1, e2) => {
                let ty = e2.ty();
                let e1 = e1.to_string()?;
                let e2 = e2.to_string_(indent)?;
                write!(buf, "{e1}\n{indent}{e2}\n{indent}: {ty:?}")?;
            }
        }
        Ok(buf)
    }
}
impl ApplySubst for TypedIr {
    fn apply_subst(&self, subs: &Substitutions) -> Option<Self>
    where
        Self: Sized,
    {
        use TypedIr::*;
        match self {
            Nop(_) | LiteralInt(_, _) | LiteralStr(_, _) => None,
            Id(name, ty) => ty.apply_subst(subs).map(|ty| Id(name, ty)),
            App { e1, e2, ty } => {
                let se1 = e1.apply_subst(subs);
                let se2 = e2.apply_subst(subs);
                let sty = ty.apply_subst(subs);
                if se1.is_some() || se2.is_some() || sty.is_some() {
                    let e1 = se1.map_or_else(|| Box::clone(e1), Box::new);
                    let e2 = se2.map_or_else(|| Box::clone(e2), Box::new);
                    let ty = sty.unwrap_or_else(|| ty.clone());
                    Some(App { e1, e2, ty })
                } else {
                    None
                }
            }
            Lam { binding, body, ty } => {
                let subbed_body = body.apply_subst(subs);
                let subbed_ty = ty.apply_subst(subs);
                if subbed_body.is_some() || subbed_ty.is_some() {
                    let body = subbed_body.map_or_else(|| Box::clone(body), Box::new);
                    let ty = subbed_ty.unwrap_or_else(|| ty.clone());
                    Some(Lam { binding, body, ty })
                } else {
                    None
                }
            }
            Let {
                e1,
                binding,
                e2,
                ty,
            } => {
                let se1 = e1.apply_subst(subs);
                let se2 = e2.apply_subst(subs);
                let sty = ty.apply_subst(subs);
                if se1.is_some() || se2.is_some() || sty.is_some() {
                    let e1 = se1.map_or_else(|| Box::clone(e1), Box::new);
                    let e2 = se2.map_or_else(|| Box::clone(e2), Box::new);
                    let ty = sty.unwrap_or_else(|| ty.clone());
                    Some(Let {
                        e1,
                        binding,
                        e2,
                        ty,
                    })
                } else {
                    None
                }
            }
            Add(lhs, rhs) => {
                let slhs = lhs.apply_subst(subs);
                let srhs = rhs.apply_subst(subs);
                if slhs.is_some() || srhs.is_some() {
                    let lhs = slhs.map_or_else(|| Box::clone(lhs), Box::new);
                    let rhs = srhs.map_or_else(|| Box::clone(rhs), Box::new);
                    Some(Add(lhs, rhs))
                } else {
                    None
                }
            }
            Seq(lhs, rhs) => {
                let slhs = lhs.apply_subst(subs);
                let srhs = rhs.apply_subst(subs);
                if slhs.is_some() || srhs.is_some() {
                    let lhs = slhs.map_or_else(|| Box::clone(lhs), Box::new);
                    let rhs = srhs.map_or_else(|| Box::clone(rhs), Box::new);
                    Some(Seq(lhs, rhs))
                } else {
                    None
                }
            }
        }
    }
}

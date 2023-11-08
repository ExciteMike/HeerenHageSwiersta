#![warn(clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
mod apply_subst;
mod ast;
mod constraints;
mod free_vars;
mod generalize;
mod infer_type;
mod instantiate;
mod ir;
mod mgu;
mod solve;
mod r#type;
mod typedir;

use std::collections::{HashMap, HashSet};

pub use apply_subst::ApplySubst;
pub use ast::Ast;
pub use constraints::*;
pub use free_vars::*;
pub use generalize::*;
pub use infer_type::*;
pub use instantiate::*;
pub use ir::Ir;
pub use mgu::mgu;
use r#type::Scheme;
pub use r#type::Type;
pub use solve::solve;
pub use typedir::TypedIr;

type Assumptions = HashSet<(&'static str, Type)>;
type Environment = HashMap<&'static str, Scheme>;
/// not using an actual Set type because it needs to be hashable
type TypeSet = Box<[Type]>;
type Substitutions = HashMap<u32, Type>;

/// combine substitutions
#[must_use]
pub fn compose(mut s1: Substitutions, mut s2: Substitutions) -> Substitutions {
    for value in s2.values_mut() {
        if let Some(new_value) = value.apply_subst(&s1) {
            *value = new_value;
        }
    }
    s1.extend(s2);
    s1
}

/// make a brand new type variable id that isn't used anywhere yet
pub fn fresh_type_id() -> u32 {
    use std::sync::atomic::AtomicU32;
    static ID: AtomicU32 = AtomicU32::new(0);
    ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// make a brand new type variable that isn't isn't used anywhere yet
pub fn fresh_type_var() -> Type {
    use std::sync::atomic::AtomicU32;
    static ID: AtomicU32 = AtomicU32::new(0);
    let id = ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Type::Unknown(id)
}

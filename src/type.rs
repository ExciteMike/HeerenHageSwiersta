use std::collections::HashSet;

use crate::FreeVars;

pub type Scheme = (HashSet<u32>, Type);

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Type {
    Int,
    Str,
    Nothing,
    F(Box<Type>, Box<Type>),
    //Tuple(Box<Type>, Box<Type>),
    Unknown(u32),
}

impl FreeVars for Type {
    fn free_vars(&self) -> HashSet<u32> {
        use Type::*;
        match self {
            Int | Str | Nothing => HashSet::new(),
            F(t1, t2) => &t1.free_vars() | &t2.free_vars(),
            Unknown(id) => [*id].into(),
        }
    }
}

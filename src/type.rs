use std::collections::HashSet;

use itertools::Itertools;

use crate::FreeVars;

pub type Scheme = (HashSet<u32>, Type);

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    Int,
    Str,
    Nothing,
    F(Box<Type>, Box<Type>),
    //Tuple(Box<Type>, Box<Type>),
    Unknown(u32),
    Scheme(HashSet<u32>, Box<Type>),
}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Type::*;
        core::mem::discriminant(self).hash(state);
        match self {
            Int | Str | Nothing => (),
            F(t1, t2) => {
                t1.hash(state);
                t2.hash(state);
            }
            Unknown(id) => id.hash(state),
            Scheme(alphas, tau) => {
                alphas.iter().sorted().collect_vec().hash(state);
                tau.hash(state);
            }
        }
    }
}

impl FreeVars for Type {
    fn free_vars(&self) -> HashSet<u32> {
        use Type::*;
        match self {
            Int | Str | Nothing => HashSet::new(),
            F(t1, t2) => &t1.free_vars() | &t2.free_vars(),
            Unknown(id) => [*id].into(),
            Scheme(a, t) => (&t.free_vars()) - a,
        }
    }
}

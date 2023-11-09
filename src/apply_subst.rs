use std::{
    collections::HashSet,
    hash::{BuildHasher, Hash},
};

use crate::{Substitutions, Type};

/// Trait for applying `Substitutions` without needing mutability or ownership.
pub trait ApplySubst {
    fn apply_subst(&mut self, subs: &Substitutions);
}
impl<T> ApplySubst for Box<[T]>
where
    T: ApplySubst + Clone,
{
    fn apply_subst(&mut self, subs: &Substitutions) {
        for t in self.iter_mut() {
            t.apply_subst(subs);
        }
    }
}
impl<T> ApplySubst for (T, T)
where
    T: ApplySubst + Clone,
{
    fn apply_subst(&mut self, subs: &Substitutions) {
        self.0.apply_subst(subs);
        self.1.apply_subst(subs);
    }
}
impl ApplySubst for Type {
    fn apply_subst(&mut self, subs: &Substitutions) {
        use Type::*;
        match self {
            Int | Str | Nothing => (),
            F(t1, t2) => {
                t1.apply_subst(subs);
                t2.apply_subst(subs);
            }
            Unknown(id) => {
                if let Some(new) = subs.get(id) {
                    new.clone_into(self);
                }
            }
        }
    }
}
impl<T, S> ApplySubst for HashSet<T, S>
where
    T: Eq + Hash + ApplySubst + Clone,
    S: BuildHasher + Default,
{
    fn apply_subst(&mut self, subs: &Substitutions) {
        *self = self
            .iter()
            .map(|t| {
                let mut t = t.clone();
                t.apply_subst(subs);
                t
            })
            .collect();
    }
}

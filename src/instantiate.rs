use crate::{fresh_type_var, ApplySubst, Substitutions, Type};

pub fn instantiate<I>(quantified_type_vars: I, t: &Type) -> Type
where
    I: Iterator<Item = u32>,
{
    let subs: Substitutions = quantified_type_vars
        .map(|a| (a, fresh_type_var()))
        .collect();
    let mut t = t.clone();
    t.apply_subst(&subs);
    t
}

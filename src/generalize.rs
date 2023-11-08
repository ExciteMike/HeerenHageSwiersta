use crate::{r#type::Scheme, FreeVars, Type};

/// Makes t into a type scheme.
/// In jargon: quantifies the type variables that are free in t but do not appear in env
/// TODO: I think in the only place I generalize, the only thing that happens to the result is instantiation
/// Could probably combine those into one operation.
pub fn generalize<M>(env: &M, t: &Type) -> Scheme
where
    M: FreeVars,
{
    let env = env.free_vars();
    let mut quantified_type_vars = t.free_vars();
    quantified_type_vars.retain(|t| !env.contains(t));
    let quantified_type_vars = quantified_type_vars.into_iter().collect();
    (quantified_type_vars, t.clone())
}

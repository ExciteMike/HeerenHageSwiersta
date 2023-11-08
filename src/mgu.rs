use crate::{compose, Substitutions, Type};

/// find the most general unifier for the two types
/// # Panics
/// Panics if it encounters a pair of types I didn't anticipate
#[must_use]
pub fn mgu(t1: &Type, t2: &Type) -> Substitutions {
    use Type::*;
    match (t1, t2) {
        (Int, Int) | (Str, Str) | (Nothing, Nothing) => Substitutions::new(),
        (F(param1, result1), F(param2, result2)) => {
            let s1 = mgu(param1, param2);
            let s2 = mgu(result1, result2);
            compose(s1, s2)
        }
        (Unknown(id1), Unknown(id2)) if id1 == id2 => Substitutions::new(),
        (Unknown(id), known) | (known, Unknown(id)) => [(*id, known.clone())].into_iter().collect(),
        _ => panic!("Unable to unify types: {t1:?} {t2:?}"),
    }
}

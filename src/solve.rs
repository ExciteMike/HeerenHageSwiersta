use crate::{compose, generalize, instantiate, mgu, Constraints, Substitutions};

/// attempt to solve a set of constraints
#[must_use]
pub fn solve(cs: Constraints) -> Substitutions {
    let mut cs = cs;
    let mut subs = Substitutions::new();
    while !cs.is_empty() {
        if let Some(c) = cs.next_eq() {
            cs.remove_eq(&c);
            let (t1, t2) = c;
            let s = mgu(&t1, &t2);
            cs.apply_subst_in_place(&s);
            subs = compose(s, subs);
        } else if let Some(exp) = cs.next_explicit() {
            cs.remove_exp(&exp);
            let t2 = instantiate(exp.scheme.0.iter().copied(), &exp.scheme.1);
            cs.insert_eq(exp.instance, t2);
            // TODO: could probably save a step and instead of adding a constraint, turn the explicit instance constraint into a substitution directly
        } else if let Some(imp) = cs.next_implicit() {
            cs.remove_imp(&imp);
            // TODO: it seems to me we could save a couple steps by reinstantiating and unifying right away
            let scheme = generalize(&imp.do_not_generalize, &imp.to_generalize);
            cs.insert_explicit(imp.instance, scheme);
        } else {
            unreachable!("unhandled case in solve! {cs:?}");
        }
    }
    subs
}

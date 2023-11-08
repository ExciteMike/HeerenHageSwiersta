use crate::{Substitutions, Type};

pub trait ApplySubst {
    /// produce a replaced copy, or none if no substitutions were applicable
    fn apply_subst(&self, subs: &Substitutions) -> Option<Self>
    where
        Self: Sized;
}
impl<T> ApplySubst for Box<[T]>
where
    T: ApplySubst + Clone,
{
    fn apply_subst(&self, subs: &Substitutions) -> Option<Self> {
        let mut v = None;
        for (i, t) in self.iter().enumerate() {
            if let Some(u) = t.apply_subst(subs) {
                v.get_or_insert_with(|| {
                    let mut v = Vec::with_capacity(self.len());
                    v.extend_from_slice(&self[..i]);
                    v
                })
                .push(u);
            } else if let Some(v) = v.as_mut() {
                v.push(t.clone());
            }
        }
        v.map(Vec::into_boxed_slice)
    }
}
impl<T> ApplySubst for (T, T)
where
    T: ApplySubst + Clone,
{
    fn apply_subst(&self, subs: &Substitutions) -> Option<Self> {
        let (t1, t2) = self;
        let u1 = t1.apply_subst(subs);
        let u2 = t2.apply_subst(subs);
        if u1.is_some() || u2.is_some() {
            let t1 = u1.unwrap_or_else(|| t1.clone());
            let t2 = u2.unwrap_or_else(|| t2.clone());
            Some((t1, t2))
        } else {
            None
        }
    }
}
impl ApplySubst for Type {
    fn apply_subst(&self, subs: &Substitutions) -> Option<Type> {
        use Type::*;
        match self {
            Int | Str | Nothing => None,
            F(t1, t2) => {
                let u1 = t1.apply_subst(subs);
                let u2 = t2.apply_subst(subs);
                if u1.is_some() || u2.is_some() {
                    let t1 = u1.map_or_else(|| t1.clone(), Box::new);
                    let t2 = u2.map_or_else(|| t2.clone(), Box::new);
                    Some(F(t1, t2))
                } else {
                    None
                }
            }
            Unknown(id) => subs.get(id).cloned(),
            Scheme(alphas, tau) => tau
                .apply_subst(subs)
                .map(|tau| Scheme(alphas.clone(), tau.into())),
        }
    }
}

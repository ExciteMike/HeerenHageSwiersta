use std::{collections::HashSet, hash::Hash};

use itertools::Itertools;

use crate::{r#type::Scheme, ApplySubst, FreeVars, Substitutions, Type};

/// constraint set, organized by type
#[derive(Default, Debug)]
pub struct Constraints {
    equality: HashSet<(Type, Type)>,
    explicit: HashSet<ExplicitInstance>,
    implicit: HashSet<ImplicitInstance>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExplicitInstance {
    pub instance: Type,
    pub scheme: Scheme,
}

impl Hash for ExplicitInstance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.instance.hash(state);
        self.scheme.0.iter().sorted().collect_vec().hash(state);
        self.scheme.1.hash(state);
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ImplicitInstance {
    pub instance: Type,
    pub do_not_generalize: Box<[Type]>,
    pub to_generalize: Type,
}

/// transform a set in place by mapping elements
fn set_xform<T, F>(set: &mut HashSet<T>, f: F)
where
    T: PartialEq + Eq + Hash + Clone,
    F: Fn(&T) -> Option<T>,
{
    let mut replacements = Vec::new();
    for v in &*set {
        if let Some(new) = f(v) {
            replacements.push((v.clone(), new));
        }
    }
    for (from, to) in replacements {
        set.remove(&from);
        set.insert(to);
    }
}

impl Constraints {
    /// apply substitutions to the types appearing in constraints
    pub fn apply_subst_in_place(&mut self, subs: &Substitutions) {
        set_xform(&mut self.equality, |x| x.apply_subst(subs));
        set_xform(&mut self.explicit, |x| x.apply_subst(subs));
        set_xform(&mut self.implicit, |x| x.apply_subst(subs));
    }
    /// add an equality constraint
    pub fn insert_eq(&mut self, left: Type, right: Type) {
        self.equality.insert((left, right));
    }
    /// add an explicit instance constraint
    /// `instance` - the type that should be an instance of the scheme
    /// `scheme` - type scheme which we require instance to match
    pub fn insert_explicit(&mut self, instance: Type, scheme: (HashSet<u32>, Type)) {
        self.explicit.insert(ExplicitInstance { instance, scheme });
    }
    /// add an implicit instance constraint
    /// `instance` - the type that should be an instance of the yet-to-be-determined scheme
    /// `do_not_generalize` - types to not generalize when they appear in `to_generalize`
    /// `to_generalize` - type to be generalized into a scheme
    pub fn insert_implicit(
        &mut self,
        instance: Type,
        do_not_generalize: Box<[Type]>,
        to_generalize: Type,
    ) {
        self.implicit.insert(ImplicitInstance {
            instance,
            do_not_generalize,
            to_generalize,
        });
    }
    /// Returns `true` if the constraint set contains no elements
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.equality.is_empty() && self.explicit.is_empty() && self.implicit.is_empty()
    }
    /// merge another set of constraints into this one
    pub fn merge(&mut self, other: Self) {
        self.equality.extend(other.equality);
        self.explicit.extend(other.explicit);
        self.implicit.extend(other.implicit);
    }
    /// create default  (empty) constraint set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Get an arbitrary one of the equality constraints, if any.
    #[must_use]
    pub fn next_eq(&self) -> Option<(Type, Type)> {
        self.equality.iter().next().cloned()
    }
    /// Get an arbitrary one of the explicit instance constraints, if any.
    #[must_use]
    pub fn next_explicit(&self) -> Option<ExplicitInstance> {
        self.explicit.iter().next().cloned()
    }
    /// Get an arbitrary one of the implicit instance constraints, if any.
    #[must_use]
    pub fn next_implicit(&self) -> Option<ImplicitInstance> {
        self.implicit
            .iter()
            .find(|ImplicitInstance { to_generalize, .. }| {
                to_generalize.free_vars().is_disjoint(&self.active_vars())
            })
            .cloned()
    }
    /// remove an equality constraint
    pub fn remove_eq(&mut self, value: &(Type, Type)) -> bool {
        self.equality.remove(value)
    }
    /// remove an explicit instance constraint
    pub fn remove_exp(&mut self, value: &ExplicitInstance) -> bool {
        self.explicit.remove(value)
    }
    /// remove an implicit instance constraint
    pub fn remove_imp(&mut self, value: &ImplicitInstance) -> bool {
        self.implicit.remove(value)
    }
}

impl ApplySubst for ExplicitInstance {
    fn apply_subst(&self, subs: &Substitutions) -> Option<ExplicitInstance> {
        let ExplicitInstance { instance, scheme } = self;
        let st1 = instance.apply_subst(subs);
        let st2 = scheme.1.apply_subst(subs);
        if st1.is_some() || st2.is_some() {
            let instance = st1.unwrap_or_else(|| instance.clone());
            let scheme_ty = st2.unwrap_or_else(|| scheme.1.clone());
            Some(ExplicitInstance {
                instance,
                scheme: (scheme.0.clone(), scheme_ty),
            })
        } else {
            None
        }
    }
}

impl ApplySubst for ImplicitInstance {
    fn apply_subst(&self, subs: &Substitutions) -> Option<ImplicitInstance> {
        let ImplicitInstance {
            instance,
            do_not_generalize,
            to_generalize,
        } = self;
        let u1 = instance.apply_subst(subs);
        let dng = do_not_generalize.apply_subst(subs);
        let u2 = to_generalize.apply_subst(subs);
        if u1.is_some() || dng.is_some() || u2.is_some() {
            let instance = u1.unwrap_or_else(|| instance.clone());
            let do_not_generalize = dng.unwrap_or_else(|| do_not_generalize.clone());
            let to_generalize = u2.unwrap_or_else(|| to_generalize.clone());
            Some(ImplicitInstance {
                instance,
                do_not_generalize,
                to_generalize,
            })
        } else {
            None
        }
    }
}

trait ActiveVars {
    fn active_vars(&self) -> HashSet<u32>;
}

impl ActiveVars for Constraints {
    fn active_vars(&self) -> HashSet<u32> {
        let mut active_vars = HashSet::new();
        for (t1, t2) in &self.equality {
            active_vars.extend(t1.free_vars().iter());
            active_vars.extend(t2.free_vars().iter());
        }
        for ExplicitInstance { instance, scheme } in &self.explicit {
            active_vars.extend(instance.free_vars().iter());
            active_vars.extend(scheme.1.free_vars().difference(&scheme.0));
        }
        for ImplicitInstance {
            instance,
            do_not_generalize,
            to_generalize,
        } in &self.implicit
        {
            active_vars.extend(instance.free_vars().iter());
            active_vars.extend(&do_not_generalize.free_vars() & &to_generalize.free_vars());
        }
        active_vars
    }
}

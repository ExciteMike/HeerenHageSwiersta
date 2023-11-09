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
/// Sometimes we do not know the polymorphic type of a declaration in a `let`
/// expression right away. Implicit instance constraints are our way of
/// defering an instance constraint until it is known.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ImplicitInstance {
    /// type that must be a specialization of the yet-to-be-determined scheme
    pub instance: Type,
    /// Types that are monomorphic due to environment. These do not become
    /// quantified variables when we generalize `to_generalize`.
    pub monomorphics: Box<[Type]>,
    /// pseudo-type-scheme. Type variables within it, excepting those in
    /// `monomorphics`, will be made into quantified types, then the resulting
    /// scheme will be instantiated and unified with `instance`
    pub to_generalize: Type,
}

impl Constraints {
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
    /// `monomorphics` - types to not generalize when they appear in `to_generalize`
    /// `to_generalize` - type to be generalized into a scheme
    pub fn insert_implicit(
        &mut self,
        instance: Type,
        monomorphics: Box<[Type]>,
        to_generalize: Type,
    ) {
        self.implicit.insert(ImplicitInstance {
            instance,
            monomorphics,
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
impl ApplySubst for Constraints {
    /// apply substitutions to the types appearing in constraints
    fn apply_subst(&mut self, subs: &Substitutions) {
        self.equality.apply_subst(subs);
        self.explicit.apply_subst(subs);
        self.implicit.apply_subst(subs);
    }
}

impl ApplySubst for ExplicitInstance {
    fn apply_subst(&mut self, subs: &Substitutions) {
        self.instance.apply_subst(subs);
        self.scheme.1.apply_subst(subs);
    }
}

impl ApplySubst for ImplicitInstance {
    fn apply_subst(&mut self, subs: &Substitutions) {
        self.instance.apply_subst(subs);
        self.monomorphics.apply_subst(subs);
        self.to_generalize.apply_subst(subs);
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
            monomorphics,
            to_generalize,
        } in &self.implicit
        {
            active_vars.extend(instance.free_vars().iter());
            active_vars.extend(&monomorphics.free_vars() & &to_generalize.free_vars());
        }
        active_vars
    }
}

use crate::{
    fresh_type_var, solve, ApplySubst, Assumptions, Constraints, Environment, Ir, Substitutions,
    Type, TypeSet, TypedIr,
};
use itertools::Itertools;

/// # Panics
/// If the expression referred to an identifier that could not be
/// found in that scope, it panics with a message including those varnames
#[must_use]
pub fn infer_type(environment: &Environment, expr: Ir) -> (Substitutions, TypedIr) {
    //let env_types = environment.iter().map(|(_, (_, t))| t.clone()).collect();
    let InferStep {
        assumptions,
        constraints,
        typed_expr,
    } = infer_type_(&TypeSet::default(), expr);

    // `ids` -- identifiers that couldn't be found in expr need to come from environment
    // `constraints` -- if they are in the environment, make sure that expr's usage of them
    // matches their scheme
    let mut ids = Vec::new();
    let mut constraints = constraints;
    for (name, t) in &assumptions {
        if let Some(s) = environment.get(name) {
            constraints.insert_explicit(t.clone(), s.clone());
        } else {
            ids.push(*name);
        }
    }
    assert!(
        ids.is_empty(),
        "unrecognized identifiers: {}",
        ids.iter().join(", ")
    );

    let substitutions = solve(constraints);
    eprintln!("subs after solve {substitutions:?}");
    let typed_expr = typed_expr.apply_subst(&substitutions).unwrap_or(typed_expr);
    (substitutions, typed_expr)
}

struct InferStep {
    assumptions: Assumptions,
    constraints: Constraints,
    typed_expr: TypedIr,
}

impl InferStep {
    pub fn nop() -> Self {
        InferStep {
            assumptions: Assumptions::default(),
            constraints: Constraints::default(),
            typed_expr: TypedIr::Nop(Type::Nothing),
        }
    }
    pub fn literal_int(i: i64) -> Self {
        InferStep {
            assumptions: Assumptions::default(),
            constraints: Constraints::default(),
            typed_expr: TypedIr::LiteralInt(i, Type::Int),
        }
    }
    pub fn literal_str(s: &'static str) -> Self {
        InferStep {
            assumptions: Assumptions::default(),
            constraints: Constraints::default(),
            typed_expr: TypedIr::LiteralStr(s, Type::Str),
        }
    }
    pub fn var(s: &'static str) -> Self {
        let fresh = fresh_type_var();
        InferStep {
            assumptions: [(s, fresh.clone())].into_iter().collect(),
            constraints: Constraints::default(),
            typed_expr: TypedIr::Id(s, fresh),
        }
    }
    pub fn app(infer1: Self, infer2: Self) -> Self {
        let fresh = fresh_type_var();
        let mut assumptions = infer1.assumptions;
        assumptions.extend(infer2.assumptions);
        let mut constraints = infer1.constraints;
        constraints.merge(infer2.constraints);
        constraints.insert_eq(
            infer1.typed_expr.ty().clone(),
            Type::F(infer2.typed_expr.ty().clone().into(), fresh.clone().into()),
        );
        InferStep {
            assumptions,
            constraints,
            typed_expr: TypedIr::App {
                e1: infer1.typed_expr.into(),
                e2: infer2.typed_expr.into(),
                ty: fresh,
            },
        }
    }
    pub fn abs(binding: &'static str, body: Self) -> Self {
        let fresh = fresh_type_var();
        let Self {
            mut assumptions,
            mut constraints,
            typed_expr,
        } = body;
        for (name, ty) in &assumptions {
            if *name == binding {
                constraints.insert_eq(ty.clone(), fresh.clone());
            }
        }
        assumptions.retain(|(name, _)| *name != binding);
        let ty = typed_expr.ty().clone().into();
        InferStep {
            assumptions,
            constraints,
            typed_expr: TypedIr::Lam {
                binding,
                body: typed_expr.into(),
                ty: Type::F(fresh.into(), ty),
            },
        }
    }
    pub fn let_(
        monomorphic_types: &TypeSet,
        infer1: Self,
        binding: &'static str,
        infer2: Self,
    ) -> Self {
        let mut constraints = infer1.constraints;
        constraints.merge(infer2.constraints);
        let monomorphic_types: Box<[Type]> = monomorphic_types
            .iter()
            .cloned()
            .collect_vec()
            .into_boxed_slice();
        for (name, ty) in &infer2.assumptions {
            if *name == binding {
                constraints.insert_implicit(
                    ty.clone(),
                    monomorphic_types.clone(),
                    infer1.typed_expr.ty().clone(),
                );
            }
        }

        let mut assumptions = infer2.assumptions;
        assumptions.retain(|(name, _)| *name != binding);
        assumptions.extend(infer1.assumptions);
        let ty = infer2.typed_expr.ty().clone();
        InferStep {
            assumptions,
            constraints,
            typed_expr: TypedIr::Let {
                e1: infer1.typed_expr.into(),
                binding,
                e2: infer2.typed_expr.into(),
                ty,
            },
        }
    }
    pub fn add(lhs: Self, rhs: Self) -> Self {
        let mut assumptions = lhs.assumptions;
        assumptions.extend(rhs.assumptions);
        let mut constraints = lhs.constraints;
        constraints.merge(rhs.constraints);
        constraints.insert_eq(lhs.typed_expr.ty().clone(), Type::Int);
        constraints.insert_eq(rhs.typed_expr.ty().clone(), Type::Int);
        InferStep {
            assumptions,
            constraints,
            typed_expr: TypedIr::Add(lhs.typed_expr.into(), rhs.typed_expr.into()),
        }
    }
    pub fn seq(lhs: Self, rhs: Self) -> Self {
        let mut assumptions = lhs.assumptions;
        assumptions.extend(rhs.assumptions);
        let mut constraints = lhs.constraints;
        constraints.merge(rhs.constraints);
        InferStep {
            assumptions,
            constraints,
            typed_expr: TypedIr::Seq(lhs.typed_expr.into(), rhs.typed_expr.into()),
        }
    }
}

fn infer_type_(monomorphic_types: &TypeSet, expr: Ir) -> InferStep {
    use Ir::*;
    match expr {
        Nop => InferStep::nop(),
        LiteralInt(i) => InferStep::literal_int(i),
        LiteralStr(s) => InferStep::literal_str(s),
        Id(s) => InferStep::var(s),
        App { e1, e2 } => {
            let infer1 = infer_type_(monomorphic_types, *e1);
            let infer2 = infer_type_(monomorphic_types, *e2);
            InferStep::app(infer1, infer2)
        }
        Lam { binding, body } => {
            let body = infer_type_(monomorphic_types, *body);
            InferStep::abs(binding, body)
        }
        Let { e1, binding, e2 } => {
            let infer1 = infer_type_(monomorphic_types, *e1);
            let infer2 = infer_type_(monomorphic_types, *e2);
            InferStep::let_(monomorphic_types, infer1, binding, infer2)
        }
        Add(lhs, rhs) => {
            let lhs = infer_type_(monomorphic_types, *lhs);
            let rhs = infer_type_(monomorphic_types, *rhs);
            InferStep::add(lhs, rhs)
        }
        Seq(lhs, rhs) => {
            let lhs = infer_type_(monomorphic_types, *lhs);
            let rhs = infer_type_(monomorphic_types, *rhs);
            InferStep::seq(lhs, rhs)
        }
    }
}

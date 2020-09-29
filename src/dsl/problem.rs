extern crate uuid;

use std::collections::HashMap;
use std::ops::AddAssign;


use self::uuid::Uuid;
use dsl::*;
use dsl::LpExpression;

/// Enum helping to specify the objective function of the linear problem.
///
/// # Examples:
///
/// ```
/// use lp_modeler::dsl::{LpObjective, LpProblem};
///
/// let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);
/// ```
#[derive(Debug, PartialEq)]
pub enum LpObjective {
    Minimize,
    Maximize,
}

pub trait Problem {
    fn add_objective_expression(&mut self, expr_arena: &mut LpExprArena);
    fn add_constraints(&mut self, contraint_expr: &LpConstraint);
}

/// Structure used for creating the model and solving a linear problem.
///
/// # Examples:
///
/// ```
/// use lp_modeler::dsl::*;
/// use lp_modeler::solvers::{SolverTrait, CbcSolver, Solution};
///
/// let ref a = LpInteger::new("a");
/// let ref b = LpInteger::new("b");
/// let ref c = LpInteger::new("c");
///
/// let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);
/// problem += 10.0 * a + 20.0 * b;
///
/// problem += (500*a + 1200*b + 1500*c).le(10000);
/// problem += (a + b*2 + c).le(10);
/// problem += (a).le(b);
///
/// let solver = CbcSolver::new();
///
/// match solver.run(&problem) {
/// Ok( solution ) => {
///     println!("Status {:?}", solution.status);
///         for (name, value) in solution.results.iter() {
///             println!("value of {} = {}", name, value);
///         }
///     },
///     Err(msg) => println!("{}", msg),
/// }
/// ```
#[derive(Debug)]
pub struct LpProblem {
    pub name: &'static str,
    pub unique_name: String,
    pub objective_type: LpObjective,
    pub obj_expr_arena: Option<LpExprArena>,
    pub constraints: Vec<LpConstraint>,
}

impl LpProblem {
    /// Create a new problem
    pub fn new(name: &'static str, objective: LpObjective) -> LpProblem {
        let unique_name = format!("{}_{}", name, Uuid::new_v4());
        LpProblem {
            name,
            unique_name,
            objective_type: objective,
            obj_expr_arena: None,
            constraints: Vec::new(),
        }
    }
    fn var<'a>(&self, expr: &'a LpExpression, lst: &mut Vec<(String, &'a LpExpression)>) {
        match expr {
            LpExpression::LpAtomicExpr::ConsBin(LpBinary { ref name, .. })
            | LpExpression::LpAtomicExpr::ConsInt(LpInteger { ref name, .. })
            | LpExpression::LpAtomicExpr::ConsCont(LpContinuous { ref name, .. }) => {
                lst.push((name.clone(), expr));
            },
            LpExpression::LpCompExpr(LpExprOp::Multiply, _, e) => {
                self.var(self.obj_expr_arena.unwrap().expr_ref_at(*e), lst);
            },
            LpExpression::LpCompExpr(LpExprOp::Add, e1, e2)
            | LpExpression::LpCompExpr(LpExprOp::Subtract, e1, e2) => {
                self.var(self.obj_expr_arena.unwrap().expr_ref_at(*e1), lst);
                self.var(self.obj_expr_arena.unwrap().expr_ref_at(*e2), lst);
            }
            _ => (),
        }
    }

    // TODO: Call once and pass into parameter
    // TODO: Check variables on the objective function
    pub fn variables(&self) -> HashMap<String, &LpExpression> {
        let mut lst: Vec<_> = Vec::new();
        for e in &self.constraints {
            self.var(self.obj_expr_arena.unwrap().get_root_expr_ref(), &mut lst);
        }
        lst.iter()
            .map(|&(ref n, ref x)| (n.clone(), *x))
            .collect::<HashMap<String, &LpExpression>>()
    }
}

impl Problem for LpProblem {
    fn add_objective_expression(&mut self, expr_arena: &mut LpExprArena) {
        if let Some(e) = &self.obj_expr_arena {
            let (_, simpl_expr) = expr_arena.merge(&e, LpExprOp::Add).simplify()
                .split_constant_and_expr();
            self.obj_expr_arena = Some(simpl_expr);
        } else {
            let (_, simpl_expr) = expr_arena.simplify().split_constant_and_expr();
            self.obj_expr_arena = Some(simpl_expr);
        }
    }

    fn add_constraints(&mut self, constraint_expr: &LpConstraint) {
        self.constraints.push(constraint_expr.clone());
    }
}

macro_rules! impl_addassign_for_generic_problem {
    ($problem: ty) => {
        /// Add constraints
        impl AddAssign<LpConstraint> for $problem {
            fn add_assign(&mut self, _rhs: LpConstraint) {
                self.add_constraints(&_rhs);
            }
        }
        /// Add an expression as an objective function
        impl<T> AddAssign<T> for $problem
        where
            T: Into<LpExprArena>,
        {
            fn add_assign(&mut self, _rhs: T) {
                self.add_objective_expression(&mut _rhs.into());
            }
        }
    };
}
impl_addassign_for_generic_problem!(LpProblem);

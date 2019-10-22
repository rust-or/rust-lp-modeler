extern crate uuid;

use std::collections::HashMap;
use std::ops::AddAssign;


use self::uuid::Uuid;
use dsl::*;
use dsl::LpExpression::*;

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
    fn add_objective_expression(&mut self, expr: &LpExpression);
    fn add_constraints(&mut self, expr: &LpConstraint);
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
    pub obj_expr: Option<LpExpression>,
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
            obj_expr: None,
            constraints: Vec::new(),
        }
    }

    // TODO: Call once and pass into parameter
    // TODO: Check variables on the objective function
    pub fn variables(&self) -> HashMap<String, &LpExpression> {
        fn var<'a>(expr: &'a LpExpression, lst: &mut Vec<(String, &'a LpExpression)>) {
            match expr {
                &ConsBin(LpBinary { ref name, .. })
                | &ConsInt(LpInteger { ref name, .. })
                | &ConsCont(LpContinuous { ref name, .. }) => {
                    lst.push((name.clone(), expr));
                }

                &MulExpr(_, ref e) => {
                    var(&*e, lst);
                }
                &AddExpr(ref e1, ref e2) | &SubExpr(ref e1, ref e2) => {
                    var(&*e1, lst);
                    var(&*e2, lst);
                }
                _ => (),
            }
        }

        let mut lst: Vec<_> = Vec::new();
        for e in &self.constraints {
            var(&e.0, &mut lst);
        }
        lst.iter()
            .map(|&(ref n, ref x)| (n.clone(), *x))
            .collect::<HashMap<String, &LpExpression>>()
    }
}

impl Problem for LpProblem {
    fn add_objective_expression(&mut self, expr: &LpExpression) {
        if let Some(e) = self.obj_expr.clone() {
            let (_, simpl_expr) = split_constant_and_expr(&simplify(&AddExpr(
                Box::new(expr.clone()),
                Box::new(e.clone()),
            )));
            self.obj_expr = Some(simpl_expr);
        } else {
            let (_, simpl_expr) = split_constant_and_expr(&simplify(expr));
            self.obj_expr = Some(simpl_expr);
        }
    }

    fn add_constraints(&mut self, expr: &LpConstraint) {
        self.constraints.push(expr.clone());
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
            T: Into<LpExpression>,
        {
            fn add_assign(&mut self, _rhs: T) {
                self.add_objective_expression(&_rhs.into());
            }
        }
    };
}
impl_addassign_for_generic_problem!(LpProblem);

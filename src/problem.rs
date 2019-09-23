extern crate uuid;

use std;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::ops::AddAssign;

use variables::LpExpression::*;
use variables::*;

use self::uuid::Uuid;

/// Enum helping to specify the objective function of the linear problem.
///
/// # Examples:
///
/// ```
/// use lp_modeler::problem::{LpObjective, LpProblem};
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
/// use lp_modeler::problem::{LpObjective, Problem, LpProblem};
/// use lp_modeler::operations::{LpOperations};
/// use lp_modeler::variables::LpInteger;
/// use lp_modeler::solvers::{SolverTrait, CbcSolver};
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
/// Ok((status, res)) => {
///     println!("Status {:?}", status);
///         for (name, value) in res.iter() {
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
    objective_type: LpObjective,
    obj_expr: Option<LpExpression>,
    constraints: Vec<LpConstraint>,
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
    fn variables(&self) -> HashMap<String, &LpExpression> {
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

pub trait LpFileFormat {
    fn to_lp_file_format(&self) -> String;
    fn write_lp(&self, file_model: &str) -> std::io::Result<()> {
        let mut buffer = File::create(file_model)?;
        buffer.write(self.to_lp_file_format().as_bytes())?;
        Ok(())
    }
}

fn objective_lp_file_block(prob: &LpProblem) -> String {
    // Write objectives
    let obj_type = match prob.objective_type {
        LpObjective::Maximize => "Maximize\n  ",
        LpObjective::Minimize => "Minimize\n  "
    };
    match prob.obj_expr {
        Some(ref expr) => format!("{}obj: {}", obj_type, expr.to_lp_file_format()),
        _ => String::new()
    }
}
fn constraints_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    let mut constraints = prob.constraints.iter();
    let mut index = 1;
    while let Some(ref constraint) = constraints.next() {
        res.push_str(&format!("  c{}: {}\n", index.to_string(), &constraint.to_lp_file_format()));
        index += 1;
    }
    res
}

fn bounds_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    for (_, v) in prob.variables() {
        match v {
            &ConsInt(LpInteger {
                         ref name,
                         lower_bound,
                         upper_bound,
                     })
            | &ConsCont(LpContinuous {
                            ref name,
                            lower_bound,
                            upper_bound,
                        }) => {
                if let Some(l) = lower_bound {
                    res.push_str(&format!("  {} <= {}", &l.to_string(), &name));
                    if let Some(u) = upper_bound {
                        res.push_str(&format!(" <= {}", &u.to_string()));
                    }
                    res.push_str("\n");
                } else if let Some(u) = upper_bound {
                    res.push_str(&format!("  {} <= {}\n", &name, &u.to_string()));
                } else {
                    match v {
                        &ConsCont(LpContinuous { .. }) => {
                            res.push_str(&format!("  {} free\n", &name));
                        } // TODO: IntegerVar => -INF to INF
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    res
}

fn integers_lp_file_block(prob: &LpProblem) -> String {
    let mut res = String::new();
    for (_, v) in prob.variables() {
        match v {
            &ConsInt(LpInteger { ref name, .. }) => {
                res.push_str(format!("{} ", name).as_str());
            }
            _ => (),
        }
    }
    res
}

fn binaries_lp_file_block(prob: &LpProblem) -> String  {
    let mut res = String::new();
    for (_, v) in prob.variables() {
        match v {
            &ConsBin(LpBinary { ref name }) => {
                res.push_str(format!("{} ", name).as_str());
            }
            _ => (),
        }
    }
    res
}


impl LpFileFormat for LpProblem {

    fn to_lp_file_format(&self) -> String {

        let mut buffer = String::new();

        buffer.push_str(format!("\\ {}\n\n", &self.name).as_str());

        buffer.push_str( &objective_lp_file_block(self) );

        let constraints_block = constraints_lp_file_block(self);
        if constraints_block.len() > 0 {
            buffer.push_str(format!("\n\nSubject To\n{}", &constraints_block).as_str());
        }

        let bounds_block = bounds_lp_file_block(self);
        if bounds_block.len() > 0 {
            buffer.push_str(format!("\nBounds\n{}", &bounds_block).as_str());
        }

        let integers_block = integers_lp_file_block(self);
        if integers_block.len() > 0 {
            buffer.push_str(format!("\nGenerals\n  {}\n", &integers_block).as_str());
        }

        let binaries_block = binaries_lp_file_block(self);
        if binaries_block.len() > 0 {
            buffer.push_str(format!("\nBinary\n  {}\n", &binaries_block).as_str());
        }

        buffer.push_str("\nEnd");

        buffer
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

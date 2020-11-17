extern crate uuid;
use coin_cbc;

use dsl::LpExpression::*;
use dsl::*;
use solvers::{Solution, SolverTrait, Status, WithMaxSeconds, WithNbThreads};
use std::collections::HashMap;

/// Solver that calls cbc through [rust bindings](https://github.com/KardinalAI/coin_cbc)
#[derive(Debug, Clone, Default)]
pub struct NativeCbcSolver {
    name: String,
    threads: Option<u32>,
    seconds: Option<u32>,
}

impl NativeCbcSolver {
    pub fn new() -> NativeCbcSolver {
        NativeCbcSolver {
            name: "CbcNative".to_string(),
            threads: None,
            seconds: None,
        }
    }
}

impl WithMaxSeconds<NativeCbcSolver> for NativeCbcSolver {
    fn max_seconds(&self) -> Option<u32> {
        self.seconds
    }
    fn with_max_seconds(&self, seconds: u32) -> NativeCbcSolver {
        NativeCbcSolver {
            seconds: Some(seconds),
            ..self.clone()
        }
    }
}
impl WithNbThreads<NativeCbcSolver> for NativeCbcSolver {
    fn nb_threads(&self) -> Option<u32> {
        self.threads
    }
    fn with_nb_threads(&self, threads: u32) -> NativeCbcSolver {
        NativeCbcSolver {
            threads: Some(threads),
            ..self.clone()
        }
    }
}

/// Recursively unwrap an expression in a Vec of variables and literals.
fn var_lit(expr: &LpExpression, lst: &mut Vec<(String, f32)>, mul: Option<f32>) {
    let mul = match mul {
        Some(lit) => lit,
        None => 1.,
    };
    match expr {
        &ConsBin(LpBinary { ref name, .. })
        | &ConsInt(LpInteger { ref name, .. })
        | &ConsCont(LpContinuous { ref name, .. }) => {
            let coeff = split_constant_and_expr(expr).0;
            let coeff = if coeff == 0. { mul } else { mul * coeff };
            lst.push((name.clone(), coeff));
        }

        MulExpr(val, ref e) => match **e {
            ConsBin(LpBinary { ref name, .. })
            | ConsInt(LpInteger { ref name, .. })
            | ConsCont(LpContinuous { ref name, .. }) => {
                if let LitVal(lit) = *val.clone() {
                    lst.push((name.clone(), mul * lit))
                }
            }
            MulExpr(..) | AddExpr(..) | SubExpr(..) => {
                let next_mul = match *val.clone() {
                    LitVal(lit) => Some(lit * mul),
                    _ => None,
                };
                var_lit(&*e, lst, next_mul)
            }
            _ => (),
        },
        &AddExpr(ref e1, ref e2) => {
            var_lit(&*e1, lst, None);
            var_lit(&*e2, lst, None);
        }
        &SubExpr(ref e1, ref e2) => {
            var_lit(&*e1, lst, None);
            var_lit(&*e2, lst, Some(-1.));
        }
        _ => (),
    }
}

fn always_literal(expr: &LpExpression) -> f64 {
    match *expr {
        LitVal(num) => num as f64,
        _ => panic!("wrong generalization"),
    }
}

fn add_variable(m: &mut coin_cbc::Model, expr: &LpExpression) -> coin_cbc::Col {
    match expr {
        ConsInt(LpInteger {
            name: _,
            lower_bound,
            upper_bound,
        }) => {
            let col = m.add_integer();
            if let Some(lb) = lower_bound {
                m.set_col_lower(col, *lb as f64)
            }
            if let Some(ub) = upper_bound {
                m.set_col_upper(col, *ub as f64)
            }
            col
        }
        ConsCont(LpContinuous {
            name: _,
            lower_bound,
            upper_bound,
        }) => {
            let col = m.add_col();
            if let Some(lb) = lower_bound {
                m.set_col_lower(col, *lb as f64)
            }
            if let Some(ub) = upper_bound {
                m.set_col_upper(col, *ub as f64)
            }
            col
        }
        ConsBin(_) => m.add_binary(),
        _ => panic!("Unexpected LpExpression on LpProblem.variables()!"),
    }
}

impl SolverTrait for NativeCbcSolver {
    type P = LpProblem;

    fn run<'a>(&self, problem: &'a Self::P) -> Result<Solution<'a>, String> {
        let mut m = coin_cbc::Model::default();
        // columns (variables)
        let cols: HashMap<String, coin_cbc::Col> = problem
            .variables()
            .iter()
            .map(|(name, expr)| (name.clone(), add_variable(&mut m, expr)))
            .collect();
        // rows (constraints)
        problem.constraints.iter().for_each(|cons| {
            let row = m.add_row();
            let general = cons.generalize();
            match general.1 {
                Constraint::GreaterOrEqual => m.set_row_lower(row, always_literal(&general.2)),
                Constraint::LessOrEqual => m.set_row_upper(row, always_literal(&general.2)),
                Constraint::Equal => m.set_row_equal(row, always_literal(&general.2)),
            }
            let mut lst: Vec<_> = Vec::new();
            var_lit(&general.0, &mut lst, None);
            lst.iter()
                .for_each(|(n, lit)| m.set_weight(row, cols[n], *lit as f64));
        });
        // objective
        if let Some(objective) = &problem.obj_expr {
            let mut lst: Vec<_> = Vec::new();
            var_lit(&objective, &mut lst, None);
            lst.iter()
                .for_each(|(n, lit)| m.set_obj_coeff(cols[n], *lit as f64))
        }
        m.set_obj_sense(match problem.objective_type {
            LpObjective::Maximize => coin_cbc::Sense::Maximize,
            LpObjective::Minimize => coin_cbc::Sense::Minimize,
        });

        let sol = m.solve();

        Ok(Solution {
            status: match sol.raw().status() {
                coin_cbc::raw::Status::Finished => Status::Optimal,
                coin_cbc::raw::Status::Abandoned => Status::Infeasible,
                _ => Status::NotSolved,
            },
            results: cols
                .iter()
                .map(|(name, col)| (name.to_owned(), sol.col(*col) as f32))
                .collect(),
            related_problem: Some(problem),
        })
    }
}

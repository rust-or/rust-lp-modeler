extern crate uuid;
use coin_cbc;

use dsl::LpExprNode::*;
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

impl LpExpression {
    /// Recursively unwrap an arena of expressions.
    /// Assumes that self.simplify() was called on the arena before starting recursion, here. That way,
    /// `LitVal()`s of nested Multiplication expressions are already accumulated towards the left side
    /// and `LitVal()`s of all Addition / Subtraction expressions are already accumulated towards the
    /// right end of the tree, with all Addition / SUbtraction expressions on the most global level
    /// possible.
    fn var_lit(&self, expr_index: LpExprArenaIndex, lst: &mut Vec<(String, f32)>, mul: f32) {
        match self.expr_ref_at(expr_index) {
            &ConsBin(LpBinary { ref name, .. })
            | &ConsInt(LpInteger { ref name, .. })
            | &ConsCont(LpContinuous { ref name, .. }) => {
                lst.push((name.clone(), mul));
            }

            LpCompExpr(LpExprOp::Multiplication, val, ref e) => {
                match self.expr_ref_at(*e) {
                    &ConsBin(LpBinary { ref name, .. })
                    | &ConsInt(LpInteger { ref name, .. })
                    | &ConsCont(LpContinuous { ref name, .. }) => {
                        if let &LitVal(lit) = self.expr_ref_at(*val) {
                            lst.push((name.clone(), mul * lit))
                        } else {
                            panic!("This Multiplication expression has a non-`LitVal()` left-hand side.\n\
                                    Did you call `simplify()` before `var_lit()`? If not:\n\
                                    This could point to a bug in LpExpression::simplify().");
                        }
                    },
                    _ => {
                        panic!("This Multiplication expression has a non-`Cons*()` right-hand side.\n\
                                Did you call `simplify()` before `var_lit()`? If not:\n\
                                This could point to a bug in LpExpression::simplify().");
                    },
                }
            },
            &LpCompExpr(LpExprOp::Addition, ref e1, ref e2) => {
                self.var_lit(*e1, lst, 1.0);
                self.var_lit(*e2, lst, 1.0);
            }
            &LpCompExpr(LpExprOp::Subtraction, ref e1, ref e2) => {
                self.var_lit(*e1, lst, 1.0);
                self.var_lit(*e2, lst, -1.0);
            }
            _ => (),
        }
    }
}

fn always_literal(expr_arena: &LpExpression) -> f64 {
    match expr_arena.get_root_expr_ref() {
        &LitVal(num) => num as f64,
        _ => panic!("wrong generalization"),
    }
}

fn add_variable(m: &mut coin_cbc::Model, expr: &LpExprNode) -> coin_cbc::Col {
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
        let mut cols: HashMap<String, coin_cbc::Col> = HashMap::new();
        for (name, (constraint_index, lp_expr_arena_index)) in problem.variables() {
            cols.insert(name, add_variable(&mut m, problem.constraints.get(constraint_index).unwrap().0.expr_ref_at(lp_expr_arena_index) ) );
        }
        // rows (constraints)
        for cons in problem.constraints.clone() {
            let row = m.add_row();
            let mut general = cons.generalize();
            match general.1 {
                Constraint::GreaterOrEqual => m.set_row_lower(row, always_literal(&general.2)),
                Constraint::LessOrEqual => m.set_row_upper(row, always_literal(&general.2)),
                Constraint::Equal => m.set_row_equal(row, always_literal(&general.2)),
            }
            let mut lst: Vec<_> = Vec::new();
            general.0.simplify();
            let root_index = general.0.get_root_index();
            general.0.var_lit(root_index, &mut lst, 1.0);
            lst.iter()
                .for_each(|(n, lit)| m.set_weight(row, cols[n], *lit as f64));
        };
        // objective
        if let Some(objective) = &problem.obj_expr_arena {
            let mut lst: Vec<_> = Vec::new();
            let mut cloned_objective = objective.clone();
            cloned_objective.simplify();
            let root_index = cloned_objective.get_root_index();
            cloned_objective.var_lit(root_index, &mut lst, 1.0);
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

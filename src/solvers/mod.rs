use std::collections::HashMap;

use dsl::Problem;

pub mod cbc;
pub use self::cbc::*;

pub mod gurobi;
pub use self::gurobi::*;

pub mod glpk;
pub use self::glpk::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    Optimal,
    SubOptimal,
    Infeasible,
    Unbounded,
    NotSolved,
}

pub trait SolverTrait {
    type P: Problem;
    fn run(&self, problem: &Self::P) -> Result<(Status, HashMap<String, f32>), String>;
}
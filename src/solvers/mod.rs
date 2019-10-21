use std::collections::HashMap;

use dsl::Problem;

pub mod cbc;
pub use self::cbc::*;

pub mod gurobi;
pub use self::gurobi::*;

pub mod glpk;
pub use self::glpk::*;
use std::fs::File;
use std::fs;

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

pub trait SolverWithSolutionParsing {
    fn read_solution(&self, temp_solution_file: &String) -> Result<(Status, HashMap<String, f32>), String> {
        match File::open( temp_solution_file ) {
            Ok(f) => {
                let res = self.read_specific_solution(&f)?;
                let _ = fs::remove_file(temp_solution_file);
                Ok(res)
            }
            Err(_) => return Err("Cannot open file".to_string()),
        }
    }
    fn read_specific_solution(&self, f: &File) -> Result<(Status, HashMap<String, f32>), String>;
}

pub trait WithMaxSeconds<T> {
    fn max_seconds(&self) -> Option<u32>;
    fn with_max_seconds(&self, seconds: u32) -> T;
}

pub trait WithNbThreads<T> {
    fn nb_threads(&self) -> Option<u32>;
    fn with_nb_threads(&self, threads: u32) -> T;
}


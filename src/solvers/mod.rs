use std::collections::HashMap;

use dsl::{Problem, LpContinuous, LpBinary, LpInteger};

pub mod cbc;
pub use self::cbc::*;

pub mod gurobi;
pub use self::gurobi::*;

pub mod glpk;
pub use self::glpk::*;
use std::fs::File;
use std::fs;
use util::is_zero;

#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    Optimal,
    SubOptimal,
    Infeasible,
    Unbounded,
    NotSolved,
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub status: Status,
    pub results: HashMap<String, f32>
}
impl Solution {
    fn check_possible_solution(&self) {
        match &self.status {
            Status::Unbounded | Status::NotSolved | Status::Infeasible => panic!("Solution must be optimal or suboptimal"),
            _ => ()
        }
    }
    pub fn get_raw_value(&self, name: &str) -> f32 {
        self.check_possible_solution();
        *self.results.get(name).expect("No value found for this variable. Check if the variable has been used in the related problem.")
    }
    pub fn get_bool(&self, var: &LpBinary) -> bool {
        self.check_possible_solution();
        self.results.get(&var.name).and_then(|&f| if is_zero(1.0-f) { Some(true) } else if is_zero(f) { Some(false) } else { None } ).expect("Result value cannot be interpreted as boolean")
    }
    pub fn get_float(&self, var: &LpContinuous) -> f32 {
        self.check_possible_solution();
        *self.results.get(&var.name).expect("No value found for this variable. Check if the variable has been used in the related problem.")
    }
    pub fn get_int(&self, var: &LpInteger) -> i32 {
        self.check_possible_solution();
        let &f = self.results.get(&var.name).expect("No value found for this variable. Check if the variable has been used in the related problem.");
        let i = f as i32;
        assert!( is_zero( f-(i as f32)), format!("Value {} cannot be interpreted as integer.", f) );
        i
    }
}

pub trait SolverTrait {
    type P: Problem;
    fn run(&self, problem: &Self::P) -> Result<Solution, String>;
}

pub trait SolverWithSolutionParsing {
    fn read_solution(&self, temp_solution_file: &String) -> Result<Solution, String> {
        match File::open( temp_solution_file ) {
            Ok(f) => {
                let res = self.read_specific_solution(&f)?;
                let _ = fs::remove_file(temp_solution_file);
                Ok(res)
            }
            Err(_) => return Err("Cannot open file".to_string()),
        }
    }
    fn read_specific_solution(&self, f: &File) -> Result<Solution, String>;
}

pub trait WithMaxSeconds<T> {
    fn max_seconds(&self) -> Option<u32>;
    fn with_max_seconds(&self, seconds: u32) -> T;
}

pub trait WithNbThreads<T> {
    fn nb_threads(&self) -> Option<u32>;
    fn with_nb_threads(&self, threads: u32) -> T;
}


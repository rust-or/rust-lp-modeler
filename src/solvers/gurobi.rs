extern crate uuid;
use self::uuid::Uuid;

use std::fs;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::process::Command;

use dsl::LpProblem;
use format::lp_format::*;
use solvers::{Status, SolverTrait, SolverWithSolutionParsing, Solution};

pub struct GurobiSolver {
    name: String,
    command_name: String,
    temp_solution_file: String,
}

impl GurobiSolver {
    pub fn new() -> GurobiSolver {
        GurobiSolver {
            name: "Gurobi".to_string(),
            command_name: "gurobi_cl".to_string(),
            temp_solution_file: format!("{}.sol", Uuid::new_v4().to_string()),
        }
    }
    pub fn command_name(&self, command_name: String) -> GurobiSolver {
        GurobiSolver {
            name: self.name.clone(),
            command_name,
            temp_solution_file: self.temp_solution_file.clone(),
        }
    }
}

impl SolverWithSolutionParsing for GurobiSolver {
    fn read_specific_solution(&self, f: &File, _problem: Option<&LpProblem>) -> Result<Solution, String> {
        let mut vars_value: HashMap<_, _> = HashMap::new();
        let mut file = BufReader::new(f);
        let mut buffer = String::new();
        let _ = file.read_line(&mut buffer);

        if let Some(_) = buffer.split(" ").next() {
            for line in file.lines() {
                let l = line.unwrap();

                // Gurobi version 7 add comments on the header file
                if let Some('#') = l.chars().next() {
                    continue;
                }

                let result_line: Vec<_> = l.split_whitespace().collect();
                if result_line.len() == 2 {
                    match result_line[1].parse::<f32>() {
                        Ok(n) => {
                            vars_value.insert(result_line[0].to_string(), n);
                        }
                        Err(e) => return Err(format!("{}", e.to_string())),
                    }
                } else {
                    return Err("Incorrect solution format".to_string());
                }
            }
        } else {
            return Err("Incorrect solution format".to_string());
        }
        Ok( Solution { status: Status::Optimal, results: vars_value } )
    }
}

impl SolverTrait for GurobiSolver {
    type P = LpProblem;
    fn run(&self, problem: &Self::P) -> Result<Solution, String> {
        let file_model = &format!("{}.lp", problem.unique_name);

        match problem.write_lp(file_model) {
            Ok(_) => {
                let result = match Command::new(&self.command_name)
                    .arg(format!("ResultFile={}", self.temp_solution_file))
                    .arg(file_model)
                    .output()
                    {
                        Ok(r) => {
                            let mut status = Status::SubOptimal;
                            let result = String::from_utf8(r.stdout).expect("");
                            if result.contains("Optimal solution found")
                            {
                                status = Status::Optimal;
                            } else if result.contains("infesible") {
                                status = Status::Infeasible;
                            }
                            if r.status.success() {
                                self.read_solution(&self.temp_solution_file, Some(problem)).map(|solution| Solution {status, ..solution.clone()} )
                            } else {
                                Err(r.status.to_string())
                            }
                        }
                        Err(_) => Err(format!("Error running the {} solver", self.name)),
                    };
                let _ = fs::remove_file(&file_model);

                result
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

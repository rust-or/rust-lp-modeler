extern crate uuid;
use self::uuid::Uuid;

use std::fs;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::process::Command;

use dsl::LpProblem;
use format::lp_format::*;
use solvers::{Status, SolverTrait};

pub struct CbcSolver {
    name: String,
    command_name: String,
    temp_solution_file: String,
}

impl CbcSolver {
    pub fn new() -> CbcSolver {
        CbcSolver {
            name: "Cbc".to_string(),
            command_name: "cbc".to_string(),
            temp_solution_file: format!("{}.sol", Uuid::new_v4().to_string()),
        }
    }
    pub fn command_name(&self, command_name: String) -> CbcSolver {
        CbcSolver {
            name: self.name.clone(),
            command_name,
            temp_solution_file: self.temp_solution_file.clone(),
        }
    }
    pub fn temp_solution_file(&self, temp_solution_file: String) -> CbcSolver {
        CbcSolver {
            name: self.name.clone(),
            command_name: self.command_name.clone(),
            temp_solution_file,
        }
    }
    pub fn read_solution(&self) -> Result<(Status, HashMap<String, f32>), String> {
        fn read_specific_solution(f: &File) -> Result<(Status, HashMap<String, f32>), String> {
            let mut vars_value: HashMap<_, _> = HashMap::new();

            let mut file = BufReader::new(f);
            let mut buffer = String::new();
            let _ = file.read_line(&mut buffer);

            let status = if let Some(status_line) = buffer.split_whitespace().next() {
                match status_line.split_whitespace().next() {
                    Some("Optimal") => Status::Optimal,
                    // Infeasible status is either "Infeasible" or "Integer infeasible"
                    Some("Infeasible") | Some("Integer") => Status::Infeasible,
                    Some("Unbounded") => Status::Unbounded,
                    // "Stopped" can be "on time", "on iterations", "on difficulties" or "on ctrl-c"
                    Some("Stopped") => Status::SubOptimal,
                    _ => Status::NotSolved,
                }
            } else {
                return Err("Incorrect solution format".to_string());
            };
            for line in file.lines() {
                let l = line.unwrap();
                let result_line: Vec<_> = l.split_whitespace().collect();
                if result_line.len() == 4 {
                    match result_line[2].parse::<f32>() {
                        Ok(n) => {
                            vars_value.insert(result_line[1].to_string(), n);
                        }
                        Err(e) => return Err(e.to_string()),
                    }
                } else {
                    return Err("Incorrect solution format".to_string());
                }
            }
            Ok((status, vars_value))
        }

        match File::open(&self.temp_solution_file) {
            Ok(f) => {
                let res = try!(read_specific_solution(&f));
                let _ = fs::remove_file(&self.temp_solution_file);
                Ok(res)
            }
            Err(_) => return Err("Cannot open file".to_string()),
        }
    }
}

impl SolverTrait for CbcSolver {
    type P = LpProblem;
    fn run(&self, problem: &Self::P) -> Result<(Status, HashMap<String, f32>), String> {
        let file_model = &format!("{}.lp", problem.unique_name);

        match problem.write_lp(file_model) {
            Ok(_) => {
                let result = match Command::new(&self.command_name)
                    .arg(file_model)
                    .arg("solve")
                    .arg("solution")
                    .arg(&self.temp_solution_file)
                    .output()
                    {
                        Ok(r) => {
                            if r.status.success() {
                                self.read_solution()
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


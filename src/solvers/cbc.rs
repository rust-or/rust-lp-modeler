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
    params: HashMap<String, String>,
}

impl CbcSolver {
    pub fn new() -> CbcSolver {
        CbcSolver {
            name: "Cbc".to_string(),
            command_name: "cbc".to_string(),
            temp_solution_file: format!("{}.sol", Uuid::new_v4().to_string()),
            params: Default::default(),
        }
    }

    pub fn command_name(&self, command_name: String) -> CbcSolver {
        CbcSolver {
            name: self.name.clone(),
            command_name,
            temp_solution_file: self.temp_solution_file.clone(),
            params: self.params.clone(),
        }
    }

    pub fn temp_solution_file(&self, temp_solution_file: String) -> CbcSolver {
        CbcSolver {
            name: self.name.clone(),
            command_name: self.command_name.clone(),
            temp_solution_file,
            params: self.params.clone(),
        }
    }

    pub fn seconds(&self, seconds: f32) -> CbcSolver {
        self.add_param("seconds".to_owned(), seconds.to_string())
    }

    pub fn threads(&self, threads: u32) -> CbcSolver {
        self.add_param("threads".to_owned(), threads.to_string())
    }

    fn add_param(&self, name: String, value: String) -> CbcSolver {
        let mut new_params = self.params.clone();
        new_params.insert(name, value);

        CbcSolver {
            name: self.name.clone(),
            command_name: self.command_name.clone(),
            temp_solution_file: self.temp_solution_file.clone(),
            params: new_params,
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
                let mut result_line: Vec<_> = l.split_whitespace().collect();
                if result_line[0] == "**" {
                    result_line.remove(0);
                };
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
        let file_model = format!("{}.lp", problem.unique_name);
        problem.write_lp(&file_model).map_err(|e| e.to_string())?;

        let result = Command::new(&self.command_name)
            .arg(&file_model)
            .args(self.params.iter().flat_map(|(k, v)| vec![k, v]))
            .arg("solve")
            .arg("solution")
            .arg(&self.temp_solution_file)
            .output()
            .map_err(|_| format!("Error running the {} solver", self.name))
            .and_then(|r| {
                if r.status.success() {
                    self.read_solution()
                } else {
                    Err(r.status.to_string())
                }
            });

        let _ = fs::remove_file(file_model);
        result
    }
}

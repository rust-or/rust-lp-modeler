extern crate uuid;
use self::uuid::Uuid;

use std::fs;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::process::Command;

use dsl::LpProblem;
use format::lp_format::*;
use solvers::{Status, SolverTrait, WithMaxSeconds, WithNbThreads, SolverWithSolutionParsing, Solution};

#[derive(Debug, Clone)]
pub struct CbcSolver {
    name: String,
    command_name: String,
    temp_solution_file: String,
    threads: Option<u32>,
    seconds: Option<u32>,
}

impl CbcSolver {
    pub fn new() -> CbcSolver {
        CbcSolver {
            name: "Cbc".to_string(),
            command_name: "cbc".to_string(),
            temp_solution_file: format!("{}.sol", Uuid::new_v4().to_string()),
            threads: None,
            seconds: None,
        }
    }

    pub fn command_name(&self, command_name: String) -> CbcSolver {
        CbcSolver {
            name: self.name.clone(),
            command_name,
            temp_solution_file: self.temp_solution_file.clone(),
            threads: None,
            seconds: None,
        }
    }

    pub fn with_temp_solution_file(&self, temp_solution_file: String) -> CbcSolver {
        CbcSolver {
            name: self.name.clone(),
            command_name: self.command_name.clone(),
            temp_solution_file,
            threads: None,
            seconds: None,
        }
    }
}

impl SolverWithSolutionParsing for CbcSolver {
    fn read_specific_solution<'a>(&self, f: &File, problem: Option<&'a LpProblem>) -> Result<Solution<'a>, String> {
        let mut vars_value: HashMap<_, _> = HashMap::new();

        // populate default values for all vars
        // CBC keeps only non-zero values from a number of variables
        if let Some(p) = problem {
            let variables = p.variables();
            for (name, _) in variables {
                vars_value.insert(name.clone(), 0.0);
            }
        }

        let mut file = BufReader::new(f);
        let mut buffer = String::new();
        let _ = file.read_line(&mut buffer);

        let status = if let Some(status) = buffer.split_whitespace().next() {
            match status {
                "Optimal" => Status::Optimal,
                // Infeasible status is either "Infeasible" or "Integer infeasible"
                "Infeasible" | "Integer" => Status::Infeasible,
                "Unbounded" => Status::Unbounded,
                // "Stopped" can be "on time", "on iterations", "on difficulties" or "on ctrl-c"
                "Stopped" => Status::SubOptimal,
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
        if let Some(p) = problem {
            Ok( Solution::with_problem(status, vars_value, p) )
        } else {
            Ok( Solution::new(status, vars_value) )
        }
    }
}

impl WithMaxSeconds<CbcSolver> for CbcSolver {
    fn max_seconds(&self) -> Option<u32> {
        self.seconds
    }
    fn with_max_seconds(&self, seconds: u32) -> CbcSolver {
        CbcSolver {
            seconds: Some(seconds),
            ..self.clone()
        }
    }
}
impl WithNbThreads<CbcSolver> for CbcSolver {
    fn nb_threads(&self) -> Option<u32> {
        self.threads
    }
    fn with_nb_threads(&self, threads: u32) -> CbcSolver {
        CbcSolver {
            threads: Some(threads),
            ..self.clone()
        }
    }
}

impl SolverTrait for CbcSolver {
    type P = LpProblem;

    fn run<'a>(&self, problem: &'a Self::P) -> Result<Solution<'a>, String> {
        let file_model = format!("{}.lp", problem.unique_name);
        problem.write_lp(&file_model).map_err(|e| e.to_string())?;

        let mut params: HashMap<String, String> = Default::default();
        let optional_params: Vec<Option<(String, u32)>> = vec![
            self.max_seconds().map(|s| ("seconds".to_owned(), s )),
            self.nb_threads().map(|t| ("threads".to_owned(), t)) ];

        for (arg, value) in optional_params.iter().flatten() {
            params.insert(arg.to_string(), value.to_string());
        }
        params.iter().for_each( |(a,b)| println!("{},{}",a,b));

        let result = Command::new(&self.command_name)
            .arg(&file_model)
            .args(params.iter().flat_map(|(k, v)| vec![k, v]))
            .arg("solve")
            .arg("solution")
            .arg(&self.temp_solution_file)
            .output()
            .map_err(|_| format!("Error running the {} solver", self.name))
            .and_then(|r| {
                if r.status.success() {
                    self.read_solution(&self.temp_solution_file, Some(problem))
                } else {
                    Err(r.status.to_string())
                }
            });

        let _ = fs::remove_file(file_model);
        result
    }
}

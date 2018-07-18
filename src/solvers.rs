use std::fs;
use std::io::prelude::*;
use std::process::Command;
use std::collections::HashMap;
use std::io::BufReader;
use problem::{LpProblem, Problem};
use problem::{LpFileFormat};
use std::fs::File;
use std::io::Error;


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
    fn run(&self, problem: &Self::P) -> Result<(Status, HashMap<String,f32>), String>;
}

pub struct GurobiSolver {
    name: String,
    command_name: String,
    temp_solution_file: String,
}
pub struct CbcSolver {
    name: String,
    command_name: String,
    temp_solution_file: String,
}
pub struct GlpkSolver {
    name: String,
    command_name: String,
    temp_solution_file: String,
}

impl GurobiSolver {
    pub fn new() -> GurobiSolver {
        GurobiSolver { name: "Gurobi".to_string(), command_name: "gurobi_cl".to_string(), temp_solution_file: "sol.sol".to_string() }
    }
    pub fn command_name(&self, command_name: String) -> GurobiSolver {
        GurobiSolver { name: self.name.clone(), command_name: command_name, temp_solution_file: self.temp_solution_file.clone() }
    }
    fn read_solution(&self) -> Result<(Status, HashMap<String, f32>), String> {
        fn read_specific_solution(f: &File) -> Result<(Status, HashMap<String, f32>), String> {

            let mut vars_value: HashMap<_,_> = HashMap::new();
            let mut file = BufReader::new(f);
            let mut buffer = String::new();
            let _ = file.read_line(&mut buffer);

            if let Some(_) = buffer.split(" ").next() {
                for line in file.lines() {
                    let l = line.unwrap();

                    // Gurobi version 7 add comments on the header file
                    if let Some('#') = l.chars().next() {continue}

                    let result_line: Vec<_> = l.split_whitespace().collect();
                    if result_line.len() == 2 {
                        match result_line[1].parse::<f32>() {
                            Ok(n) => {
                                vars_value.insert(result_line[0].to_string(), n);
                            },
                            Err(e) => return Err(format!("{}", e.to_string()))
                        }
                    } else {
                        return Err("Incorrect solution format".to_string())
                    }
                }
            }else{
                return Err("Incorrect solution format".to_string())
            }
            Ok((Status::Optimal, vars_value))
        }

        match File::open(&self.temp_solution_file) {
            Ok(f) => {
                let res = try!(read_specific_solution(&f));
                let _ = fs::remove_file(&self.temp_solution_file);
                Ok(res)
            },
            Err(_) => return Err("Cannot open file".to_string())
        }
    }
}
impl CbcSolver {
    pub fn new() -> CbcSolver {
        CbcSolver { name: "Cbc".to_string(), command_name: "cbc".to_string(), temp_solution_file: "sol.sol".to_string() }
    }
    pub fn command_name(&self, command_name: String) -> CbcSolver {
        CbcSolver { name: self.name.clone(), command_name: command_name, temp_solution_file: self.temp_solution_file.clone() }
    }
    pub fn temp_solution_file(&self, temp_solution_file: String) -> CbcSolver {
        CbcSolver { name: self.name.clone(), command_name: self.command_name.clone(), temp_solution_file: temp_solution_file }
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
                    _ => Status::NotSolved
                }
            } else {
                return Err("Incorrect solution format".to_string())
            };
            for line in file.lines() {
                let l = line.unwrap();
                let result_line: Vec<_> = l.split_whitespace().collect();
                if result_line.len() == 4 {
                    match result_line[2].parse::<f32>() {
                        Ok(n) => {
                            vars_value.insert(result_line[1].to_string(), n);
                        },
                        Err(e) => return Err(e.to_string())
                    }
                } else {
                    return Err("Incorrect solution format".to_string())
                }
            }
            Ok((status, vars_value))
        }

        match File::open(&self.temp_solution_file) {
            Ok(f) => {
                let res = try!(read_specific_solution(&f));
                let _ = fs::remove_file(&self.temp_solution_file);
                Ok(res)
            },
            Err(_) => return Err("Cannot open file".to_string())
        }
    }
}
impl GlpkSolver {
    pub fn new() -> GlpkSolver {
        GlpkSolver { name: "Glpk".to_string(), command_name: "glpsol".to_string(), temp_solution_file: "sol.sol".to_string() }
    }
    pub fn command_name(&self, command_name: String) -> GlpkSolver {
        GlpkSolver { name: self.name.clone(), command_name: command_name, temp_solution_file: self.temp_solution_file.clone() }
    }
    pub fn temp_solution_file(&self, temp_solution_file: String) -> GlpkSolver {
        GlpkSolver { name: self.name.clone(), command_name: self.command_name.clone(), temp_solution_file: temp_solution_file }
    }
    pub fn read_solution(&self) -> Result<(Status, HashMap<String, f32>), String> {
        fn read_specific_solution(f: &File) -> Result<(Status, HashMap<String, f32>), String> {
            fn read_size(line: Option<Result<String, Error>>) -> Result<usize, String> {
                match line {
                    Some(Ok(l)) => {
                        match l.split_whitespace().nth(1) {
                            Some(value) => match value.parse::<usize>() {
                                Ok(v) => Ok(v),
                                _ => return Err("Incorrect solution format".to_string())
                            },
                            _ => return Err("Incorrect solution format".to_string())
                        }
                    },
                    _ => return Err("Incorrect solution format".to_string())
                }
            }
            let mut vars_value: HashMap<_, _> = HashMap::new();

            let file = BufReader::new(f);

            let mut iter = file.lines();
            let row = match read_size(iter.nth(1)) {
                Ok(value) => value,
                Err(e) => return Err(e.to_string())
            };
            let col = match read_size(iter.nth(0)) {
                Ok(value) => value,
                Err(e) => return Err(e.to_string())
            };
            let status = match iter.nth(1) {
                Some(Ok(status_line)) => {
                    match &status_line[12..] {
                        "INTEGER OPTIMAL" | "OPTIMAL" => Status::Optimal,
                        "INFEASIBLE (FINAL)" | "INTEGER EMPTY" => Status::Infeasible,
                        "UNDEFINED" => Status::NotSolved,
                        "INTEGER UNDEFINED" | "UNBOUNDED" => Status::Unbounded,
                        _ => return Err("Incorrect solution format".to_string())
                    }
                },
                _ => return Err("Incorrect solution format".to_string())
            };
            let mut result_lines = iter.skip(row + 7);
            for _ in 0..col {
                let line = match result_lines.next() {
                    Some(Ok(l)) => l,
                    _ => return Err("Incorrect solution format".to_string())
                };
                let result_line: Vec<_> = line.split_whitespace().collect();
                if result_line.len() == 5 {
                    match result_line[3].parse::<f32>() {
                        Ok(n) => {
                            vars_value.insert(result_line[1].to_string(), n);
                        },
                        Err(e) => return Err(e.to_string())
                    }
                } else {
                    return Err("Incorrect solution format".to_string())
                }
            }
            Ok((status, vars_value))
        }

        match File::open(&self.temp_solution_file) {
            Ok(f) => {
                let res = try!(read_specific_solution(&f));
                let _ = fs::remove_file(&self.temp_solution_file);
                Ok(res)
            },
            Err(_) => return Err("Cannot open file".to_string())
        }
    }
}


impl SolverTrait for GurobiSolver {
    type P = LpProblem;
    fn run(&self, problem: &Self::P) -> Result<(Status, HashMap<String,f32>), String> {

        let file_model = &format!("{}.lp", problem.name);

        match problem.write_lp(file_model) {
            Ok(_) => {
                match Command::new(&self.command_name).arg(format!("ResultFile={}", self.temp_solution_file)).arg(file_model).output() {
                    Ok(r) => {
                        let mut status = Status::SubOptimal;
                        if String::from_utf8(r.stdout).expect("").contains("Optimal solution found") {
                            status = Status::Optimal;
                        }
                        if r.status.success() {
                            let (_, res) = try!(self.read_solution());
                            Ok((status, res))
                        } else {
                            Err(r.status.to_string())
                        }
                    },
                    Err(_) => Err(format!("Error running the {} solver", self.name)),
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

impl SolverTrait for CbcSolver {
    type P = LpProblem;
    fn run(&self, problem: &Self::P) -> Result<(Status, HashMap<String,f32>), String> {

        let file_model = &format!("{}.lp", problem.name);

        match problem.write_lp(file_model) {
            Ok(_) => {
                match Command::new(&self.command_name).arg(file_model).arg("solve").arg("solution").arg(&self.temp_solution_file).output() {
                    Ok(r) => {
                        if r.status.success(){
                            self.read_solution()
                        }else{
                            Err(r.status.to_string())
                        }
                    },
                    Err(_) => Err(format!("Error running the {} solver", self.name)),
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

impl SolverTrait for GlpkSolver {
    type P = LpProblem;
    fn run(&self, problem: &Self::P) -> Result<(Status, HashMap<String,f32>), String> {

        let file_model = &format!("{}.lp", problem.name);

        match problem.write_lp(file_model) {
            Ok(_) => {
                match Command::new(&self.command_name).arg("--lp").arg(file_model).arg("-o").arg(&self.temp_solution_file).output() {
                    Ok(r) => {
                        if r.status.success() {
                            self.read_solution()
                        } else {
                            Err(r.status.to_string())
                        }
                    },
                    Err(_) => Err(format!("Error running the {} solver", self.name)),
                }
            },
            Err(e) => Err(e.to_string()),
        }
    }
}
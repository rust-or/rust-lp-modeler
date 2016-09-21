use std::fs;
use std::fs::{File};
use std::io::prelude::*;
use std::process::Command;
use std::collections::HashMap;
use std::io::BufReader;


#[derive(Debug, PartialEq)]
pub enum Status {
    Optimal,
    SubOptimal,
    Infeasible,
    Unbounded,
    NotSolved,
}

pub trait SolverTrait {
    fn name(&self) -> String;
    fn command_name(&self) -> String;
    fn run_solver(&self, file_model: &str, file_solution: &str) -> Result<Option<Status>, String>;
    fn read_specific_solution(&self, f: &File) -> Result<(Status, HashMap<String, f32>), String>;
    fn read_solution(&self, file_solution: &str) ->  Result<(Status, HashMap<String,f32>), String> {
        match File::open(file_solution) {
            Ok(f) => {
                let res = try!(self.read_specific_solution(&f));
                let _ = fs::remove_file(file_solution);
                Ok(res)
            },
            Err(_) => return Err("Cannot open file".to_string())
        }
    }
}

pub struct GurobiSolver {}
pub struct CbcSolver {}

impl SolverTrait for GurobiSolver {
    fn name(&self) -> String {
        "gurobi".to_string()
    }
    fn command_name(&self) -> String {
        "gurobi_cl".to_string()
    }
    fn run_solver(&self, file_model: &str, file_solution: &str) -> Result<Option<Status>, String> {
        match Command::new(self.command_name()).arg(format!("ResultFile={}", file_solution)).arg(file_model).output() {
            Ok(r) => {
                let mut status = Status::SubOptimal;
                if String::from_utf8(r.stdout).expect("").contains("Optimal solution found") {
                    status = Status::Optimal;
                }
                if r.status.success() {
                    Ok(Some(status))
                } else {
                    Err(r.status.to_string())
                }
            },
            Err(_) => Err(format!("Error running the {} solver", self.name())),
        }
    }

    fn read_specific_solution(&self, f: &File) -> Result<(Status, HashMap<String, f32>), String> {

        let mut vars_value: HashMap<_,_> = HashMap::new();

        let mut file = BufReader::new(f);
        let mut buffer = String::new();
        let _ = file.read_line(&mut buffer);

        if let Some(_) = buffer.split(" ").next() {
            for line in file.lines() {
                let l = line.unwrap();
                let result_line: Vec<_> = l.split_whitespace().collect();
                if result_line.len() == 2 {
                    match result_line[1].parse::<f32>() {
                        Ok(n) => {
                            vars_value.insert(result_line[0].to_string(), n);
                        },
                        Err(e) => return Err(format!("{}", e.to_string()))
                    }
                }else{
                    return Err("Incorrect solution format".to_string())
                }
            }
        }else{
            return Err("Incorrect solution format".to_string())
        }
        Ok((Status::Optimal, vars_value))
    }
}

impl SolverTrait for CbcSolver {
    fn name(&self) -> String {
        "cbc".to_string()
    }
    fn command_name(&self) -> String {
        "cbc".to_string()
    }
    fn run_solver(&self, file_model: &str, file_solution: &str) -> Result<Option<Status>, String> {
        match Command::new(self.command_name()).arg(file_model).arg("solve").arg("solution").arg(file_solution).output() {
            Ok(r) => {
                if r.status.success(){
                    Ok(None)
                }else{
                    Err(r.status.to_string())
                }
            },
            Err(_) => Err(format!("Error running the {} solver", self.name())),
        }
    }

    fn read_specific_solution(&self, f: &File) -> Result<(Status, HashMap<String, f32>), String> {

        let mut vars_value: HashMap<_,_> = HashMap::new();

        let mut file = BufReader::new(f);
        let mut buffer = String::new();
        let _ = file.read_line(&mut buffer);
        let mut status = Status::SubOptimal;

        if let Some(status_line) = buffer.split(" ").next() {
            if status_line.contains("Optimal") {
                status = Status::Optimal;
            }
            for line in file.lines() {
                let l = line.unwrap();
                let result_line: Vec<_> = l.split_whitespace().collect();
                if result_line.len() == 4 {
                    match result_line[2].parse::<f32>() {
                        Ok(n) => {
                            vars_value.insert(result_line[1].to_string(), n);
                        },
                        Err(e) => return Err(format!("{}", e.to_string()))
                    }
                }else{
                    return Err("Incorrect solution format".to_string())
                }
            }
        }else{
            return Err("Incorrect solution format".to_string())
        }
        Ok((status, vars_value))
    }
}


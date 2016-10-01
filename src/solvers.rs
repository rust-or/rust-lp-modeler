use std;
use std::fs;
use std::io::prelude::*;
use std::process::Command;
use std::collections::HashMap;
use std::io::BufReader;
use problem::{LpProblem, LpObjective};
use problem::{LpFileFormat};
use std::fs::File;


#[derive(Debug, PartialEq)]
pub enum Status {
    Optimal,
    SubOptimal,
    Infeasible,
    Unbounded,
    NotSolved,
}

// TODO: Let only run_solver which run, read and send the solution
pub trait SolverTrait {
    fn run_solver(&self, problem: &LpProblem) -> Result<(Status, HashMap<String,f32>), String>;
}

pub trait LinearSolverTrait : SolverTrait {
    fn write_lp(&self, problem: &LpProblem, file_model: &str) -> std::io::Result<()>  {
        let mut buffer = try!(File::create(file_model));
        try!(buffer.write(problem.to_lp_file_format().as_bytes()));
        Ok(())
    }
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

    fn read_solution(&self) -> Result<(Status, HashMap<String, f32>), String> {
        fn read_specific_solution(f: &File) -> Result<(Status, HashMap<String, f32>), String> {
            let mut vars_value: HashMap<_, _> = HashMap::new();

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
                    } else {
                        return Err("Incorrect solution format".to_string())
                    }
                }
            } else {
                return Err("Incorrect solution format".to_string())
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


impl LinearSolverTrait for GurobiSolver {}
impl SolverTrait for GurobiSolver {
    fn run_solver(&self, problem: &LpProblem) -> Result<(Status, HashMap<String,f32>), String> {

        use std::fs::File;
        use std::io::prelude::*;
        let file_model = "test.lp";

        match self.write_lp(problem, file_model) {
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

impl LinearSolverTrait for CbcSolver {}
impl SolverTrait for CbcSolver {
    fn run_solver(&self, problem: &LpProblem) -> Result<(Status, HashMap<String,f32>), String> {

        use std::fs::File;
        use std::io::prelude::*;
        let file_model = "test.lp";

        match self.write_lp(problem, file_model) {
            Ok(_) => {
                match Command::new(&self.command_name).arg(format!("ResultFile={}", self.temp_solution_file)).arg(file_model).output() {
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


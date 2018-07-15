extern crate lp_modeler;

use std::process;
use std::collections::HashMap;

use lp_modeler::variables::*;
use lp_modeler::variables::LpExpression::*;
use lp_modeler::operations::LpOperations;
use lp_modeler::problem::{LpObjective, LpProblem, LpFileFormat};
use lp_modeler::solvers::{SolverTrait, CbcSolver};

#[test]
fn test_readme_example_1() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);
    problem += 10.0 * a + 20.0 * b;

    problem += (500*a + 1200*b + 1500*c).le(10000);
    problem += (a).le(b);

    let solver = CbcSolver::new();

    match solver.run(&problem) {
        Ok((status, res)) => {
            println!("Status {:?}", status);
            for (name, value) in res.iter() {
                println!("value of {} = {}", name, value);
            }
        },
        Err(msg) => println!("{}", msg),
    }

    let output1 = "\\ One Problem

Maximize
  obj: 10 a + 20 b

Subject To
  c1: 500 a + 1200 b + 1500 c <= 10000
  c2: a - b <= 0

".to_string();
    let output2 = problem.to_lp_file_format();
    let output2 = output2.split("Generals").collect::<Vec<&str>>();
    let output2 = output2[0];
    assert_eq!(output1, output2);
}

#[test]
fn test_readme_example_2() {
    // Problem Data
    let men = vec!["A", "B", "C"];
    let women = vec!["D", "E", "F"];
    let scores: HashMap<(&'static str, &'static str), f32> = [
        (("A", "D"), 50.0),
        (("A", "E"), 75.0),
        (("A", "F"), 75.0),
        (("B", "D"), 60.0),
        (("B", "E"), 95.0),
        (("B", "F"), 80.0),
        (("C", "D"), 60.0),
        (("C", "E"), 70.0),
        (("C", "F"), 80.0),
        ].iter().cloned().collect();

    // Define Problem
    let mut problem = LpProblem::new("Matchmaking", LpObjective::Maximize);

    // Define Variables
    let mut vars = HashMap::new();
    for m in &men{
        for w in &women{
            vars.insert((m, w), LpBinary::new(&format!("{}_{}", m, w)));
        }
    }

    // Define Objective Function
    let mut obj_vec: Vec<LpExpression> = Vec::new();
    for (&(m, w), var) in &vars{
        let obj_coef = scores.get(&(m, w)).unwrap();
        obj_vec.push(*obj_coef * var);
    }
    problem += lp_sum(&obj_vec);

    // Define Constraints
    // Constraint 1: Each man must be assigned to exactly one woman
    for m in &men{
        let mut constr_vec = Vec::new();

        for w in &women{
            constr_vec.push(1.0 * vars.get(&(m, w)).unwrap());
        }

        problem += lp_sum(&constr_vec).equal(1);
    }

    // Constraint 2: Each woman must be assigned to exactly one man
    for w in &women{
        let mut constr_vec = Vec::new();

        for m in &men{
            constr_vec.push(1.0 * vars.get(&(m, w)).unwrap());
        }

        problem += lp_sum(&constr_vec).equal(1);
    }

    // Optionally write to file
    // let result = problem.write_lp("problem.lp");
    // match result{
    //     Ok(_) => println!("Written to file"),
    //     Err(msg) => println!("{}", msg)
    // }

    // Run Solver
    let solver = CbcSolver::new();
    let result = solver.run(&problem);

    // Terminate if error, or assign status & variable values
    assert!(result.is_ok(), result.unwrap_err());
    let (solver_status, var_values) = result.unwrap();

    // Compute final objective function value
    let mut obj_value = 0f32;
    for (&(m, w), var) in &vars{
        let obj_coef = scores.get(&(m, w)).unwrap();
        let var_value = var_values.get(&var.name).unwrap();
        
        obj_value += obj_coef * var_value;
    }

    // Print output
    println!("Status: {:?}", solver_status);
    println!("Objective Value: {}", obj_value);
    // println!("{:?}", var_values);
    for (var_name, var_value) in &var_values{
        let int_var_value = *var_value as u32;
        if int_var_value == 1{
            println!("{} = {}", var_name, int_var_value);
        }
    }
}
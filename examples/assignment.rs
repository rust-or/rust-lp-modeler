extern crate lp_modeler;
#[macro_use] extern crate maplit;

use std::collections::HashMap;

use lp_modeler::dsl::*;
use lp_modeler::solvers::{SolverTrait, CbcSolver};

fn main() {
    // Problem Data
    let men = vec!["A", "B", "C"];
    let women = vec!["D", "E", "F"];
    let compat_scores = hashmap!{
    ("A", "D") => 50.0,
    ("A", "E") => 75.0,
    ("A", "F") => 75.0,
    ("B", "D") => 60.0,
    ("B", "E") => 95.0,
    ("B", "F") => 80.0,
    ("C", "D") => 60.0,
    ("C", "E") => 70.0,
    ("C", "F") => 80.0,
    };

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
    for (&(&m, &w), var) in &vars{
        let &obj_coef = compat_scores.get(&(m, w)).unwrap();
        obj_vec.push(obj_coef * var);
    }
    problem += lp_sum(&obj_vec);

    // Define Constraints
    // Constraint 1: Each man must be assigned to exactly one woman
    for m in &men{
        problem += sum(&women, |w| vars.get(&(m,w)).unwrap() ).equal(1);
    }

    // Constraint 2: Each woman must be assigned to exactly one man
    for w in &women{
        problem += sum(&men, |m| vars.get(&(m,w)).unwrap() ).equal(1);
    }

    // Run Solver
    let solver = CbcSolver::new();
    let result = solver.run(&problem);

    // Terminate if error, or assign status & variable values
    assert!(result.is_ok(), result.unwrap_err());
    let (solver_status, var_values) = result.unwrap();

    // Compute final objective function value
    let mut obj_value = 0f32;
    for (&(&m, &w), var) in &vars{
        let obj_coef = compat_scores.get(&(m, w)).unwrap();
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


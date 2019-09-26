extern crate lp_modeler;

use std::collections::HashMap;

use lp_modeler::dsl::*;
use lp_modeler::solvers::{SolverTrait, CbcSolver};

fn main() {
    // Problem Data
    let men = vec!["A", "B", "C"];
    let women = vec!["D", "E", "F"];
    let compatibility_score: HashMap<(&str, &str),f32> = vec![
        (("A", "D"), 50.0),
        (("A", "E"), 75.0),
        (("A", "F"), 75.0),
        (("B", "D"), 60.0),
        (("B", "E"), 95.0),
        (("B", "F"), 80.0),
        (("C", "D"), 60.0),
        (("C", "E"), 70.0),
        (("C", "F"), 80.0),
    ].into_iter().collect();

    // Define Problem
    let mut problem = LpProblem::new("Matchmaking", LpObjective::Maximize);

    // Define Variables
    let vars: HashMap<(&str,&str), LpBinary> =
        men.iter()
            .flat_map(|&m| women.iter()
            .map(move |&w| {
                let key = (m,w);
                let value = LpBinary::new(&format!("{}_{}", m,w));
                (key, value)
            }))
            .collect();

    // Define Objective Function
    let obj_vec: Vec<LpExpression> = {
       vars.iter().map( |(&(m,w), bin)| {
           let &coef = compatibility_score.get(&(m, w)).unwrap();
           coef * bin
       } )
    }.collect();
    problem += obj_vec.sum();

    // Define Constraints
    // - constraint 1: Each man must be assigned to exactly one woman
    for &m in &men{
        problem += sum(&women, |&w| vars.get(&(m,w)).unwrap() ).equal(1);
    }

    // - constraint 2: Each woman must be assigned to exactly one man
    for &w in &women{
        problem += sum(&men, |&m| vars.get(&(m,w)).unwrap() ).equal(1);
    }

    // Run Solver
    let solver = CbcSolver::new();
    let result = solver.run(&problem);

    // Compute final objective function value
    // (terminate if error, or assign status & variable values)
    assert!(result.is_ok(), result.unwrap_err());
    let (solver_status, var_values) = result.unwrap();
    let mut obj_value = 0f32;
    for (&(m, w), var) in &vars{
        let obj_coef = compatibility_score.get(&(m, w)).unwrap();
        let var_value = var_values.get(&var.name).unwrap();

        obj_value += obj_coef * var_value;
    }

    // Print output
    println!("Status: {:?}", solver_status);
    println!("Objective Value: {}", obj_value);
    for (var_name, var_value) in &var_values{
        let int_var_value = *var_value as u32;
        if int_var_value == 1{
            println!("{} = {}", var_name, int_var_value);
        }
    }

}


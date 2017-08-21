extern crate lp_modeler;

use lp_modeler::operations::LpOperations;
use lp_modeler::problem::{LpProblem, LpObjective, LpFileFormat};
use lp_modeler::solvers::*;
use lp_modeler::variables::{LpInteger, LpBinary, LpExpression, LpConstraint};
use lp_modeler::variables::LpExpression::LitVal;

fn main() {


    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    let mut problem = LpProblem::new("Problem", LpObjective::Maximize);

    problem += 10.0 * a + 20.0 * b;

    problem += (a*400).le(10000);
    problem += (400*a).le(10000);
    problem += (a + b*2 + c).le(10);
    problem += (a).le(b);

    let input_coins: Vec<i64> = vec![150, 200, 300, 5000, 20323];
    let output_coins = vec![203];
    let fee_rate = 300;

    // constants

    let bytes_per_input = 92;
    let transaction_fixed_bytes = 12;
    let bytes_per_output = 34;


    // problem below




    let input_expressions: Vec<(LpBinary, LpExpression)> = input_coins.iter().enumerate().map(|(i,coin)| {
        let constraint_name = format!("contains{}", i);
        let ref constraint_variable = LpBinary::new(constraint_name.as_str());
        (constraint_variable.clone() , *coin * constraint_variable)
    }).collect();



    // Suboptimal instead of unfeasible
    // problem += (a+b).equal(10);

    // let solver = GurobiSolver
    // solver <<= BaseDirectory("/opt/gurobi1.2/...")
    // solver <<= Config().arg("-thread 2").arg("...")

    let _ = problem.write_lp("toto.lp");

    let solver = GurobiSolver::new();
    //let solver = CbcSolver::new();

    match solver.run(&problem) {
        Ok((status, res)) => {
            println!("Status {:?}", status);
            for (name, value) in res.iter() {
                println!("value of {} = {}", name, value);
            }
        },
        Err(msg) => println!("{}", msg),
    }

}

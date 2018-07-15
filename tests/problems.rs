extern crate lp_modeler;

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

extern crate lp_modeler;

use lp_modeler::operations::{LpOperations};
use lp_modeler::problem::{LpProblem, LpObjective};
use lp_modeler::solvers::*;
use lp_modeler::variables::{LpInteger};

fn main() {


    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    let mut problem = LpProblem::new("Problem", LpObjective::Maximize);

    // TODO: mult lit on the left side for expression obj. func.
    // problem += a * 10; not readable FIX:
    problem += 10.0 * a + 20.0 * b + 5 * c;

    problem += (500*a + 1200*b + 1500*c).le(10000);
    problem += (a).le(b);

    match problem.solve(GurobiSolver) {
        Ok((status, res)) => {
            println!("Status {:?}", status);
            for (name, value) in res.iter() {
                println!("value of {} = {}", name, value);
            }
        },
        Err(msg) => println!("{}", msg),
    }

}

extern crate lp_modeler;

use lp_modeler::operations::{LpOperations};
use lp_modeler::problem::{LpProblem, LpObjective};
use lp_modeler::variables::{LpVariable, LpType, LpExpression};
use lp_modeler::variables::LpExpression::*;

fn main() {


    let ref a = LpVariable::new("a", LpType::Integer);
    let ref b = LpVariable::new("b", LpType::Integer);

    let mut problem = LpProblem::new("Problem", LpObjective::Maximize);

    problem += 10.0 * a + 20.0 * b;

    problem += (500*a + 1200*b).le(10000);
    problem += (a).le(b);

    if let Ok(..) = problem.write_lp("test.lp") {
        println!("File exported");
        problem.solve();
    }

    //TODO: With solver: check names of variables (unique)
}

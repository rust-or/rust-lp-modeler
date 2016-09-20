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
    problem += (500.0 * a + 1200.0 * b + 1000.0 + 1000.0).le(10000.0);
    problem += (a).le(b);
    problem += (LitVal(12.0) + LitVal(33.0)).ge(a);
    problem += (-b).le(0.0);
    problem += (a + -b).le(0.0);
    problem += (500.0 * a + 1200.0 * b).le(10000.0);
    problem += (a - b).le(0.0);
    problem += (a + 2.0 * b + 66.0 + 14.0 - 2.0*b - 10.0).le(12.0+b + 2.0);

    if let Ok(..) = problem.write_lp("test.lp") {
        println!("File exported");
    }

    //TODO: With solver: check names of variables (unique)
}

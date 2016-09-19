extern crate lp_modeler;

use lp_modeler::operations::{LpOperations};
use lp_modeler::problem::{LpProblem, LpObjective};
use lp_modeler::variables::{LpVariable, LpType, LpExpression};
use lp_modeler::variables::LpExpression::*;

fn main() {


    let ref a = LpVariable::new("a", LpType::Integer);
    let ref b = LpVariable::new("b", LpType::Integer);

    let mut problem = LpProblem::new("Problem", LpObjective::Maximize);

    problem += 10 * a + 20 * b;
    problem += (500 * a + 1200 * b + 1000 + 1000).le(10000);
    problem += (a).le(b);
    problem += (LitVal(12) + LitVal(33)).ge(a);
    problem += (-b).le(0);
    problem += (a + -b).le(0);
    problem += (500 * a + 1200 * b).le(10000);
    problem += (a - b).le(0);
    problem += (a + 2 * b + 66 + 14 - 2*b - 10).le(12+b + 2);

    if let Ok(..) = problem.write_lp("test.lp") {
        println!("File exported");
    }

    //TODO: With solver: check names of variables (unique)
}

extern crate lp_modeler;

use lp_modeler::operations::{LpOperations};
use lp_modeler::problem::{LpProblem, LpObjective};
use lp_modeler::variables::{LpVariable, LpType, LpExpression};
use lp_modeler::variables::LpExpression::*;

fn main() {


    let ref a = LpVariable::new("a", LpType::Integer);
    let ref b = LpVariable::new("b", LpType::Integer);
    let ref c = LpVariable::new("c", LpType::Integer);

    let mut problem = LpProblem::new("Problem", LpObjective::Maximize);

    problem += 10.0 * a + 20.0 * b + 5 * c;

    problem += (500*a + 1200*b + 1500*c).le(10000);
    problem += (a).le(b);
    problem += b.le(4);
    problem += c.ge(1);

    let res = problem.solve();
    problem.write_lp("toto.lp");
    println!("{:?}", res);

    for r in res.iter() {
        println!("value of {} = {}", r.0, r.1);
    }


}

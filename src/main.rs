extern crate lp_modeler;

use lp_modeler::variables::*;
use lp_modeler::problem::*;

fn main() {

    let b = LpVariable::new("toto", LpType::Binary);
    let c = LpVariable::new("toto", LpType::Binary);
    //let c = ContinuousVariable::new("t");
    //let c1 = ContinuousVariable::new("t");

    //let prob = LpProblem::new("p1", Objective::Maximize);


//    let l1 = LpVariable::new("l1".to_string(), Category::Binary);
    println!("{:?}", b + c);
    println!("{:?}", 2 * b + c + c);
    println!("{:?}", 2 * b + 2 * c);
    println!("{:?}", 2 * b + 2 * c + 3 * b);
    //println!("{:?}", b * 3);

    //println!("{:?}", b + c);
}

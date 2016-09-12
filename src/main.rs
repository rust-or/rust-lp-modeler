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
    println!("*{:?}", 2 * b + 2);
    println!("{:?}", (2 * b + 2 * c + 3 * b).gt(b + c));

    println!("{:?}", (2 * b).gt(b));
    println!("{:?}", c.gt(b));
    println!("-{:?}", c.gt(3));
    println!("{:?}", c.eq(2 * b));
    println!("{:?}", (b + c).eq(b));

    /*
    let mut p = LpProblem::new("Coucou", Objective::Maximize);
    p += (b + c).gt(b);
    println!("{:?}", p);

    p += b ;
    println!("{:?}", p);

    // in python with pulp : lpSum([x for x in collections]) > 12
    let c = vec!(b, c);
    p += lpSum(&c).gt(b);
    */
    /*
    p += 2 * lpSum(c) > 1;
    p += lpSum(2 * c) > 1;
    println!("\n\n\n");
    println!("{:?}", lpSum(&c).gt(b));
    println!("\n\n\n");
    println!("{:?}", p);
    */


    //TODO: With solver: check names of variables (unique)
}

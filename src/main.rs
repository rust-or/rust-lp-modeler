extern crate lp_modeler;

use lp_modeler::variables::*;
use lp_modeler::problem::*;

fn main() {

    let ref b = LpVariable::new("toto", LpType::Binary);
    let ref c = LpVariable::new("toto", LpType::Binary);
    //let c = ContinuousVariable::new("t");
    //let c1 = ContinuousVariable::new("t");

    //let prob = LpProblem::new("p1", Objective::Maximize);


//    let l1 = LpVariable::new("l1".to_string(), Category::Binary);
    println!("{:?}", b + c);
    println!("{:?}", 2 * c);
    println!("{:?}", 2 * b + c);
    println!("{:?}", 2 * b + 2 * c);
    println!("*{:?}", 2 * b + 2);
    println!("*{:?}", 2 + b);
    println!("*{:?}", b + 2);
    println!("*{:?}", 2 * (b + c));
    /*
    println!("{:?}", (2 * b).gt(b));
    println!("{:?}", (2 * b + 2 * c + 3 * b).gt(b + c));

    println!("{:?}", c.gt(b));
    println!("-{:?}", c.gt(3));
    println!("{:?}", c.eq(2 * b));
    println!("{:?}", (b + c).eq(b));

    let ref expr = c + b;
    let ref e2 = expr + expr;
    let e3 = e2 + expr;
    println!("** {:?}", expr);
    println!("** {:?}", e3);
    */
    /*
    let mut p = LpProblem::new("Coucou", Objective::Maximize);
    p += (b + c).gt(b);
    println!("{:?}", p);

    p += b ;
    println!("{:?}", p);

    // in python with pulp : lp_sum([x for x in collections]) > 12
    let ref c = vec!(b, c);
    p += lp_sum(c).gt(b);
    //p += 2 * lp_sum(c) > 1;
    //p += lp_sum(2 * c) > 1;
    println!("\n\n\n");
    println!("{:?}", lp_sum(&c).gt(b));
    println!("\n\n\n");
    println!("{:?}", p);
    */


    //TODO: With solver: check names of variables (unique)
}

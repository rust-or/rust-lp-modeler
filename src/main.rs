extern crate lp_modeler;

use lp_modeler::variables::*;
use lp_modeler::problem::*;

fn main() {

    let ref a = LpVariable::new("a", LpType::Integer)
        .lower_bound(10)
        .upper_bound(20);
    let ref b = LpVariable::new("b", LpType::Integer);
    let ref c = LpVariable::new("c", LpType::Integer);
    let ref d = LpVariable::new("d", LpType::Binary);
    let ref e = LpVariable::new("e", LpType::Integer).lower_bound(100);
    let ref f = LpVariable::new("f", LpType::Continuous).lower_bound(50);

    println!("{:?}", a + b);
    println!("{:?}", 2 * c);
    println!("{:?}", 2 * a + b);
    println!("{:?}", 2 * d + 2 * b);
    println!("*{:?}", 2 * a + 2);
    println!("{:?}", 2 + d);
    println!("{:?}", a + 2);
    println!("{:?}", 2 * (a + b));

    println!("{:?}", (2 * a).gt(d));
    println!("{:?}",  a.gt(d));
    println!("{:?}", (2 * a + 2 * d + 3 * a).gt(a + b));

    println!("{:?}", b.gt(a));
    println!("{:?}", b.gt(3));
    println!("{:?}", b.equal(2 * a));
    println!("{:?}", (a + b).equal(a));

    let ref expr = b + a;
    let ref e2 = expr + expr;
    let e3 = e2 + expr;
    println!("** {:?}", expr);
    println!("** {:?}", e3);
    let mut problem = LpProblem::new("Coucou", Objective::Maximize);
    problem += (a + b).gt(a);
    println!("{:?}", problem);

    problem += (a + b + c + d + e).ge(e);

    problem += a + 2 ;
    problem += a + 2 * b;
    println!("{:?}", problem);

    // in python with pulp : lp_sum([x for x in collections]) > 12
    let ref c = vec!(b, c);
    problem += lp_sum(c).equal(a);
    let ref c = vec!(b + 2);
    problem += lp_sum(c).gt(a);
    let ref c = vec!(2 * b + 2, 2 * b, b.clone());
    problem += lp_sum(c).gt(a);
    println!("\n\n\n");
    println!("{:?}", lp_sum(c).gt(a));
    println!("\n\n\n");
    println!("{:?}", problem);

    problem.solve();

    problem.write_lp();

    //let z = a.clone();
    //println!("{}", z == a);


    //TODO: With solver: check names of variables (unique)
}

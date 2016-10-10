use variables::*;
use variables::LpExpression::*;
use std::rc::Rc;
use operations::LpOperations;
use problem::LpFileFormat;


#[test]
fn expressions_creation() {
    let ref a = LpInteger::new("a")
        .lower_bound(10.0)
        .upper_bound(20.0);
    let ref b = LpInteger::new("b");

    assert_eq!(a + b, AddExpr(Rc::new(ConsInt(a.clone())), Rc::new(ConsInt(b.clone()))));
}

#[test]
fn expressions_to_lp_file_format() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    assert_eq!((a + 2*b + c).to_lp_file_format(), "a + 2 b + c");
    assert_eq!((a + b*2 + c).to_lp_file_format(), "a + 2 b + c");
    assert_eq!((a + b*2 + 3 * 2 * c).to_lp_file_format(), "a + 2 b + 6 c");
    assert_eq!((a + 2).to_lp_file_format(), "a + 2");
    assert_eq!((2*a + 2*b -4*c).to_lp_file_format(), "2 a + 2 b - 4 c");
    assert_eq!((-2*a).to_lp_file_format(), "-2 a");
}


#[test]
fn constraints_to_lp_file_format() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    assert_eq!((a+b).equal(10).to_lp_file_format(), "a + b = 10");
    assert_eq!((2*a + b).ge(10).to_lp_file_format(), "2 a + b >= 10");
    assert_eq!((2*a + b + 20).ge(c).to_lp_file_format(), "2 a + b - c >= -20");
    assert_eq!((-a).ge(10).to_lp_file_format(), "-a >= 10");
    assert_eq!((2*a - 20 + b).ge(-c).to_lp_file_format(), "2 a + b + c >= 20");
}

#[test]
fn test_readme_example() {
    use problem::{LpObjective, LpProblem};
    use operations::{LpOperations};
    use variables::LpInteger;
    use solvers::{SolverTrait, CbcSolver};

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

    let output = "\\ One Problem

Maximize
  10 a + 20 b

Subject To
  c1: 500 a + 1200 b + 1500 c <= 10000
  c2: a - b <= 0

Generals
  a c b

End";
    assert_eq!(problem.to_lp_file_format(), output);
}

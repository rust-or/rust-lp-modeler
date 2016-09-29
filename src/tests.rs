use variables::*;
use variables::LpExpression::*;
use std::rc::Rc;
use operations::LpOperations;


#[test]
fn expressions_creation() {
    let ref a = LpInteger::new("a")
        .lower_bound(10.0)
        .upper_bound(20.0);
    let ref b = LpInteger::new("b");

    assert_eq!(a + b, AddExpr(Rc::new(ConsInt(a.clone())), Rc::new(ConsInt(b.clone()))));
}

#[test]
fn expressions_to_string() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    assert_eq!((a + 2*b + c).to_string(), "a + 2 b + c");
    assert_eq!((a + b*2 + c).to_string(), "a + 2 b + c");
    assert_eq!((a + b*2 + 3 * 2 * c).to_string(), "a + 2 b + 6 c");
    assert_eq!((a + 2).to_string(), "a + 2");
    assert_eq!((2*a + 2*b -4*c).to_string(), "2 a + 2 b - 4 c");
    assert_eq!((-2*a).to_string(), "-2 a");
}


#[test]
fn constraints_to_string() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    assert_eq!((a+b).equal(10).to_string(), "a + b = 10");
    assert_eq!((2*a + b).ge(10).to_string(), "2 a + b >= 10");
    assert_eq!((2*a + b + 20).ge(c).to_string(), "2 a + b - c >= -20");
    assert_eq!((-a).ge(10).to_string(), "-a >= 10");
    assert_eq!((2*a - 20 + b).ge(-c).to_string(), "2 a + b + c >= 20");
}

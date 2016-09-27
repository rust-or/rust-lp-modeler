use variables::*;
use variables::LpExpression::*;
use problem::*;
use std::rc::Rc;


#[test]
fn expressions_creation() {
    let ref a = LpInteger::new("a")
        .lower_bound(10.0)
        .upper_bound(20.0);
    let ref b = LpInteger::new("b");

    assert_eq!(a + b, AddExpr(Rc::new(ConsInt(a.clone())), Rc::new(ConsInt(b.clone()))));
}


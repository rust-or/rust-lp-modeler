extern crate lp_modeler;

use lp_modeler::dsl::LpOperations;
use lp_modeler::dsl::LpFileFormat;
use lp_modeler::dsl::LpExpression::*;
use lp_modeler::dsl::*;

#[test]
fn expressions_creation() {
    let ref a = LpInteger::new("a").lower_bound(10.0).upper_bound(20.0);
    let ref b = LpInteger::new("b");

    assert_eq!(
        a + b,
        AddExpr(Box::new(ConsInt(a.clone())), Box::new(ConsInt(b.clone())))
    );
}

#[test]
fn distributivity() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    let test = a * (2 + b) * 3;
    let test2 = test.clone();
    assert_eq!((2 * (2 + a)).to_lp_file_format(), "2 a + 4");
    assert_eq!(
        ((2 + a) * (2 + b)).to_lp_file_format(),
        "2 a + 2 b + b a + 4"
    );
    assert_eq!(test.to_lp_file_format(), "6 a + 3 a b");
    assert_eq!((10 * test).to_lp_file_format(), "60 a + 30 a b");
    assert_eq!(
        ((c + 10) * test2).to_lp_file_format(),
        "6 c a + 3 c a b + 60 a + 30 a b"
    );
}

#[test]
fn associativity() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    assert_eq!((a + (b + 2)).to_lp_file_format(), "a + b + 2");
    assert_eq!(((a + b) + 2).to_lp_file_format(), "a + b + 2");

    assert_eq!((a + (b - 2)).to_lp_file_format(), "a + b - 2");
    assert_eq!(((a + b) - 2).to_lp_file_format(), "a + b - 2");

    assert_eq!((a - (b + 2)).to_lp_file_format(), "a - b - 2");
    assert_eq!(((a - b) + 2).to_lp_file_format(), "a - b + 2");

    assert_eq!((a - (b - 2)).to_lp_file_format(), "a - b + 2");
    assert_eq!(((a - b) - 2).to_lp_file_format(), "a - b - 2");

    assert_eq!((a - (b - 2) + c).to_lp_file_format(), "a - b + c + 2");
    assert_eq!(((a - b) - 2 + c).to_lp_file_format(), "a - b + c - 2");
}

#[test]
fn literal_first_with_accumulation() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    assert_eq!(
        (a + 1 + b + 2 + c + 3 + a + 4).to_lp_file_format(),
        "a + b + c + a + 10"
    );
    assert_eq!(
        (a - 1 + b - 2 - c + 3 + a - 4).to_lp_file_format(),
        "a + b - c + a - 4"
    );
    assert_eq!(
        (a + b + 1 - c - a - 3).to_lp_file_format(),
        "a + b - c - a - 2"
    );
    assert_eq!(
        (a + b + (c - 1) * 2 - a - 3).to_lp_file_format(),
        "a + b + 2 c - a - 5"
    );
    assert_eq!(
        (a + b + (1 - c) * 2 - a - 3).to_lp_file_format(),
        "a + b - 2 c - a - 1"
    );
    assert_eq!((2 * (a + 5)).to_lp_file_format(), "2 a + 10");
    assert_eq!(
        ((2 + b) * (a + 5)).to_lp_file_format(),
        "2 a + a b + 5 b + 10"
    );
    assert_eq!((2 + (a + b) + 3).to_lp_file_format(), "a + b + 5");
}

#[test]
fn trivial_rules() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");

    assert_eq!(((a + b) * 0).to_lp_file_format(), "0");
    assert_eq!((0 * (a + b)).to_lp_file_format(), "0");
    assert_eq!((0 * a * b * 3).to_lp_file_format(), "0");
    assert_eq!(((a + b) + 0).to_lp_file_format(), "a + b");
    assert_eq!((0 + (a + b) + 0).to_lp_file_format(), "a + b");
    assert_eq!((0 + (a + b)).to_lp_file_format(), "a + b");
    assert_eq!(((a + b) - 0).to_lp_file_format(), "a + b");
    assert_eq!((0 - (a + b)).to_lp_file_format(), "-a - b");
    assert_eq!((0 + (a + b)).to_lp_file_format(), "a + b");
}

#[test]
fn expressions_to_lp_file_format() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    // Expressions
    assert_eq!((a + 2 * b + c).to_lp_file_format(), "a + 2 b + c");
    assert_eq!((a + b * 2 + c).to_lp_file_format(), "a + 2 b + c");
    assert_eq!((a + b * 2 + 3 * 2 * c).to_lp_file_format(), "a + 2 b + 6 c");
    assert_eq!((a + 2).to_lp_file_format(), "a + 2");
    assert_eq!(
        (2 * a + 2 * b - 4 * c).to_lp_file_format(),
        "2 a + 2 b - 4 c"
    );
    assert_eq!((-2 * a).to_lp_file_format(), "-2 a");

    // Constraints
    assert_eq!((a + b).equal(10).to_lp_file_format(), "a + b = 10");
    assert_eq!((2 * a + b).ge(10).to_lp_file_format(), "2 a + b >= 10");
    assert_eq!(
        (2 * a + b + 20).ge(c).to_lp_file_format(),
        "2 a + b - c >= -20"
    );
    assert_eq!((-a).ge(10).to_lp_file_format(), "-a >= 10");
    assert_eq!(
        (2 * a - 20 + b).ge(-c).to_lp_file_format(),
        "2 a + b + c >= 20"
    );
    assert_eq!((a + b + 10).ge(0).to_lp_file_format(), "a + b >= -10");
    assert_eq!(
        (3 * (a + b + 10)).le(0).to_lp_file_format(),
        "3 a + 3 b <= -30"
    );
    assert_eq!(
        (3 * (a + b + 10)).le(a + b).to_lp_file_format(),
        "3 a + 3 b - a - b <= -30"
    );
}

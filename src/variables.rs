/// # Module variables
use self::LpExpression::*;
use problem::LpFileFormat;
use std::convert::Into;
use util::is_zero;
use variables::Constraint::*;

pub trait BoundableLp: PartialEq + Clone {
    fn lower_bound(&self, lw: f32) -> Self;
    fn upper_bound(&self, up: f32) -> Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpBinary {
    pub name: String,
}
impl LpBinary {
    pub fn new(name: &str) -> LpBinary {
        LpBinary {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpInteger {
    pub name: String,
    pub lower_bound: Option<f32>,
    pub upper_bound: Option<f32>,
}
impl LpInteger {
    pub fn new(name: &str) -> LpInteger {
        LpInteger {
            name: name.to_string(),
            lower_bound: None,
            upper_bound: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpContinuous {
    pub name: String,
    pub lower_bound: Option<f32>,
    pub upper_bound: Option<f32>,
}
impl LpContinuous {
    pub fn new(name: &str) -> LpContinuous {
        LpContinuous {
            name: name.to_string(),
            lower_bound: None,
            upper_bound: None,
        }
    }
}

macro_rules! implement_boundable {
    ($lp_type: ident) => {
        impl BoundableLp for $lp_type {
            fn lower_bound(&self, lw: f32) -> $lp_type {
                $lp_type {
                    name: self.name.clone(),
                    lower_bound: Some(lw),
                    upper_bound: self.upper_bound,
                }
            }
            fn upper_bound(&self, up: f32) -> $lp_type {
                $lp_type {
                    name: self.name.clone(),
                    lower_bound: self.lower_bound,
                    upper_bound: Some(up),
                }
            }
        }
    };
}
implement_boundable!(LpInteger);
implement_boundable!(LpContinuous);

/// ADT for Linear Programming Expression
#[derive(Debug, Clone, PartialEq)]
pub enum LpExpression {
    ConsInt(LpInteger),
    ConsBin(LpBinary),
    ConsCont(LpContinuous),
    MulExpr(Box<LpExpression>, Box<LpExpression>),
    AddExpr(Box<LpExpression>, Box<LpExpression>),
    SubExpr(Box<LpExpression>, Box<LpExpression>),
    LitVal(f32),
    EmptyExpr,
}

impl LpExpression {
    /// Fix the numeric operand in a multiplication in an expression
    /// c * 4 must be considered as 4 c in a linear formulation lp file
    pub fn normalize(&self) -> LpExpression {
        if let MulExpr(e1, e2) = self {
            if let LitVal(..) = **e1 {
                return self.clone();
            } else {
                if let LitVal(..) = **e2 {
                    return MulExpr(Box::new(*e2.clone()), Box::new(*e1.clone()));
                } else {
                    return MulExpr(Box::new(*e1.clone()), Box::new(*e2.clone()));
                }
            }
        }
        self.clone()
    }
}

fn show(e: &LpExpression, with_parenthesis: bool) -> String {
    let str_left_mult = if with_parenthesis { "(" } else { "" };
    let str_right_mult = if with_parenthesis { ")" } else { "" };
    let str_op_mult = if with_parenthesis { " * " } else { " " };
    match e {
        &LitVal(n) => n.to_string(),
        &AddExpr(ref e1, ref e2) => {
            str_left_mult.to_string()
                + &show(e1, with_parenthesis)
                + " + "
                + &show(e2, with_parenthesis)
                + str_right_mult
        }
        &SubExpr(ref e1, ref e2) => {
            str_left_mult.to_string()
                + &show(e1, with_parenthesis)
                + " - "
                + &show(e2, with_parenthesis)
                + str_right_mult
        }
        &MulExpr(ref e1, ref e2) => {
            let ref deref_e1 = **e1;

            match deref_e1 {
                &LitVal(v) if v == 1.0 => {
                    //e2.to_lp_file_format()
                    str_left_mult.to_string()
                        + &" ".to_string()
                        + &show(e2, with_parenthesis)
                        + str_right_mult
                }
                &LitVal(v) if v == -1.0 => {
                    //"-".to_string() + &e2.to_lp_file_format()
                    str_left_mult.to_string()
                        + &"-".to_string()
                        + &show(e2, with_parenthesis)
                        + str_right_mult
                }
                _ => {
                    str_left_mult.to_string()
                        + &show(e1, with_parenthesis)
                        + str_op_mult
                        + &show(e2, with_parenthesis)
                        + str_right_mult
                }
            }
        }
        &ConsBin(LpBinary { name: ref n, .. }) => n.to_string(),
        &ConsInt(LpInteger { name: ref n, .. }) => n.to_string(),
        &ConsCont(LpContinuous { name: ref n, .. }) => n.to_string(),
        _ => "EmptyExpr!!".to_string(),
    }
}

impl LpFileFormat for LpExpression {
    fn to_lp_file_format(&self) -> String {
        fn formalize_signs(s: String) -> String {
            let mut s = s.clone();
            let mut t = "".to_string();
            while s != t {
                t = s.clone();
                s = s.replace("+ +", "+ ");
                s = s.replace("- +", "- ");
                s = s.replace("+ -", "- ");
                s = s.replace("- -", "+ ");
                s = s.replace("  ", " ");
            }
            s
        }

        formalize_signs(show(&simplify(self), false))
    }
}

#[derive(Debug, Clone)]
pub enum Constraint {
    /* Not supported by solver format files (lp file or mps file) !
    Greater,
    Less,
    */
    GreaterOrEqual,
    LessOrEqual,
    Equal,
}

#[derive(Debug, Clone)]
pub struct LpConstraint(pub LpExpression, pub Constraint, pub LpExpression);

impl LpConstraint {
    pub fn generalize(&self) -> LpConstraint {
        // TODO: Optimize tailrec
        let &LpConstraint(ref lhs, ref op, ref rhs) = self;
        let ref lhs_expr = simplify(&(lhs - rhs));
        let (constant, lhs_expr) = split_constant_and_expr(lhs_expr);
        LpConstraint(lhs_expr, op.clone(), LitVal(-constant))
    }
}

pub fn split_constant_and_expr(expr: &LpExpression) -> (f32, LpExpression) {
    match expr {
        AddExpr(e1, e2) => {
            if let LitVal(c) = **e2 {
                (c, *e1.clone())
            } else {
                (0.0, expr.clone())
            }
        }
        SubExpr(e1, e2) => {
            if let LitVal(c) = **e2 {
                (-c, *e1.clone())
            } else {
                (0.0, expr.clone())
            }
        }
        _ => (0.0, expr.clone()),
    }
}

pub fn simplify(expr: &LpExpression) -> LpExpression {
    fn simpl(expr: &LpExpression) -> LpExpression {
        match expr {
            MulExpr(ref left_expr, ref right_expr) => {
                let ref left_expr = **left_expr;
                let ref right_expr = **right_expr;

                match (left_expr, right_expr) {
                    // DISTRIBUTIVITY
                    // i*(a+b) = i*a+i*b
                    (i, &AddExpr(ref a, ref b)) => simplify(&AddExpr(
                        Box::new(MulExpr(Box::new(i.clone()), a.clone())),
                        Box::new(MulExpr(Box::new(i.clone()), b.clone())),
                    )),
                    // (a+b)*i = i*a+i*b
                    (&AddExpr(ref a, ref b), i) => simplify(&AddExpr(
                        Box::new(MulExpr(Box::new(i.clone()), a.clone())),
                        Box::new(MulExpr(Box::new(i.clone()), b.clone())),
                    )),
                    // i*(a-b) = i*a-i*b
                    (i, &SubExpr(ref a, ref b)) => simplify(&SubExpr(
                        Box::new(MulExpr(Box::new(i.clone()), a.clone())),
                        Box::new(MulExpr(Box::new(i.clone()), b.clone())),
                    )),

                    // COMMUTATIVITY WITH CONSTANTS
                    // c1*(c2*expr) = (c1*c2)*expr)
                    (&LitVal(c1), &MulExpr(ref ref_c2, ref expr)) => {
                        let ref cc2 = **ref_c2;
                        if let &LitVal(c2) = cc2 {
                            return simplify(&MulExpr(Box::new(LitVal(c1 * c2)), expr.clone()));
                        } else {
                            //MulExpr(Box::new(LitVal(c1)), Box::new(simplify(ref_right_expr)))
                            MulExpr(
                                Box::new(MulExpr(Box::new(LitVal(c1)), Box::new(cc2.clone()))),
                                expr.clone(),
                            )
                        }
                    }
                    //TODO: (Pointless?)
                    // expr1*(c*expr) = c*(expr1*expr2)

                    // COMMUTATIVITY
                    // a*(b*c) = (a*b)*c
                    (expr, &MulExpr(ref ref_left_mul, ref ref_right_mul)) => simplify(&MulExpr(
                        Box::new(MulExpr(Box::new(expr.clone()), ref_left_mul.clone())),
                        ref_right_mul.clone(),
                    )),

                    // Trivial rule: 0 * x = 0
                    (_, &LitVal(v)) if is_zero(v) => LitVal(0.0),
                    (&LitVal(v), _) if is_zero(v) => LitVal(0.0),

                    // Simplify two literals
                    (&LitVal(c1), &LitVal(c2)) => LitVal(c1 * c2),

                    // Place literal first for *
                    (expr, &LitVal(c)) => {
                        simplify(&MulExpr(Box::new(LitVal(c)), Box::new(expr.clone())))
                    }

                    (_, _) => MulExpr(
                        Box::new(simplify(left_expr)),
                        Box::new(simplify(right_expr)),
                    ),
                }
            }
            &AddExpr(ref ref_left_expr, ref ref_right_expr) => {
                let ref left_expr = **ref_left_expr;
                let ref right_expr = **ref_right_expr;

                match (left_expr, right_expr) {
                    // Trivial rule: 0 + x = x
                    (_, &LitVal(v)) if is_zero(v) => simplify(left_expr),

                    // ASSOCIATIVITY
                    // a + (b+c) = (a+b)+c
                    (a, &AddExpr(ref b, ref c)) => simplify(&AddExpr(
                        Box::new(AddExpr(Box::new(a.clone()), b.clone())),
                        c.clone(),
                    )),

                    // a + (b-c) = (a+b) - c
                    (a, &SubExpr(ref b, ref c)) => simplify(&SubExpr(
                        Box::new(AddExpr(Box::new(a.clone()), b.clone())),
                        c.clone(),
                    )),

                    // Simplify two literals
                    (&LitVal(c1), &LitVal(c2)) => LitVal(c1 + c2),

                    // Place literal at the end
                    (&LitVal(c), expr) => {
                        simplify(&AddExpr(Box::new(simplify(expr)), Box::new(LitVal(c))))
                    }

                    // Accumulate consts +/-
                    // (expr+c1)+c2 = expr+(c1+c2)
                    (&AddExpr(ref expr, ref rc1), &LitVal(c2)) => match **rc1 {
                        LitVal(c1) => simplify(&AddExpr(expr.clone(), Box::new(LitVal(c1 + c2)))),
                        _ => AddExpr(
                            Box::new(simplify(ref_left_expr)),
                            Box::new(simplify(ref_right_expr)),
                        ),
                    },
                    // (expr-c1)+c2 = expr+(c2-c1)
                    // (c1-expr)+c2 = -expr+(c1+c2)
                    (&SubExpr(ref rc1, ref rc2), &LitVal(c2)) => {
                        let ref cc1 = **rc1;
                        let ref cc2 = **rc2;
                        match (cc1, cc2) {
                            (_, &LitVal(c1)) => {
                                simplify(&AddExpr(rc1.clone(), Box::new(LitVal(c1 - c2))))
                            }
                            (&LitVal(c1), _) => simplify(&AddExpr(
                                Box::new(SubExpr(Box::new(LitVal(0.0)), rc2.clone())),
                                Box::new(LitVal(c1 + c2)),
                            )),
                            _ => AddExpr(
                                Box::new(simplify(ref_left_expr)),
                                Box::new(simplify(ref_right_expr)),
                            ),
                        }
                    }

                    // Extract the const
                    // (expr1+c)+expr2 = (expr1+expr2)+c
                    (&AddExpr(ref expr1, ref rc), expr2) => match **rc {
                        LitVal(c1) => simplify(&AddExpr(
                            Box::new(AddExpr(expr1.clone(), Box::new(expr2.clone()))),
                            Box::new(LitVal(c1)),
                        )),
                        _ => AddExpr(
                            Box::new(simplify(ref_left_expr)),
                            Box::new(simplify(ref_right_expr)),
                        ),
                    },
                    // (expr1-c)+expr2 = (expr1+expr2)-c
                    (&SubExpr(ref expr1, ref rc), expr2) => match **rc {
                        LitVal(c) => simplify(&SubExpr(
                            Box::new(AddExpr(expr1.clone(), Box::new(expr2.clone()))),
                            Box::new(LitVal(c)),
                        )),
                        _ => AddExpr(
                            Box::new(simplify(ref_left_expr)),
                            Box::new(simplify(ref_right_expr)),
                        ),
                    },

                    // Accumulate consts +/-
                    // (expr+c1)+c2 = expr+(c1+c2) OK
                    // (expr-c1)+c2 = expr+(c2-c1) OK
                    // (c1-expr)+c2 = -expr+(c1+c2) OK

                    // (expr-c1)-c2 = expr-(c1+c2) OK
                    // (expr+c1)-c2 = expr+(c1-c2) OK
                    // (c1-expr)-c2 = -expr+(c1-c2) OK

                    // Extract the const
                    // (expr1+c)+expr2 = (expr1+expr2)+c OK
                    // (expr1-c)+expr2 = (expr1+expr2)-c OK
                    // (expr1+c)-expr2 = (expr1-expr2)+c OK
                    // (expr1-c)-expr2 = (expr1-expr2)-c OK
                    _ => AddExpr(
                        Box::new(simplify(ref_left_expr)),
                        Box::new(simplify(ref_right_expr)),
                    ),
                }
            }
            &SubExpr(ref ref_left_expr, ref ref_right_expr) => {
                let ref left_expr = **ref_left_expr;
                let ref right_expr = **ref_right_expr;

                match (left_expr, right_expr) {
                    // Trivial rule: 0 + x = x
                    (_, &LitVal(v)) if is_zero(v) => simplify(left_expr),

                    // a - (b + c) = (a-b)-c
                    (a, &AddExpr(ref b, ref c)) => simplify(&SubExpr(
                        Box::new(SubExpr(Box::new(a.clone()), b.clone())),
                        c.clone(),
                    )),

                    // a - (b - c) = (a-b)+c
                    (a, &SubExpr(ref b, ref c)) => simplify(&AddExpr(
                        Box::new(SubExpr(Box::new(a.clone()), b.clone())),
                        c.clone(),
                    )),

                    // Place literal at the end
                    (&LitVal(c), expr) => simplify(&AddExpr(Box::new(-expr), Box::new(LitVal(c)))),

                    // Accumulate consts +/-
                    // (expr-c1)-c2 = expr-(c1+c2)
                    // (c1-expr)-c2 = -expr+(c1-c2)
                    (&SubExpr(ref rc1, ref rc2), &LitVal(c2)) => {
                        let ref cc1 = **rc1;
                        let ref cc2 = **rc2;
                        match (cc1, cc2) {
                            (_, &LitVal(c1)) => {
                                simplify(&SubExpr(rc1.clone(), Box::new(LitVal(c1 + c2))))
                            }
                            (&LitVal(c1), _) => simplify(&AddExpr(
                                Box::new(SubExpr(Box::new(LitVal(0.0)), rc2.clone())),
                                Box::new(LitVal(c1 - c2)),
                            )),
                            _ => SubExpr(
                                Box::new(simplify(ref_left_expr)),
                                Box::new(simplify(ref_right_expr)),
                            ),
                        }
                    }

                    // (expr+c1)-c2 = expr+(c1-c2)
                    (&AddExpr(ref expr, ref rc1), &LitVal(c2)) => match **rc1 {
                        LitVal(c1) => simplify(&AddExpr(expr.clone(), Box::new(LitVal(c1 - c2)))),
                        _ => SubExpr(
                            Box::new(simplify(ref_left_expr)),
                            Box::new(simplify(ref_right_expr)),
                        ),
                    },

                    // Extract the const:
                    // (expr1+c)-expr2 = (expr1-expr2)+c
                    (&AddExpr(ref expr1, ref rc), expr2) => match **rc {
                        LitVal(c) => simplify(&AddExpr(
                            Box::new(SubExpr(expr1.clone(), Box::new(expr2.clone()))),
                            Box::new(LitVal(c)),
                        )),
                        _ => SubExpr(
                            Box::new(simplify(ref_left_expr)),
                            Box::new(simplify(ref_right_expr)),
                        ),
                    },
                    // (expr1-c)-expr2 = (expr1-expr2)-c
                    (&SubExpr(ref expr1, ref rc), expr2) => match **rc {
                        LitVal(c) => simplify(&SubExpr(
                            Box::new(SubExpr(expr1.clone(), Box::new(expr2.clone()))),
                            Box::new(LitVal(c)),
                        )),
                        _ => SubExpr(
                            Box::new(simplify(ref_left_expr)),
                            Box::new(simplify(ref_right_expr)),
                        ),
                    },

                    _ => SubExpr(
                        Box::new(simplify(ref_left_expr)),
                        Box::new(simplify(ref_right_expr)),
                    ),
                }
            }
            &ConsBin(LpBinary { .. }) => expr.clone(),
            &ConsInt(LpInteger { .. }) => expr.clone(),
            &ConsCont(LpContinuous { .. }) => expr.clone(),
            &LitVal(_) => expr.clone(),
            _ => expr.clone(),
        }
    }

    let n = simpl(expr);
    // Use parenthesis system because on expression with different syntax tree is not equals
    //if show(self, true) != show(&n, true) {
    if expr.clone() != n.clone() {
        simpl(&n)
    } else {
        n
    }
}
impl LpFileFormat for LpConstraint {
    fn to_lp_file_format(&self) -> String {
        let mut res = String::new();
        res.push_str(&self.0.to_lp_file_format());
        match self.1 {
            GreaterOrEqual => res.push_str(" >= "),
            LessOrEqual => res.push_str(" <= "),
            Equal => res.push_str(" = "),
        }
        res.push_str(&self.2.to_lp_file_format());
        res
    }
}

/// make a complete expression or a constraint with a vector of expressions
///
/// # Examples
///
/// ```
/// use lp_modeler::problem::{LpObjective, LpProblem};
/// use lp_modeler::operations::LpOperations;
/// use lp_modeler::variables::{LpBinary, lp_sum};
///
/// let mut problem = LpProblem::new("My Problem", LpObjective::Maximize);
/// let ref a = LpBinary::new("a");
/// let ref b = LpBinary::new("b");
/// let ref c = LpBinary::new("c");
///
/// let ref v = vec!(a, b, c);
/// problem += lp_sum(v).equal(10.0);
/// ```
///
pub fn lp_sum<T>(expr: &Vec<T>) -> LpExpression
where
    T: Into<LpExpression> + Clone,
{
    let mut expr = expr.clone();
    if let Some(e1) = expr.pop() {
        if let Some(e2) = expr.pop() {
            expr.push(e2);
            AddExpr(Box::new(e1.into()), Box::new(lp_sum(&expr)))
        } else {
            e1.into()
        }
    } else {
        EmptyExpr
    }
}

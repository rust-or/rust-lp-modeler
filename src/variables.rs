/// # Module variables

use self::LpExpression::*;
use std::convert::Into;
use std::rc::Rc;
use variables::Constraint::*;
use problem::LpFileFormat;



pub trait BoundableLp : PartialEq + Clone {
    fn lower_bound(&self, lw: f32) -> Self;
    fn upper_bound(&self, up: f32) -> Self;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpBinary {
    pub name: String
}
impl LpBinary {
    pub fn new(name: &str) -> LpBinary {
        LpBinary { name: name.to_string() }
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
        LpInteger { name: name.to_string(), lower_bound: None, upper_bound: None }
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
        LpContinuous { name: name.to_string(), lower_bound: None, upper_bound: None }
    }
}

macro_rules! implement_boundable {
    ($lp_type: ident) => {
        impl BoundableLp for $lp_type {
            fn lower_bound(&self, lw: f32) -> $lp_type {
                $lp_type {
                    name: self.name.clone(),
                    lower_bound: Some(lw),
                    upper_bound: self.upper_bound
                }
            }
            fn upper_bound(&self, up: f32) -> $lp_type {
                $lp_type {
                    name: self.name.clone(),
                    lower_bound: self.lower_bound,
                    upper_bound: Some(up)
                }
            }
        }
    }
}
implement_boundable!(LpInteger);
implement_boundable!(LpContinuous);

/// ADT for Linear Programming Expression
#[derive(Debug, Clone, PartialEq)]
pub enum LpExpression {
    ConsInt(LpInteger),
    ConsBin(LpBinary),
    ConsCont(LpContinuous),
    MulExpr(Rc<LpExpression>, Rc<LpExpression>),
    AddExpr(Rc<LpExpression>, Rc<LpExpression>),
    SubExpr(Rc<LpExpression>, Rc<LpExpression>),
    LitVal(f32),
    EmptyExpr
}
/*
impl<'a> PartialEq for &'a LpExpression {
    fn eq(&self, other: &String) -> bool { *self == *other }
}
*/
impl LpExpression {
    pub fn dfs_remove_constant(&self) -> LpExpression {
        match self {
            &MulExpr(ref rc_e1, ref rc_e2) => {
                let ref e1 = **rc_e1;
                let ref e2 = **rc_e2;
                if let &LitVal(..) = e1 {
                    if let &LitVal(..) = e2 {
                        EmptyExpr
                    }else{
                        MulExpr(rc_e1.clone(), Rc::new(e2.dfs_remove_constant()))
                    }
                }else{
                    MulExpr(Rc::new(e1.dfs_remove_constant()), Rc::new(e2.dfs_remove_constant()))
                }
            },
            &AddExpr(ref rc_e1, ref rc_e2) => {
                let ref e1 = **rc_e1;
                let ref e2 = **rc_e2;
                if let &LitVal(..) = e1 {
                    if let &LitVal(..) = e2 {
                        EmptyExpr
                    }else {
                        e2.dfs_remove_constant()
                    }
                }else{
                    if let &LitVal(..) = e2 {
                        e1.dfs_remove_constant()
                    }else {
                        AddExpr(Rc::new(e1.dfs_remove_constant()), Rc::new(e2.dfs_remove_constant()))
                    }
                }
            },
            &SubExpr(ref rc_e1, ref rc_e2) => {
                let ref e1 = **rc_e1;
                let ref e2 = **rc_e2;
                if let &LitVal(..) = e1 {
                    if let &LitVal(..) = e2 {
                        EmptyExpr
                    }else {
                        e2.dfs_remove_constant()
                    }
                }else{
                    if let &LitVal(..) = e2 {
                        e1.dfs_remove_constant()
                    }else {
                        SubExpr(Rc::new(e1.dfs_remove_constant()), Rc::new(e2.dfs_remove_constant()))
                    }
                }
            },
            _ => self.clone()
        }
    }
    /// Fix the numeric operand in a multiplication in an expression
    /// c * 4 must be considered as 4 c in a linear formulation lp file
    pub fn normalize(&self) -> LpExpression {
        if let &MulExpr(ref rc_e1, ref rc_e2) = self {
            let ref e1 = **rc_e1;
            let ref e2 = **rc_e2;
            if let &LitVal(..) = e1 {
                return self.clone();
            }else{
                if let &LitVal(..) = e2 {
                    return MulExpr(rc_e2.clone(), rc_e1.clone());
                }else {
                    return MulExpr(rc_e1.clone(), rc_e2.clone());
                }
            }
        }
        self.clone()
    }
}

impl LpFileFormat for LpExpression {
    fn to_lp_file_format(&self) -> String {

        fn simplify(expr: &LpExpression) -> LpExpression {
            match expr {
                &MulExpr(ref ref_left_expr, ref ref_right_expr) => {
                    let ref left_expr = **ref_left_expr;
                    let ref right_expr = **ref_right_expr;

                    match (left_expr, right_expr) {
                        // DISTRIBUTIVITY
                        // i*(a+b) = i*a+i*b
                        (i, &AddExpr(ref a, ref b)) => {
                            simplify(&AddExpr(Rc::new(MulExpr(Rc::new(i.clone()), a.clone())), Rc::new(MulExpr(Rc::new(i.clone()), b.clone()))))
                        }
                        // (a+b)*i = i*a+i*b
                        (&AddExpr(ref a, ref b), i) => {
                            simplify(&AddExpr(Rc::new(MulExpr(Rc::new(i.clone()), a.clone())), Rc::new(MulExpr(Rc::new(i.clone()), b.clone()))))
                        }
                        // i*(a-b) = i*a-i*b
                        (i, &SubExpr(ref a, ref b)) => {
                            simplify(&SubExpr(Rc::new(MulExpr(Rc::new(i.clone()), a.clone())), Rc::new(MulExpr(Rc::new(i.clone()), b.clone()))))
                        }

                        // COMMUTATIVITY WITH CONSTANTS
                        // c1*(c2*expr) = (c1*c2)*expr)
                        (&LitVal(c1), &MulExpr(ref ref_c2, ref expr)) => {
                            let ref cc2 = **ref_c2;
                            if let &LitVal(c2) = cc2 {
                                return simplify(&MulExpr(Rc::new(LitVal(c1 * c2)), expr.clone()))
                            } else {
                                //MulExpr(Rc::new(LitVal(c1)), Rc::new(simplify(ref_right_expr)))
                                MulExpr(Rc::new(MulExpr(Rc::new(LitVal(c1)), Rc::new(cc2.clone()))), expr.clone())
                            }
                        }
                        //TODO: (Pointless?)
                        // expr1*(c*expr) = c*(expr1*expr2)

                        // COMMUTATIVITY
                        // a*(b*c) = (a*b)*c
                        (expr, &MulExpr(ref ref_left_mul, ref ref_right_mul)) => {
                            simplify(&MulExpr(Rc::new(MulExpr(Rc::new(expr.clone()), ref_left_mul.clone())), ref_right_mul.clone()))
                        }

                        // Simplify two literals
                        (&LitVal(c1), &LitVal(c2)) => {
                            LitVal(c1 * c2)
                        }

                        // Place literal first
                        (expr, &LitVal(c)) => {
                            simplify(&MulExpr(Rc::new(LitVal(c)), Rc::new(expr.clone())))
                        },

                        // Trivial rule: 0 * x = 0
                        (&LitVal(c), _) if c == 0.0 => LitVal(0.0),

                        (_, _) => {
                            MulExpr(Rc::new(simplify(left_expr)), Rc::new(simplify(right_expr)))
                        }
                    }
                }
                &AddExpr(ref ref_left_expr, ref ref_right_expr) => {
                    let ref left_expr = **ref_left_expr;
                    let ref right_expr = **ref_right_expr;

                    match (left_expr, right_expr) {
                        // Trivial rule: 0 + x = x
                        (&LitVal(c), _) if c == 0.0 => simplify(right_expr),

                        // ASSOCIATIVITY
                        // a + (b+c) = (a+b)+c
                        (a, &AddExpr(ref b, ref c)) => simplify(&AddExpr(Rc::new(AddExpr(Rc::new(a.clone()),b.clone())), c.clone())),

                        // a + (b-c) = (a+b) - c
                        (a, &SubExpr(ref b, ref c)) => simplify(&SubExpr(Rc::new(AddExpr(Rc::new(a.clone()),b.clone())), c.clone())),

                        // Simplify two literals
                        (&LitVal(c1), &LitVal(c2)) => {
                            LitVal(c1 + c2)
                        },

                        // Place literal first
                        (expr, &LitVal(c)) => {
                            simplify(&AddExpr(Rc::new(LitVal(c)), Rc::new(simplify(expr))))
                        },

                        _ => AddExpr(Rc::new(simplify(ref_left_expr)), Rc::new(simplify(ref_right_expr)))
                    }

                },
                &SubExpr(ref ref_left_expr, ref ref_right_expr) => {
                    let ref left_expr = **ref_left_expr;
                    let ref right_expr = **ref_right_expr;

                    match (left_expr, right_expr) {
                        // a - (b + c) = (a-b)-c
                        (a, &AddExpr(ref b, ref c)) => simplify(&SubExpr(Rc::new(SubExpr(Rc::new(a.clone()),b.clone())),c.clone())),

                        // a - (b - c) = (a-b)+c
                        (a, &SubExpr(ref b, ref c)) => simplify(&AddExpr(Rc::new(SubExpr(Rc::new(a.clone()),b.clone())),c.clone())),

                        // Place literal first
                        (expr, &LitVal(c)) => {
                            simplify(&AddExpr(Rc::new(LitVal(-c)), Rc::new(expr.clone())))
                        },

                        _ => SubExpr(Rc::new(simplify(ref_left_expr)), Rc::new(simplify(ref_right_expr)))
                    }
                },
                &ConsBin(LpBinary{..}) => {
                    expr.clone()
                },
                &ConsInt(LpInteger{..}) => {
                    expr.clone()
                },
                &ConsCont(LpContinuous{..}) => {
                    expr.clone()
                },
                &LitVal(_) => {
                    expr.clone()
                },
                _ => expr.clone()
            }
        }

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

        fn show(e: &LpExpression, with_parenthesis: bool) -> String {
            let str_left_mult = if with_parenthesis {"("} else {""};
            let str_right_mult = if with_parenthesis {")"} else {""};
            let str_op_mult = if with_parenthesis {" * "} else {" "};
            match e {
                &LitVal(n) => n.to_string(),
                &AddExpr(ref e1, ref e2) => str_left_mult.to_string() + &show(e1, with_parenthesis) + " + " + &show(e2, with_parenthesis) + str_right_mult,
                &SubExpr(ref e1, ref e2) => str_left_mult.to_string() + &show(e1, with_parenthesis) + " - " + &show(e2, with_parenthesis) + str_right_mult,
                &MulExpr(ref e1, ref e2) => {
                    let ref deref_e1 = **e1;

                    match deref_e1 {
                        &LitVal(v) if v == 1.0 => {
                            //e2.to_lp_file_format()
                            str_left_mult.to_string() + &" ".to_string() + &show(e2, with_parenthesis) + str_right_mult
                        },
                        &LitVal(v) if v == -1.0 => {
                            //"-".to_string() + &e2.to_lp_file_format()
                            str_left_mult.to_string() + &"-".to_string() + &show(e2, with_parenthesis) + str_right_mult
                        },
                        _ => str_left_mult.to_string() +  &show(e1, with_parenthesis) + str_op_mult + &show(e2, with_parenthesis) + str_right_mult
                    }

                },
                &ConsBin(LpBinary {name: ref n, .. }) => {
                    n.to_string()
                },
                &ConsInt(LpInteger {name: ref n, .. }) => {
                    n.to_string()
                },
                &ConsCont(LpContinuous {name: ref n, .. }) => {
                    n.to_string()
                }
                _ => "EmptyExpr!!".to_string()
            }
        }

        let n = simplify(self);
        // Use parenthesis system because on expression with different syntax tree is not equals
        if show(self, true) != show(&n, true) {
            n.to_lp_file_format()
        } else {
            formalize_signs(show(self, false))
        }
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
    Equal
}

#[derive(Debug, Clone)]
pub struct LpConstraint(pub LpExpression, pub Constraint, pub LpExpression);

impl LpConstraint {
    pub fn generalize(&self) -> LpConstraint {
        // TODO: Optimize tailrec
        fn dfs_constant(expr: &LpExpression, acc: f32) -> f32 {
            match expr {
                &MulExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e2;
                    if let &LitVal(ref x) = e1 {
                        if let &LitVal(ref y) = e2 {
                            acc+x*y
                        }else{
                            dfs_constant(e2, acc)
                        }
                    }else{
                        if let &LitVal(ref y) = e2 {
                            dfs_constant(e1, acc+y)
                        }else {
                            dfs_constant(e2, acc) + dfs_constant(e1, 0.0)
                        }
                    }
                },
                &AddExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e2;
                    if let &LitVal(ref x) = e1 {
                        if let &LitVal(ref y) = e2 {
                            acc+x+y
                        }else {
                            dfs_constant(e2, acc+x)
                        }
                    }else{
                        if let &LitVal(ref y) = e2 {
                            dfs_constant(e1, acc+y)
                        }else {
                            dfs_constant(e2, acc) + dfs_constant(e1, 0.0)
                        }
                    }
                },
                &SubExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e2;
                    if let &LitVal(ref x) = e1 {
                        if let &LitVal(ref y) = e2 {
                            acc+x-y
                        }else {
                            dfs_constant(e2, acc+x)
                        }
                    }else{
                        if let &LitVal(ref y) = e2 {
                            dfs_constant(e1, acc-y)
                        }else {
                            dfs_constant(e1, acc) - dfs_constant(e2, 0.0)
                        }
                    }
                },
                _ => acc
            }
        }

        fn split_constant_and_expr(expr: &LpExpression) -> (f32, LpExpression) {
            match expr {
                &AddExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e1;
                    if let &LitVal(c) = e1 {
                        (c, e2.clone())
                    } else {
                        (0.0,expr.clone())
                    }
                }
                &SubExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e1;
                    if let &LitVal(c) = e1 {
                        (c, e2.clone())
                    } else {
                       (0.0,expr.clone())
                    }
                }
                &MulExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e1;
                    if let &LitVal(c) = e1 {
                        (c, e2.clone())
                    } else {
                        (0.0,expr.clone())
                    }
                }
                _ => (0.0,expr.clone())
            }
        }

        let &LpConstraint(ref lhs, ref op, ref rhs) = self;
        if let &LitVal(0.0) = rhs {
            self.clone()
        }else{
            let ref lhs_expr = lhs - rhs;
            let constant = dfs_constant(lhs_expr, 0.0);
            let lhs_expr = lhs_expr.dfs_remove_constant();
//            let (constant, lhs_expr) = split_constant_and_expr(lhs_expr);
            LpConstraint(lhs_expr, op.clone(), LitVal(-constant))
        }
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
pub fn lp_sum<T>(expr: &Vec<T>) -> LpExpression where T : Into<LpExpression> + Clone {

    let mut expr = expr.clone();
    if let Some(e1) = expr.pop() {
        if let Some(e2) = expr.pop() {
            expr.push(e2);
            AddExpr(Rc::new(e1.into()), Rc::new(lp_sum(&expr)))
        } else {
            e1.into()
        }
    }else{
        EmptyExpr
    }
}










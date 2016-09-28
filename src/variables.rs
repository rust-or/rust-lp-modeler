/// # Module variables

use self::LpExpression::*;
use std::convert::Into;
use std::rc::Rc;
use variables::Constraint::*;



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
                println!("{:?}", e1);
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

impl ToString for LpExpression {
    fn to_string(&self) -> String {

        fn dfs(expr: &LpExpression, acc: &String) -> String {
            match expr {
                &MulExpr(ref e1, ref e2) => {
                    match **e1 {
                        LitVal(v) if v == 1.0 => e2.to_string(),
                        LitVal(v) if v == -1.0 => "-".to_string() + &e2.to_string(),
                        _ => e1.to_string() + " " + &e2.to_string()
                    }
                },
                &AddExpr(ref e1, ref e2) => {
                    e1.to_string() + " + " + &e2.to_string()
                },
                &SubExpr(ref e1, ref e2) => {
                    e1.to_string() + " - " + &e2.to_string()
                },
                &ConsBin(LpBinary {name: ref n, .. }) => {
                    acc.clone() + n
                },
                &ConsInt(LpInteger {name: ref n, .. }) => {
                    acc.clone() + n
                },
                &ConsCont(LpContinuous {name: ref n, .. }) => {
                    acc.clone() + n
                },
                &LitVal(n) => {
                    acc.clone() + &n.to_string()
                },
                _ => acc.clone()
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
        formalize_signs(dfs(self, &String::new()))
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


        let &LpConstraint(ref lhs, ref op, ref rhs) = self;
        if let &LitVal(0.0) = rhs {
            self.clone()
        }else{
            let ref lhs_expr = lhs - rhs;
            let constant = dfs_constant(lhs_expr, 0.0);
            let lhs_expr = lhs_expr.dfs_remove_constant();
            LpConstraint(lhs_expr, op.clone(), LitVal(-constant))
        }
    }
}

impl ToString for LpConstraint {
    fn to_string(&self) -> String {
        let mut res = String::new();
        res.push_str(&self.0.to_string());
        match self.1 {
            GreaterOrEqual => res.push_str(" >= "),
            LessOrEqual => res.push_str(" <= "),
            Equal => res.push_str(" = "),
        }
        res.push_str(&self.2.to_string());
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










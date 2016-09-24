/// # Module variables

use self::LpExpression::*;
use std::convert::Into;
use std::rc::Rc;


trait BoundableLp : PartialEq + Clone {
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

impl BoundableLp for LpInteger {
    fn lower_bound(&self, lw: f32) -> LpInteger {
        LpInteger {
            name: self.name.clone(),
            lower_bound: Some(lw),
            upper_bound: self.upper_bound
        }
    }
    fn upper_bound(&self, up: f32) -> LpInteger {
        LpInteger {
            name: self.name.clone(),
            lower_bound: self.lower_bound,
            upper_bound: Some(up)
        }
    }
}

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

        fn dfs_remove_constant(expr: &LpExpression) -> LpExpression {
            match expr {
                &MulExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e2;
                    if let &LitVal(..) = e1 {
                        if let &LitVal(..) = e2 {
                            EmptyExpr
                        }else{
                            MulExpr(rc_e1.clone(), Rc::new(dfs_remove_constant(e2)))
                        }
                    }else{
                        if let &LitVal(..) = e2 {
                            // Fix: Literal must be on the left side for multiplication
                            //MulExpr(Rc::new(dfs_remove_constant(e1)), rc_e2.clone())
                            MulExpr(rc_e2.clone(), Rc::new(dfs_remove_constant(e1)))
                        }else {
                            MulExpr(Rc::new(dfs_remove_constant(e1)), Rc::new(dfs_remove_constant(e2)))
                        }
                    }
                },
                &AddExpr(ref rc_e1, ref rc_e2) => {
                    let ref e1 = **rc_e1;
                    let ref e2 = **rc_e2;
                    if let &LitVal(..) = e1 {
                        if let &LitVal(..) = e2 {
                            EmptyExpr
                        }else {
                            dfs_remove_constant(e2)
                        }
                    }else{
                        if let &LitVal(..) = e2 {
                            dfs_remove_constant(e1)
                        }else {
                            AddExpr(Rc::new(dfs_remove_constant(e1)), Rc::new(dfs_remove_constant(e2)))
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
                            dfs_remove_constant(e2)
                        }
                    }else{
                        if let &LitVal(..) = e2 {
                            dfs_remove_constant(e1)
                        }else {
                            SubExpr(Rc::new(dfs_remove_constant(e1)), Rc::new(dfs_remove_constant(e2)))
                        }
                    }
                },
                _ => expr.clone()
            }
        }

        let &LpConstraint(ref lhs, ref op, ref rhs) = self;
        if let &LitVal(0.0) = rhs {
            self.clone()
        }else{
            let ref lhs_constraint = lhs - rhs;
            let constant = dfs_constant(lhs_constraint, 0.0);
            let lhs_constraint = dfs_remove_constant(lhs_constraint);
            LpConstraint(lhs_constraint, op.clone(), LitVal(-constant))
        }
    }
}


/// make a complete expression or a constraint with a vector of expressions
///
/// # Exemples
///
/// ```
/// use lp_modeler::problem::{LpObjective, LpProblem};
/// use lp_modeler::operations::LpOperations;
/// use lp_modeler::variables::{LpVariable, LpType, lp_sum};
///
/// let mut problem = LpProblem::new("My Problem", LpObjective::Maximize);
/// let ref a = LpVariable::new("a", LpType::Binary);
/// let ref b = LpVariable::new("b", LpType::Binary);
/// let ref c = LpVariable::new("c", LpType::Binary);
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










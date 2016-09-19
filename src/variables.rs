/// # Module variables

use self::LpExpression::*;
use std::convert::Into;
use std::rc::Rc;

/// Type of variables. Using to initialize a linear programming variable
///
/// # Exemples
///
/// ```
/// use lp_modeler::variables::{LpVariable, LpType};
///
/// let ref a1 = LpVariable::new("a1", LpType::Integer)
///     .lower_bound(10)
///     .upper_bound(20);
///
/// ```
pub enum LpType {
    /// Binary variable
    Binary,
    /// Integer variable
    Integer,
    /// Reel variable
    Continuous
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum LpExpression {
    BinaryVariable {
        name: String,
    },
    IntegerVariable {
        name: String,
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    },
    ContinuousVariable {
        name: String,
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    },
    MulExpr(Rc<LpExpression>, Rc<LpExpression>),
    AddExpr(Rc<LpExpression>, Rc<LpExpression>),
    SubExpr(Rc<LpExpression>, Rc<LpExpression>),
    LitVal(i32),
    EmptyExpr
}

pub struct LpVariable;

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

impl LpVariable {
    pub fn new<S: Into<String>>(name: S, var_type: LpType) -> LpExpression {
        match var_type {
            LpType::Binary => BinaryVariable { name: name.into() },
            LpType::Integer => IntegerVariable { name: name.into(), lower_bound: None, upper_bound: None },
            LpType::Continuous => ContinuousVariable { name: name.into(), lower_bound: None, upper_bound: None }
        }
    }
}

impl LpConstraint {
    pub fn generalize(&self) -> LpConstraint {
        // TODO: Optimize tailrec
        fn dfs_constant(expr: &LpExpression, acc: i32) -> i32 {
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
                            dfs_constant(e2, acc) + dfs_constant(e1, 0)
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
                            dfs_constant(e2, acc) + dfs_constant(e1, 0)
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
                            dfs_constant(e1, acc) - dfs_constant(e2, 0)
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
                            MulExpr(Rc::new(dfs_remove_constant(e1)), rc_e2.clone())
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
        if let &LitVal(0) = rhs {
            self.clone()
        }else{
            let ref lhs_constraint = lhs - rhs;
            let constant = dfs_constant(lhs_constraint, 0);
            let lhs_constraint = dfs_remove_constant(lhs_constraint);
            LpConstraint(lhs_constraint, op.clone(), LitVal(-constant))
        }
    }
}

#[allow(dead_code)]
impl LpExpression {
    pub fn lower_bound(&self, lw: i32) -> LpExpression {
        match self {
            &BinaryVariable { name: ref n } =>
                BinaryVariable {
                    name: n.clone()
                },
            &IntegerVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                IntegerVariable {
                    name: n.clone(),
                    lower_bound: Some(lw),
                    upper_bound: u
                },
            &ContinuousVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                ContinuousVariable {
                    name: n.clone(),
                    lower_bound: Some(lw),
                    upper_bound: u
                },
            _ => self.clone()
        }
    }
    pub fn upper_bound(&self, up: i32) -> LpExpression {
        match self {
            &BinaryVariable { name: ref n } =>
                BinaryVariable {
                    name: n.clone()
                },
            &IntegerVariable { name: ref n, lower_bound: l, upper_bound: _ } =>
                IntegerVariable {
                    name: n.clone(),
                    lower_bound: l,
                    upper_bound: Some(up)
                },
            &ContinuousVariable { name: ref n, lower_bound: l, upper_bound: _ } =>
                ContinuousVariable {
                    name: n.clone(),
                    lower_bound: l,
                    upper_bound: Some(up)
                },
            _ => self.clone()
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
/// problem += lp_sum(v).equal(10);
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










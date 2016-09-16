/// # Module variables

use std::ops::{Add, Mul, Sub, Neg};
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

pub trait LpOperations<T> where T: Into<LpExpression> {
    fn le(&self, lhs_expr: T) -> LpConstraint;
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    fn equal(&self, lhs_expr: T) -> LpConstraint;
}

impl Into<LpExpression> for i32 {
    fn into(self) -> LpExpression {
        LitVal(self)
    }
}

impl<'a> Into<LpExpression> for &'a LpExpression {
    fn into(self) -> LpExpression {
        self.clone()
    }
}

fn general_form_constraints(cstr: &LpConstraint) -> LpConstraint {
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

    let &LpConstraint(ref lhs, ref op, ref rhs) = cstr;
    let new_cstr;
    if let &LitVal(0) = rhs {
        new_cstr = cstr.clone();
    }else{
        let ref lhs_constraint = lhs - rhs;
        let constant = dfs_constant(lhs_constraint, 0);
        let lhs_constraint = dfs_remove_constant(lhs_constraint);
        new_cstr = LpConstraint(lhs_constraint, op.clone(), LitVal(-constant));
    }
    new_cstr
}

// <LpExr> op <LpExpr> where LpExpr is implicit
impl<T: Into<LpExpression> + Clone, U> LpOperations<T> for U where U: Into<LpExpression> + Clone {
    fn le(&self, lhs_expr: T) -> LpConstraint {
        let c = LpConstraint(self.clone().into(), Constraint::LessOrEqual, lhs_expr.clone().into());
        let c = general_form_constraints(&c);
        c
    }
    fn ge(&self, lhs_expr: T) -> LpConstraint {
        let c = LpConstraint(self.clone().into(), Constraint::GreaterOrEqual, lhs_expr.clone().into());
        let c = general_form_constraints(&c);
        c
    }
    fn equal( &self, lhs_expr: T) -> LpConstraint {
        let c = LpConstraint(self.clone().into(), Constraint::Equal, lhs_expr.clone().into());
        let c = general_form_constraints(&c);
        c
    }
}


// LpExpr + (LpExpr, &LpExpr, i32)
impl<T> Add<T> for LpExpression where T: Into<LpExpression> + Clone {
    type Output = LpExpression;
    fn add(self, _rhs: T) -> LpExpression {
        AddExpr(Rc::new(self.clone()), Rc::new(_rhs.into()))
    }
}

// &LpExpr + (LpExpr, &LpExpr, i32)
impl<'a, T> Add<T> for &'a LpExpression where T: Into<LpExpression> + Clone {
    type Output = LpExpression;
    fn add(self, _rhs: T) -> LpExpression {
        AddExpr(Rc::new(self.clone()), Rc::new(_rhs.into()))
    }
}

// i32 + &LpExpr
impl<'a> Add<&'a LpExpression> for i32 {
    type Output = LpExpression;
    fn add(self, _rhs: &'a LpExpression) -> LpExpression {
        AddExpr(Rc::new(LitVal(self)), Rc::new(_rhs.clone()))
    }
}

// LpExpr - (LpExpr, &LpExpr, i32)
impl<T> Sub<T> for LpExpression where T: Into<LpExpression> + Clone {
    type Output = LpExpression;
    fn sub(self, _rhs: T) -> LpExpression {
        SubExpr(Rc::new(self.clone()), Rc::new(_rhs.into()))
    }
}

// &LpExpr - (LpExpr, &LpExpr, i32)
impl<'a, T> Sub<T> for &'a LpExpression where T: Into<LpExpression> + Clone {
    type Output = LpExpression;
    fn sub(self, _rhs: T) -> LpExpression {
        SubExpr(Rc::new(self.clone()), Rc::new(_rhs.into()))
    }
}

// i32 - &LpExpr
impl<'a> Sub<&'a LpExpression> for i32 {
    type Output = LpExpression;
    fn sub(self, _rhs: &'a LpExpression) -> LpExpression {
        SubExpr(Rc::new(LitVal(self)), Rc::new(_rhs.clone()))
    }
}

impl<'a> Neg for &'a LpExpression {
    type Output = LpExpression;
    fn neg(self) -> LpExpression {
        MulExpr(Rc::new(LitVal(-1)), Rc::new(self.clone()))
    }
}



// i32 * LpExpr
impl Mul<LpExpression> for i32 {
    type Output = LpExpression;
    fn mul(self, _rhs: LpExpression) -> LpExpression {
        LpExpression::MulExpr(Rc::new(LitVal(self)), Rc::new(_rhs))
    }
}

// i32 * &LpExp
impl<'a> Mul<&'a LpExpression> for i32 {
    type Output = LpExpression;

    fn mul(self, _rhs: &'a LpExpression) -> LpExpression {
        MulExpr(Rc::new(LitVal(self)), Rc::new(_rhs.clone()))
    }
}

/// make a complete expression or a constraint with a vector of expressions
///
/// # Exemples
///
/// ```
/// use lp_modeler::problem::{LpObjective, LpProblem};
/// use lp_modeler::variables::{LpVariable, LpType, lp_sum, LpOperations};
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










use std::ops::{Add, Mul};
use self::LpExpression::*;
use std::convert::Into;

pub enum LpType {
    Binary,
    Integer,
    Continuous
}

#[derive(Debug, Clone, Copy)]
pub enum LpVariable {
    BinaryVariable {
        name: &'static str,
    },
    IntegerVariable {
        name: &'static str,
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    },
    ContinuousVariable {
        name: &'static str,
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    }
}

impl LpVariable {
    pub fn new(name: &'static str, var_type: LpType) -> LpVariable {
        match var_type {
            LpType::Binary => LpVariable::BinaryVariable { name: name },
            LpType::Integer => LpVariable::IntegerVariable { name: name, lower_bound: None, upper_bound: None },
            LpType::Continuous => LpVariable::ContinuousVariable { name: name, lower_bound: None, upper_bound: None }

        }
    }
    fn lower_bound(&self, lw: i32) -> LpVariable {
        match self {
            &LpVariable::BinaryVariable { name: ref n } =>
                LpVariable::BinaryVariable {
                    name: n
                },
            &LpVariable::IntegerVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                LpVariable::IntegerVariable {
                    name: n,
                    lower_bound: Some(lw),
                    upper_bound: u
                },
            &LpVariable::ContinuousVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                LpVariable::ContinuousVariable {
                    name: n,
                    lower_bound: Some(lw),
                    upper_bound: u
                }
        }
    }
    fn upper_bound(&self, up: i32) -> LpVariable {
        match self {
            &LpVariable::BinaryVariable { name: ref n } =>
                LpVariable::BinaryVariable {
                    name: n.clone()
                },
            &LpVariable::IntegerVariable { name: ref n, lower_bound: l, upper_bound: _ } =>
                LpVariable::IntegerVariable {
                    name: n.clone(),
                    lower_bound: l,
                    upper_bound: Some(up)
                },
            &LpVariable::ContinuousVariable { name: ref n, lower_bound: l, upper_bound: _ } =>
                LpVariable::ContinuousVariable {
                    name: n.clone(),
                    lower_bound: l,
                    upper_bound: Some(up)
                }
        }
    }
}





#[derive(Debug, Clone)]
pub enum LpExpression {
    MulExpr(i32, LpVariable),
    AddExpr(Box<LpExpression>, Box<LpExpression>),
    LitVar(LpVariable),
    LitVal(i32),
    EmptyExpr
}

pub trait LpOperations<T> where T: Into<LpExpression> {
    fn lt(&self, lhs_expr: T) -> LpConstraint;
    fn le(&self, lhs_expr: T) -> LpConstraint;
    fn gt(&self, lhs_expr: T) -> LpConstraint;
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    fn eq(&self, lhs_expr: T) -> LpConstraint;
}

impl Into<LpExpression> for i32 {
    fn into(self) -> LpExpression {
        LitVal(self)
    }
}
impl Into<LpExpression> for LpVariable {
    fn into(self) -> LpExpression {
        MulExpr(1, self)
    }
}

// <LpExr> op <LpExpr> where LpExpr is implicit
impl<T: Into<LpExpression>, U> LpOperations<T> for U where U: Into<LpExpression> + Clone {
    fn lt(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Less, lhs_expr.into())
    }
    fn le(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::LessOrEqual, lhs_expr.into())
    }
    fn gt(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Greater, lhs_expr.into())
    }
    fn ge(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::GreaterOrEqual, lhs_expr.into())
    }
    fn eq( &self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Equal, lhs_expr.into())
    }
}


// i32 * LpVar
impl Mul<LpVariable> for i32 {
    type Output = LpExpression;
    fn mul(self, _rhs: LpVariable) -> LpExpression {
        LpExpression::MulExpr(self, _rhs)
    }
}

// LpVar + LpVar
impl Add for LpVariable {
    type Output = LpExpression;
    fn add(self, _rhs: LpVariable) -> LpExpression {
        AddExpr(Box::new(MulExpr(1, self)), Box::new(MulExpr(1, _rhs)))
    }
}

// LpExpr + LpVar
impl Add<LpVariable> for LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: LpVariable) -> LpExpression {
             AddExpr(Box::new(self), Box::new(MulExpr(1, _rhs)))
    }
}

// LpExpr + LpExpr
impl Add for LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: LpExpression) -> LpExpression {
        AddExpr(Box::new(self), Box::new(_rhs))
    }
}
pub fn lpSum(expr: &Vec<LpVariable>) -> LpExpression {

    let mut expr = expr.clone();

    if let Some(e1) = expr.pop() {
        if let Some(e2) = expr.pop() {
            AddExpr(Box::new(AddExpr(Box::new(MulExpr(1, e1)), Box::new(MulExpr(1, e2)))), Box::new(lpSum(&expr)))
        } else {
            MulExpr(1, e1)
        }
    }else {
        EmptyExpr
    }
}



#[derive(Debug)]
pub enum Constraint {
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
    Equal
}

#[derive(Debug)]
pub struct LpConstraint(LpExpression, Constraint, LpExpression);

impl LpConstraint{
}





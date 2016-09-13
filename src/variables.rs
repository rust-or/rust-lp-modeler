use std::ops::{Add, Mul};
use self::LpExpression::*;
use std::convert::Into;

pub enum LpType {
    Binary,
    Integer,
    Continuous
}

#[derive(Debug, Clone)]
pub enum LpExpression {
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
    },
    MulExpr(i32, Box<LpExpression>),
    AddExpr(Box<LpExpression>, Box<LpExpression>),
    LitVal(i32),
    EmptyExpr
}

pub struct LpVariable;

impl LpVariable {
    pub fn new(name: &'static str, var_type: LpType) -> LpExpression {
        match var_type {
            LpType::Binary => BinaryVariable { name: name },
            LpType::Integer => IntegerVariable { name: name, lower_bound: None, upper_bound: None },
            LpType::Continuous => ContinuousVariable { name: name, lower_bound: None, upper_bound: None }
        }
    }
}
impl LpExpression {
    fn lower_bound(&self, lw: i32) -> LpExpression {
        match self {
            &BinaryVariable { name: ref n } =>
                BinaryVariable {
                    name: n
                },
            &IntegerVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                IntegerVariable {
                    name: n,
                    lower_bound: Some(lw),
                    upper_bound: u
                },
            &ContinuousVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                ContinuousVariable {
                    name: n,
                    lower_bound: Some(lw),
                    upper_bound: u
                },
            _ => EmptyExpr
        }
    }
    fn upper_bound(&self, up: i32) -> LpExpression {
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
            _ => EmptyExpr
        }
    }
}





/*
#[derive(Debug, Clone)]
pub enum LpExpression {
    MulExpr(i32, LpVariable),
    AddExpr(Box<LpExpression>, Box<LpExpression>),
    LitVar(LpVariable),
    LitVal(i32),
    EmptyExpr
}
*/

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
/*
impl Into<LpExpression> for LpVariable {
    fn into(self) -> LpExpression {
        MulExpr(1, self)
    }
}
*/

// <LpExr> op <LpExpr> where LpExpr is implicit
impl<T: Into<LpExpression> + Clone, U> LpOperations<T> for U where U: Into<LpExpression> + Clone {
    fn lt(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Less, lhs_expr.clone().into())
    }
    fn le(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::LessOrEqual, lhs_expr.clone().into())
    }
    fn gt(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Greater, lhs_expr.clone().into())
    }
    fn ge(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::GreaterOrEqual, lhs_expr.clone().into())
    }
    fn eq( &self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Equal, lhs_expr.clone().into())
    }
}


// LpExpr + &LpExpr
impl<'a> Add<&'a LpExpression> for LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: &'a LpExpression) -> LpExpression {
        AddExpr(Box::new(self), Box::new(_rhs.clone()))
    }
}

// &LpExpr + &LpExpr
impl<'a, 'b> Add<&'a LpExpression> for &'b LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: &'a LpExpression) -> LpExpression {
        AddExpr(Box::new(self.clone()), Box::new(_rhs.clone()))
    }
}

// LpExpr + LpExpr
impl Add<LpExpression> for LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: LpExpression) -> LpExpression {
        AddExpr(Box::new(self.clone()), Box::new(_rhs))
    }
}

// i32 + LpExpr
impl Add<i32> for LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: i32) -> LpExpression {
        AddExpr(Box::new(self.clone()), Box::new(LitVal(_rhs)))
    }
}

// i32 + &LpExpr
impl<'a> Add<i32> for &'a LpExpression {
    type Output = LpExpression;
    fn add(self, _rhs: i32) -> LpExpression {
        AddExpr(Box::new(self.clone()), Box::new(LitVal(_rhs)))
    }
}

// &LpExpr + i32
impl<'a> Add<&'a LpExpression> for i32 {
    type Output = LpExpression;
    fn add(self, _rhs: &'a LpExpression) -> LpExpression {
        AddExpr(Box::new(LitVal(self)), Box::new(_rhs.clone()))
    }
}

// i32 * LpExp
impl Mul<LpExpression> for i32 {
    type Output = LpExpression;
    fn mul(self, _rhs: LpExpression) -> LpExpression {
        LpExpression::MulExpr(self, Box::new(_rhs))
    }
}

// i32 * &LpExp
impl<'a> Mul<&'a LpExpression> for i32 {
    type Output = LpExpression;

    fn mul(self, _rhs: &'a LpExpression) -> LpExpression {
        AddExpr(Box::new(LitVal(self)), Box::new(_rhs.clone()))
    }
}

pub fn lp_sum(expr: &Vec<LpExpression>) -> LpExpression {

    let mut expr = expr.clone();

    if let Some(e1) = expr.pop() {
        if let Some(e2) = expr.pop() {
            AddExpr(Box::new(AddExpr(Box::new(MulExpr(1, Box::new(e1))), Box::new(MulExpr(1, Box::new(e2))))), Box::new(lp_sum(&expr)))
        } else {
            MulExpr(1, Box::new(e1))
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





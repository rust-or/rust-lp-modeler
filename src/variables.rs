use std::ops::{Add, Mul};
use self::LpExpression::*;

pub enum LpType {
    Binary,
    Integer,
    Continuous
}

/*
 enum Expr {
    LpVar,
    Mul(i32, Box<LpVar>),
    Vec<LpVar>

 }
*/
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
    EmptyExpr
}

pub trait LpOperations<T> {
    fn lt(&self, lhs_expr: T) -> LpConstraint;
    fn le(&self, lhs_expr: T) -> LpConstraint;
    fn gt(&self, lhs_expr: T) -> LpConstraint;
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    fn eq(&self, lhs_expr: T) -> LpConstraint;
}

// <LpExr> op <LpExpr>
impl LpOperations<LpExpression> for LpExpression {
    fn lt(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::Less, lhs_expr)
    }
    fn le(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::LessOrEqual, lhs_expr)
    }
    fn gt(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::Greater, lhs_expr)
    }
    fn ge(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::GreaterOrEqual, lhs_expr)
    }
    fn eq( &self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::Equal, lhs_expr)
    }
}

// <LpExr> op <LpVar>
impl LpOperations<LpVariable> for LpExpression {
    fn lt(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::Less, MulExpr(1, lhs_expr))
    }
    fn le(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::LessOrEqual, MulExpr(1, lhs_expr))
    }
    fn gt(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::Greater, MulExpr(1, lhs_expr))
    }
    fn ge(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::GreaterOrEqual, MulExpr(1, lhs_expr))
    }
    fn eq( &self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(self.clone(), Constraint::Equal, MulExpr(1, lhs_expr))
    }
}

// <LpVar> op <LpVar>
impl LpOperations<LpVariable> for LpVariable {
    fn lt(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::Less, MulExpr(1, lhs_expr))
    }
    fn le(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::LessOrEqual, MulExpr(1, lhs_expr))
    }
    fn gt(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::Greater, MulExpr(1, lhs_expr))
    }
    fn ge(&self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::GreaterOrEqual, MulExpr(1, lhs_expr))
    }
    fn eq( &self, lhs_expr: LpVariable) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::Equal, MulExpr(1, lhs_expr))
    }
}

// <LpVar> op <LpExpr>
impl LpOperations<LpExpression> for LpVariable {
    fn lt(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::Less, lhs_expr)
    }
    fn le(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::LessOrEqual, lhs_expr)
    }
    fn gt(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::Greater, lhs_expr)
    }
    fn ge(&self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::GreaterOrEqual, lhs_expr)
    }
    fn eq( &self, lhs_expr: LpExpression) -> LpConstraint {
        LpConstraint(MulExpr(1, self.clone()), Constraint::Equal, lhs_expr)
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





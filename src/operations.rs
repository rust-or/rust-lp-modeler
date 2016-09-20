use std::ops::{Add, Mul, Sub, Neg};
use variables::{LpExpression, LpConstraint, Constraint};
use variables::LpExpression::*;
use std::rc::Rc;

/// Operations trait for any type implementing Into<LpExpressions> trait
pub trait LpOperations<T> where T: Into<LpExpression> {
    /// Less or equal binary syntax for LpExpression
    fn le(&self, lhs_expr: T) -> LpConstraint;
    /// Greater or equal binary syntax for LpExpression
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    /// Equality binary syntax for LpExpression
    fn equal(&self, lhs_expr: T) -> LpConstraint;
}

/// Macro implementing Into<LpExpression> for any type coercing to f32
macro_rules! num_to_into_expr {
    ($type_to:ty) => {
        impl Into<LpExpression> for $type_to {
            fn into(self) -> LpExpression {
                LitVal(self as f32)
            }
        }
    };
}

/// Macro implementing binary operations Into<LpExpression> and &Into<LpExpression>
macro_rules! expr_ops_expr {
    ($trait_name: ident, $f_name: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for LpExpression where T: Into<LpExpression> + Clone {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type(Rc::new(self.clone()), Rc::new(_rhs.into()))
            }
        }

        impl<'a, T> $trait_name<T> for &'a LpExpression where T: Into<LpExpression> + Clone {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type(Rc::new(self.clone()), Rc::new(_rhs.into()))
            }
        }
    };
}

/// Macro implementing binary operations for a numeric type
macro_rules! num_ops_expr {
    ($num_type: ty, $trait_name: ident, $f_name: ident) => {
        impl $trait_name<LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: LpExpression) -> LpExpression {
                MulExpr(Rc::new(LitVal(self as f32)), Rc::new(_rhs))
            }
        }
        impl<'a> $trait_name<&'a LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: &'a LpExpression) -> LpExpression {
                MulExpr(Rc::new(LitVal(self as f32)), Rc::new(_rhs.clone()))
            }
        }
    };
}

/// &LpExpression to LpExpression
impl<'a> Into<LpExpression> for &'a LpExpression {
    fn into(self) -> LpExpression {
        self.clone()
    }
}

/// Implementing LpOperations trait for any Into<LpExpression>
impl<T: Into<LpExpression> + Clone, U> LpOperations<T> for U where U: Into<LpExpression> + Clone {
    fn le(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::LessOrEqual, lhs_expr.clone().into()).generalize()
    }
    fn ge(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::GreaterOrEqual, lhs_expr.clone().into()).generalize()
    }
    fn equal( &self, lhs_expr: T) -> LpConstraint {
        LpConstraint(self.clone().into(), Constraint::Equal, lhs_expr.clone().into()).generalize()
    }
}

impl<'a> Neg for &'a LpExpression {
    type Output = LpExpression;
    fn neg(self) -> LpExpression {
        MulExpr(Rc::new(LitVal(-1.0)), Rc::new(self.clone()))
    }
}

num_to_into_expr!(f32);
num_to_into_expr!(i32);

expr_ops_expr!(Add, add, AddExpr);
expr_ops_expr!(Sub, sub, SubExpr);

num_ops_expr!(f32, Add, add);
num_ops_expr!(f32, Mul, mul);
num_ops_expr!(f32, Sub, sub);
num_ops_expr!(i32, Add, add);
num_ops_expr!(i32, Mul, mul);
num_ops_expr!(i32, Sub, sub);



use std::ops::{Add, Mul, Neg, Sub};
use dsl::LpExpression::*;
use dsl::{Constraint, LpBinary, LpConstraint, LpContinuous, LpExpression, LpInteger, LpExprOp, LpExprArena};

/// Operations trait for any type implementing Into<LpExpressions> trait
pub trait LpOperations<T> where T: Into<LpExprArena> {
    /// Less or equal binary syntax for LpExpression
    fn le(&self, lhs_expr: T) -> LpConstraint;
    /// Greater or equal binary syntax for LpExpression
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    /// Equality binary syntax for LpExpression
    fn equal(&self, lhs_expr: T) -> LpConstraint;
}

/// Macro implementing Into<LpExpression> for types
macro_rules! to_into_expr {
    ($type_to:ty) => {
        impl Into<LpExpression> for $type_to {
            fn into(self) -> LpExpression {
                LpExpression::LpAtomicExpr::LitVal(self as f32)
            }
        }
    };
    ($type_to:ty, $wrapper: ident) => {
        impl Into<LpExpression> for $type_to {
            fn into(self) -> LpExpression {
                $wrapper(self)
            }
        }
        impl<'a> Into<LpExpression> for &'a $type_to {
            fn into(self) -> LpExpression {
                $wrapper(self.clone())
            }
        }
    };
}
to_into_expr!(f32);
to_into_expr!(i32);
to_into_expr!(LpBinary, LpExpression::LpAtomicExpr::ConsBin);
to_into_expr!(LpInteger, LpExpression::LpAtomicExpr::ConsInt);
to_into_expr!(LpContinuous, LpExpression::LpAtomicExpr::ConsCont);

/// Macro implementing binary operations for Into<LpExpression> or &Into<LpExpression>
macro_rules! operations_for_expr {
    ($trait_name: ident, $f_name: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                LpExpression::LpCompExpr($expr_type, self.clone(), _rhs.into())
            }
        }
        impl<'a, T> $trait_name<T> for &'a LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                LpExpression::LpCompExpr($expr_type, self.clone(), _rhs.into())
            }
        }
    };
}

operations_for_expr!(Add, add, LpExprOp::Add);
operations_for_expr!(Sub, sub, LpExprOp::Subtract);
operations_for_expr!(Mul, mul, LpExprOp::Multiply);

/// Macro implementing a binary operation with a LpVars and a Into<Expression>
macro_rules! lpvars_operation_for_intoexpr {
    ($trait_name: ident, $f_name: ident, $lp_type: ident, $expr_type: ident, $cons_type: ident) => {
        impl<T> $trait_name<T> for $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type($cons_type(self.clone()), _rhs.into()).normalize()
            }
        }
        impl<'a, T> $trait_name<T> for &'a $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type($cons_type(self.clone()), _rhs.into()).normalize()
            }
        }
    };
}

lpvars_operation_for_intoexpr!(Mul, mul, LpBinary, LpExprOp::Multiply, LpExpression::LpAtomicExpr::ConsBin);
lpvars_operation_for_intoexpr!(Add, add, LpBinary, LpExprOp::Add, LpExpression::LpAtomicExpr::ConsBin);
lpvars_operation_for_intoexpr!(Sub, sub, LpBinary, LpExprOp::Subtract, LpExpression::LpAtomicExpr::ConsBin);
lpvars_operation_for_intoexpr!(Mul, mul, LpInteger, LpExprOp::Multiply, LpExpression::LpAtomicExpr::ConsInt);
lpvars_operation_for_intoexpr!(Add, add, LpInteger, LpExprOp::Add, LpExpression::LpAtomicExpr::ConsInt);
lpvars_operation_for_intoexpr!(Sub, sub, LpInteger, LpExprOp::Subtract, LpExpression::LpAtomicExpr::ConsInt);
lpvars_operation_for_intoexpr!(Mul, mul, LpContinuous, LpExprOp::Multiply, LpExpression::LpAtomicExpr::ConsCont);
lpvars_operation_for_intoexpr!(Add, add, LpContinuous, LpExprOp::Add, LpExpression::LpAtomicExpr::ConsCont);
lpvars_operation_for_intoexpr!(Sub, sub, LpContinuous, LpExprOp::Subtract, LpExpression::LpAtomicExpr::ConsCont);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_expr {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident) => {
        impl $trait_name<LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: LpExpression) -> LpExpression {
                LpExpression::LpCompExpr($type_expr, LpExpression::LpAtomicExpr::LitVal(self as f32), _rhs)
            }
        }
        impl<'a> $trait_name<&'a LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: &'a LpExpression) -> LpExpression {
                LpExpression::LpCompExpr($type_expr, LpExpression::LpAtomicExpr::LitVal(self as f32), _rhs.clone())
            }
        }
    };
}
/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_expr {
    ($num_type: ty) => {
        numeric_operation_for_expr!($num_type, Add, add, LpExprOp::Add);
        numeric_operation_for_expr!($num_type, Mul, mul, LpExprOp::Multiply);
        numeric_operation_for_expr!($num_type, Sub, sub, LpExprOp::Subtract);
    };
}
numeric_all_ops_for_expr!(f32);
numeric_all_ops_for_expr!(i32);

/// &LpExpression to LpExpression
impl<'a> Into<LpExpression> for &'a LpExpression {
    fn into(self) -> LpExpression {
        self.clone()
    }
}

/// Implementing LpOperations trait for any Into<LpExprArena>
impl<T: Into<LpExprArena> + Clone, U> LpOperations<T> for U where U: Into<LpExprArena> + Clone {
    fn le(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(
            self.clone().into(),
            Constraint::LessOrEqual,
            lhs_expr.clone().into(),
        )
        .generalize()
    }
    fn ge(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(
            self.clone().into(),
            Constraint::GreaterOrEqual,
            lhs_expr.clone().into(),
        )
        .generalize()
    }
    fn equal(&self, lhs_expr: T) -> LpConstraint {
        LpConstraint(
            self.clone().into(),
            Constraint::Equal,
            lhs_expr.clone().into(),
        )
        .generalize()
    }
}

//impl<'a> Neg for &'a LpExpression {
//    type Output = LpExpression;
//    fn neg(self) -> LpExpression {
//        LpExpression::LpCompExpr(
//            LpExprOp::Multiply,
//            LpExpression::LpAtomicExpr::LitVal(-1.0),
//            self
//        )
//    }
//}
macro_rules! neg_operation_for_lpvars {
    ($lp_var_type: ty, $constr_expr: ident) => {
        impl<'a> Neg for &'a $lp_var_type {
            type Output = LpExpression;
            fn neg(self) -> LpExpression {
                LpExpression::LpCompExpr(
                    LpExprOp::Multiply,
                    LpExpression::LpAtomicExpr::LitVal(-1.0), 
                    $constr_expr(self.clone())
                )
            }
        }
    };
}
neg_operation_for_lpvars!(LpInteger, LpExpression::LpAtomicExpr::ConsInt);
neg_operation_for_lpvars!(LpContinuous, LpExpression::LpAtomicExpr::ConsCont);
neg_operation_for_lpvars!(LpBinary, LpExpression::LpAtomicExpr::ConsBin);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_lpvars {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident, $lp_type: ty, $cons_expr: ident) => {
        impl $trait_name<$lp_type> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: $lp_type) -> LpExpression {
                LpExpression::LpCompExpr(
                    $type_expr,
                    LpExpression::LpAtomicExpr::LitVal(self as f32),
                    $cons_expr(_rhs)
                )
            }
        }
        impl<'a> $trait_name<&'a $lp_type> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: &'a $lp_type) -> LpExpression {
                LpExpression::LpCompExpr(
                    $type_expr,
                    LpExpression::LpAtomicExpr::LitVal(self as f32),
                    $cons_expr(_rhs.clone()),
                )
            }
        }
    };
}

/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_lpvars {
    ($num_type: ty) => {
        numeric_operation_for_lpvars!($num_type, Add, add, LpExprOp::Add,  LpInteger, LpExpression::LpAtomicExpr::ConsInt);
        numeric_operation_for_lpvars!($num_type, Add, add, LpExprOp::Add,  LpBinary, LpExpression::LpAtomicExpr::ConsBin);
        numeric_operation_for_lpvars!($num_type, Add, add, LpExprOp::Add,  LpContinuous, LpExpression::LpAtomicExpr::ConsCont);
        numeric_operation_for_lpvars!($num_type, Mul, mul, LpExprOp::Multiply,  LpInteger, LpExpression::LpAtomicExpr::ConsInt);
        numeric_operation_for_lpvars!($num_type, Mul, mul, LpExprOp::Multiply,  LpBinary, LpExpression::LpAtomicExpr::ConsBin);
        numeric_operation_for_lpvars!($num_type, Mul, mul, LpExprOp::Multiply,  LpContinuous, LpExpression::LpAtomicExpr::ConsCont);
        numeric_operation_for_lpvars!($num_type, Sub, sub, LpExprOp::Subtract,  LpInteger, LpExpression::LpAtomicExpr::ConsInt);
        numeric_operation_for_lpvars!($num_type, Sub, sub, LpExprOp::Subtract,  LpBinary, LpExpression::LpAtomicExpr::ConsBin);
        numeric_operation_for_lpvars!($num_type, Sub, sub, LpExprOp::Subtract,  LpContinuous, LpExpression::LpAtomicExpr::ConsCont);
    };
}
numeric_all_ops_for_lpvars!(i32);
numeric_all_ops_for_lpvars!(f32);

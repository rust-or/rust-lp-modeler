use std::ops::{Add, Mul, Neg, Sub};
use dsl::LpExprNode::*;
use dsl::{Constraint, LpBinary, LpConstraint, LpContinuous, LpExprNode, LpInteger, LpExpression};
use dsl::LpExprOp::{Addition, Subtraction, Multiplication};

/// Operations trait for any type implementing Into<LpExpression> trait
pub trait LpOperations<T> where T: Into<LpExpression> {
    /// Less or equal binary syntax for LpExpression
    fn le(&self, lhs_expr: T) -> LpConstraint;
    /// Greater or equal binary syntax for LpExpression
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    /// Equality binary syntax for LpExpression
    fn equal(&self, lhs_expr: T) -> LpConstraint;
}

/// Macro implementing binary operations for Into<LpExpression> or &Into<LpExpression>
macro_rules! operations_for_expr {
    ($trait_name: ident, $f_name: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExpression {
                let new_lp_expr_arena = self.clone();
                new_lp_expr_arena.merge_cloned_arenas(&not_yet_lp_expr_arena.into(), $expr_type)
            }
        }
        impl<'a, T> $trait_name<T> for &'a LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExpression {
                let new_lp_expr_arena = (*self).clone();
                new_lp_expr_arena.merge_cloned_arenas(&not_yet_lp_expr_arena.into(), $expr_type)
            }
        }
    };
}

operations_for_expr!(Add, add, Addition);
operations_for_expr!(Sub, sub, Subtraction);
operations_for_expr!(Mul, mul, Multiplication);

/// Macro implementing a binary operation with a LpVars and a Into<Expression>
macro_rules! lpvars_operation_for_intoexpr {
    ($trait_name: ident, $f_name: ident, $lp_type: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExpression {
                let new_lp_expr_arena: LpExpression = self.clone().into();
                new_lp_expr_arena.merge_cloned_arenas(&not_yet_lp_expr_arena.into(), $expr_type)
            }
        }
        impl<'a, T> $trait_name<T> for &'a $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExpression {
                let new_lp_expr_arena: LpExpression = (*self).clone().into();
                new_lp_expr_arena.merge_cloned_arenas(&not_yet_lp_expr_arena.into(), $expr_type)
            }
        }
    };
}

lpvars_operation_for_intoexpr!(Mul, mul, LpBinary, Multiplication);
lpvars_operation_for_intoexpr!(Add, add, LpBinary, Addition);
lpvars_operation_for_intoexpr!(Sub, sub, LpBinary, Subtraction);
lpvars_operation_for_intoexpr!(Mul, mul, LpInteger, Multiplication);
lpvars_operation_for_intoexpr!(Add, add, LpInteger, Addition);
lpvars_operation_for_intoexpr!(Sub, sub, LpInteger, Subtraction);
lpvars_operation_for_intoexpr!(Mul, mul, LpContinuous, Multiplication);
lpvars_operation_for_intoexpr!(Add, add, LpContinuous, Addition);
lpvars_operation_for_intoexpr!(Sub, sub, LpContinuous, Subtraction);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_expr {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident) => {
        impl $trait_name<LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: LpExpression) -> LpExpression {
                let new_lp_expr_arena: LpExpression = (self as f32).into();
                new_lp_expr_arena.merge_cloned_arenas(&lp_expr_arena.clone(), $type_expr)
            }
        }
        impl<'a> $trait_name<&'a LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: &'a LpExpression) -> LpExpression {
                let new_lp_expr_arena: LpExpression = (self as f32).into();
                new_lp_expr_arena.merge_cloned_arenas(lp_expr_arena, $type_expr)
            }
        }
    };
}
/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_expr {
    ($num_type: ty) => {
        numeric_operation_for_expr!($num_type, Add, add, Addition);
        numeric_operation_for_expr!($num_type, Mul, mul, Multiplication);
        numeric_operation_for_expr!($num_type, Sub, sub, Subtraction);
    };
}
numeric_all_ops_for_expr!(f32);
numeric_all_ops_for_expr!(i32);

/// &LpExpression to LpExpression
impl<'a> Into<LpExpression> for &'a LpExpression {
    fn into(self) -> LpExpression {
        (*self).clone()
    }
}

/// Implementing LpOperations trait for any Into<LpExpression>
impl<T: Into<LpExpression> + Clone, U> LpOperations<T> for U where U: Into<LpExpression> + Clone {
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

impl<'a> Neg for &'a LpExprNode {
    type Output = LpExpression;
    fn neg(self) -> LpExpression {
        let new_lp_expr_arena: LpExpression = LitVal(-1.0).into();
        new_lp_expr_arena.merge_cloned_arenas(&self.clone().into(), Multiplication)
    }
}

macro_rules! neg_operation_for_lpvars {
    ($lp_var_type: ty) => {
        impl<'a> Neg for &'a $lp_var_type {
            type Output = LpExpression;
            fn neg(self) -> LpExpression {
                let new_lp_expr_arena: LpExpression = LitVal(-1.0).into();
                new_lp_expr_arena.merge_cloned_arenas(&self.clone().into(), Multiplication)
            }
        }
    };
}
neg_operation_for_lpvars!(LpInteger);
neg_operation_for_lpvars!(LpContinuous);
neg_operation_for_lpvars!(LpBinary);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_lpvars {
    ($num_type_left: ty, $trait_name: ident, $f_name: ident, $type_expr: ident, $lp_type_right: ty) => {
        impl $trait_name<$lp_type_right> for $num_type_left {
            type Output = LpExpression;
            fn $f_name(self, var: $lp_type_right) -> LpExpression {
                let new_lp_expr_arena: LpExpression = (self as f32).clone().into();
                let new_right: LpExpression = var.clone().into();
                new_lp_expr_arena.merge_cloned_arenas(&new_right, $type_expr)
            }
        }
        impl<'a> $trait_name<&'a $lp_type_right> for $num_type_left {
            type Output = LpExpression;
            fn $f_name(self, var: &'a $lp_type_right) -> LpExpression {
                let new_lp_expr_arena: LpExpression = (self as f32).into();
                let new_right: LpExpression = (*var).clone().into();
                new_lp_expr_arena.merge_cloned_arenas(&new_right, $type_expr)
            }
        }
    };
}

/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_lpvars {
    ($num_type: ty) => {
        numeric_operation_for_lpvars!($num_type, Add, add, Addition, LpInteger);
        numeric_operation_for_lpvars!($num_type, Add, add, Addition, LpBinary);
        numeric_operation_for_lpvars!($num_type, Add, add, Addition, LpContinuous);
        numeric_operation_for_lpvars!($num_type, Mul, mul, Multiplication, LpInteger);
        numeric_operation_for_lpvars!($num_type, Mul, mul, Multiplication, LpBinary);
        numeric_operation_for_lpvars!($num_type, Mul, mul, Multiplication, LpContinuous);
        numeric_operation_for_lpvars!($num_type, Sub, sub, Subtraction, LpInteger);
        numeric_operation_for_lpvars!($num_type, Sub, sub, Subtraction, LpBinary);
        numeric_operation_for_lpvars!($num_type, Sub, sub, Subtraction, LpContinuous);
    };
}
numeric_all_ops_for_lpvars!(i32);
numeric_all_ops_for_lpvars!(f32);

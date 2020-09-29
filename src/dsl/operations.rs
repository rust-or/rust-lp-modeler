use std::ops::{Add, Mul, Neg, Sub};
use dsl::{Constraint, LpBinary, LpConstraint, LpContinuous, LpExpression, LpAtomicExpr, LpCompExpr, LpInteger, LpExprArena};
use dsl::LpExpression::*;
use dsl::LpAtomicExpr::*;
use dsl::LpExprOp::*;

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
                LpAtomicExpr::LitVal(self as f32)
            }
        }
    };
    ($type_to:ty, $wrapper: ident) => {
        impl Into<LpExpression> for $type_to {
            fn into(self) -> LpExpression {
                LpAtomicExpr::$wrapper(self)
            }
        }
        impl<'a> Into<LpExpression> for &'a $type_to {
            fn into(self) -> LpExpression {
                LpAtomicExpr::$wrapper(self.clone())
            }
        }
    };
}
to_into_expr!(f32);
to_into_expr!(i32);
to_into_expr!(LpBinary, ConsBin);
to_into_expr!(LpInteger, ConsInt);
to_into_expr!(LpContinuous, ConsCont);

/// Macro implementing binary operations for Into<LpExpression> or &Into<LpExpression>
macro_rules! operations_for_expr {
    ($trait_name: ident, $f_name: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, expr: T) -> LpExpression {
                let new_left_index = lp_expr_arena.add_lp_expr(self.clone());
                let new_right_index = lp_expr_arena.add_lp_expr(expr.into());
                LpCompExpr($expr_type, new_left_index, new_right_index)
            }
        }
        impl<'a, T> $trait_name<T> for &'a LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, expr: T) -> LpExpression {
                let new_left_index = lp_expr_arena.add_lp_expr(self.clone());
                let new_right_index = lp_expr_arena.add_lp_expr(expr.into());
                LpCompExpr($expr_type, new_left_index, new_right_index)
            }
        }
    };
}

operations_for_expr!(Add, add, Add);
operations_for_expr!(Sub, sub, Subtract);
operations_for_expr!(Mul, mul, Multiply);

/// Macro implementing a binary operation with a LpVars and a Into<Expression>
macro_rules! lpvars_operation_for_intoexpr {
    ($trait_name: ident, $f_name: ident, $lp_type: ident, $expr_type: ident, $cons_type: ident) => {
        impl<T> $trait_name<T> for $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: &mut LpExprArena, expr: T) -> LpExpression {
                let new_left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::$cons_type(self.clone()));
                let new_right_index = lp_expr_arena.add_lp_expr(expr.into());
                LpCompExpr($expr_type, new_left_index, new_right_index).normalize()
            }
        }
        impl<'a, T> $trait_name<T> for &'a $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, expr: T) -> LpExpression {
                let new_left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::$cons_type(self.clone()));
                let new_right_index = lp_expr_arena.add_lp_expr(expr.into());
                LpCompExpr($expr_type, new_left_index, new_right_index).normalize()
            }
        }
    };
}

lpvars_operation_for_intoexpr!(Mul, mul, LpBinary, Multiply, ConsBin);
lpvars_operation_for_intoexpr!(Add, add, LpBinary, Add, ConsBin);
lpvars_operation_for_intoexpr!(Sub, sub, LpBinary, Subtract, ConsBin);
lpvars_operation_for_intoexpr!(Mul, mul, LpInteger, Multiply, ConsInt);
lpvars_operation_for_intoexpr!(Add, add, LpInteger, Add, ConsInt);
lpvars_operation_for_intoexpr!(Sub, sub, LpInteger, Subtract, ConsInt);
lpvars_operation_for_intoexpr!(Mul, mul, LpContinuous, Multiply, ConsCont);
lpvars_operation_for_intoexpr!(Add, add, LpContinuous, Add, ConsCont);
lpvars_operation_for_intoexpr!(Sub, sub, LpContinuous, Subtract, ConsCont);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_expr {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident) => {
        impl $trait_name<LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: &mut LpExprArena, expr: LpExpression) -> LpExpression {
                let new_left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::LitVal(self as f32));
                let new_right_index = lp_expr_arena.add_lp_expr(expr);
                LpCompExpr($type_expr, new_left_index, new_right_index)
            }
        }
        impl<'a> $trait_name<&'a LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: &mut LpExprArena, expr: &'a LpExpression) -> LpExpression {
                let new_left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::LitVal(self as f32));
                let new_right_index = lp_expr_arena.add_lp_expr(expr.clone());
                LpCompExpr($type_expr, new_left_index, new_right_index)
            }
        }
    };
}
/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_expr {
    ($num_type: ty) => {
        numeric_operation_for_expr!($num_type, Add, add, Add);
        numeric_operation_for_expr!($num_type, Mul, mul, Multiply);
        numeric_operation_for_expr!($num_type, Sub, sub, Subtract);
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
//        LpCompExpr(
//            Multiply,
//            LitVal(-1.0),
//            self
//        )
//    }
//}
macro_rules! neg_operation_for_lpvars {
    ($lp_var_type: ty, $constr_expr: ident) => {
        impl<'a> Neg for &'a $lp_var_type {
            type Output = LpExpression;
            fn neg(self, lp_expr_arena: &mut LpExprArena) -> LpExpression {
                let left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::LitVal(-1.0));
                let right_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::$constr_expr(self.clone()));
                LpCompExpr(
                    Multiply,
                    left_index,
                    right_index
                )
            }
        }
    };
}
neg_operation_for_lpvars!(LpInteger, ConsInt);
neg_operation_for_lpvars!(LpContinuous, ConsCont);
neg_operation_for_lpvars!(LpBinary, ConsBin);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_lpvars {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident, $lp_type: ty, $cons_expr: ident) => {
        impl $trait_name<$lp_type> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: &mut LpExprArena, var: $lp_type) -> LpExpression {
                let left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::LitVal(self as f32));
                let right_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::$cons_expr(var));
                LpCompExpr(
                    $type_expr,
                    left_index,
                    right_index
                )
            }
        }
        impl<'a> $trait_name<&'a $lp_type> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, lp_expr_arena: &mut LpExprArena, var: &'a $lp_type) -> LpExpression {
                let left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::LitVal(self as f32));
                let right_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::$cons_expr(var.clone()));
                LpCompExpr(
                    $type_expr,
                    left_index,
                    right_index
                )
            }
        }
    };
}

/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_lpvars {
    ($num_type: ty) => {
        numeric_operation_for_lpvars!($num_type, Add, add, Add,  LpInteger, ConsInt);
        numeric_operation_for_lpvars!($num_type, Add, add, Add,  LpBinary, ConsBin);
        numeric_operation_for_lpvars!($num_type, Add, add, Add,  LpContinuous, ConsCont);
        numeric_operation_for_lpvars!($num_type, Mul, mul, Multiply,  LpInteger, ConsInt);
        numeric_operation_for_lpvars!($num_type, Mul, mul, Multiply,  LpBinary, ConsBin);
        numeric_operation_for_lpvars!($num_type, Mul, mul, Multiply,  LpContinuous, ConsCont);
        numeric_operation_for_lpvars!($num_type, Sub, sub, Subtract,  LpInteger, ConsInt);
        numeric_operation_for_lpvars!($num_type, Sub, sub, Subtract,  LpBinary, ConsBin);
        numeric_operation_for_lpvars!($num_type, Sub, sub, Subtract,  LpContinuous, ConsCont);
    };
}
numeric_all_ops_for_lpvars!(i32);
numeric_all_ops_for_lpvars!(f32);

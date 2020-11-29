use dsl::LpExpression::*;
use dsl::{Constraint, LpBinary, LpConstraint, LpContinuous, LpExpression, LpInteger};
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Operations trait for any type implementing Into<LpExpressions> trait
pub trait LpOperations<T> where T: Into<LpExpression> {
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
                LitVal(self as f32)
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
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type(Box::new(self.clone()), Box::new(_rhs.into()))
            }
        }
        impl<'a, T> $trait_name<T> for &'a LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type(Box::new(self.clone()), Box::new(_rhs.into()))
            }
        }
    };
}

operations_for_expr!(Add, add, AddExpr);
operations_for_expr!(Sub, sub, SubExpr);
operations_for_expr!(Mul, mul, MulExpr);

macro_rules! assign_operations_for_expr {
    ($trait_name: ident, $f_name: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for LpExpression
        where
            T: Into<LpExpression> + Clone,
        {
            fn $f_name(&mut self, _rhs: T) {
                *self = $expr_type(Box::new(self.clone()), Box::new(_rhs.into()));
            }
        }
    };
}

assign_operations_for_expr!(AddAssign, add_assign, AddExpr);
assign_operations_for_expr!(SubAssign, sub_assign, SubExpr);
assign_operations_for_expr!(MulAssign, mul_assign, MulExpr);

/// Macro implementing a binary operation with a LpVars and a Into<Expression>
macro_rules! lpvars_operation_for_intoexpr {
    ($trait_name: ident, $f_name: ident, $lp_type: ident, $expr_type: ident, $cons_type: ident) => {
        impl<T> $trait_name<T> for $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type(Box::new($cons_type(self.clone())), Box::new(_rhs.into())).normalize()
            }
        }
        impl<'a, T> $trait_name<T> for &'a $lp_type
        where
            T: Into<LpExpression> + Clone,
        {
            type Output = LpExpression;
            fn $f_name(self, _rhs: T) -> LpExpression {
                $expr_type(Box::new($cons_type(self.clone())), Box::new(_rhs.into())).normalize()
            }
        }
    };
}

lpvars_operation_for_intoexpr!(Mul, mul, LpBinary, MulExpr, ConsBin);
lpvars_operation_for_intoexpr!(Add, add, LpBinary, AddExpr, ConsBin);
lpvars_operation_for_intoexpr!(Sub, sub, LpBinary, SubExpr, ConsBin);
lpvars_operation_for_intoexpr!(Mul, mul, LpInteger, MulExpr, ConsInt);
lpvars_operation_for_intoexpr!(Add, add, LpInteger, AddExpr, ConsInt);
lpvars_operation_for_intoexpr!(Sub, sub, LpInteger, SubExpr, ConsInt);
lpvars_operation_for_intoexpr!(Mul, mul, LpContinuous, MulExpr, ConsCont);
lpvars_operation_for_intoexpr!(Add, add, LpContinuous, AddExpr, ConsCont);
lpvars_operation_for_intoexpr!(Sub, sub, LpContinuous, SubExpr, ConsCont);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_expr {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident) => {
        impl $trait_name<LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: LpExpression) -> LpExpression {
                $type_expr(Box::new(LitVal(self as f32)), Box::new(_rhs))
            }
        }
        impl<'a> $trait_name<&'a LpExpression> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: &'a LpExpression) -> LpExpression {
                $type_expr(Box::new(LitVal(self as f32)), Box::new(_rhs.clone()))
            }
        }
    };
}
/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_expr {
    ($num_type: ty) => {
        numeric_operation_for_expr!($num_type, Add, add, AddExpr);
        numeric_operation_for_expr!($num_type, Mul, mul, MulExpr);
        numeric_operation_for_expr!($num_type, Sub, sub, SubExpr);
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

impl<'a> Neg for &'a LpExpression {
    type Output = LpExpression;
    fn neg(self) -> LpExpression {
        MulExpr(Box::new(LitVal(-1.0)), Box::new(self.clone()))
    }
}
macro_rules! neg_operation_for_lpvars {
    ($lp_var_type: ty, $constr_expr: ident) => {
        impl<'a> Neg for &'a $lp_var_type {
            type Output = LpExpression;
            fn neg(self) -> LpExpression {
                MulExpr(Box::new(LitVal(-1.0)), Box::new($constr_expr(self.clone())))
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
            fn $f_name(self, _rhs: $lp_type) -> LpExpression {
                $type_expr(Box::new(LitVal(self as f32)), Box::new($cons_expr(_rhs)))
            }
        }
        impl<'a> $trait_name<&'a $lp_type> for $num_type {
            type Output = LpExpression;
            fn $f_name(self, _rhs: &'a $lp_type) -> LpExpression {
                $type_expr(
                    Box::new(LitVal(self as f32)),
                    Box::new($cons_expr(_rhs.clone())),
                )
            }
        }
    };
}

/// Macro implementing add, mul and sub for a specific numeric type
macro_rules! numeric_all_ops_for_lpvars {
    ($num_type: ty) => {
        numeric_operation_for_lpvars!($num_type, Add, add, AddExpr, LpInteger, ConsInt);
        numeric_operation_for_lpvars!($num_type, Add, add, AddExpr, LpBinary, ConsBin);
        numeric_operation_for_lpvars!($num_type, Add, add, AddExpr, LpContinuous, ConsCont);
        numeric_operation_for_lpvars!($num_type, Mul, mul, MulExpr, LpInteger, ConsInt);
        numeric_operation_for_lpvars!($num_type, Mul, mul, MulExpr, LpBinary, ConsBin);
        numeric_operation_for_lpvars!($num_type, Mul, mul, MulExpr, LpContinuous, ConsCont);
        numeric_operation_for_lpvars!($num_type, Sub, sub, SubExpr, LpInteger, ConsInt);
        numeric_operation_for_lpvars!($num_type, Sub, sub, SubExpr, LpBinary, ConsBin);
        numeric_operation_for_lpvars!($num_type, Sub, sub, SubExpr, LpContinuous, ConsCont);
    };
}
numeric_all_ops_for_lpvars!(i32);
numeric_all_ops_for_lpvars!(f32);

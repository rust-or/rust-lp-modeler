use std::ops::{Add, Mul, Neg, Sub};
use dsl::LpExpression::*;
use dsl::{Constraint, LpBinary, LpConstraint, LpContinuous, LpExpression, LpInteger, LpExprArena};
use dsl::LpExprOp::{Addition, Subtraction, Multiplication};

/// Operations trait for any type implementing Into<LpExprArena> trait
pub trait LpOperations<T> where T: Into<LpExprArena> {
    /// Less or equal binary syntax for LpExprArena
    fn le(&self, lhs_expr: T) -> LpConstraint;
    /// Greater or equal binary syntax for LpExprArena
    fn ge(&self, lhs_expr: T) -> LpConstraint;
    /// Equality binary syntax for LpExprArena
    fn equal(&self, lhs_expr: T) -> LpConstraint;
}


// struct OperatorOverloadableType<O: Into<LpExprArena> + Clone>(O);
//
// impl<O: Into<LpExprArena> + Clone> From<OperatorOverloadableType<O>> for LpExprArena {
//     fn from(from: OperatorOverloadableType<O>) -> Self{
//         let lp_expr_arena: LpExprArena = from.into();
//         lp_expr_arena
//     }
// }
//
// impl<T: Into<LpExprArena> + Clone, O> Add<T> for OperatorOverloadableType<O> where O: Into<LpExprArena> + Clone {
//     type Output = LpExprArena;
//     fn add(self, right_unknown: T) -> LpExprArena {
//         let left_lp_expr_arena: LpExprArena = self.into();
//         let right_lp_expr_arena: LpExprArena = right_unknown.into();
//         left_lp_expr_arena.merge(&right_lp_expr_arena, Addition)
//     }
// }
//
// //impl<'a, 'b, O, T: Into<LpExprArena> + Clone> Add<&'b T> for OperatorOverloadableType<&'a O> where &'a O: Into<LpExprArena> + Clone {
// //    type Output = LpExprArena;
// //    fn add(self, right_unknown: &'b T) -> LpExprArena {
// //        let left_lp_expr_arena: LpExprArena = (*self).into();
// //        let right_lp_expr_arena: LpExprArena = (*right_unknown).into();
// //        left_lp_expr_arena.merge(&right_lp_expr_arena, Addition)
// //    }
// //}
//
// impl<T: Into<LpExprArena> + Clone, O> Sub<T> for OperatorOverloadableType<O> where O: Into<LpExprArena> + Clone {
//     type Output = LpExprArena;
//     fn sub(self, right_unknown: T) -> LpExprArena {
//         let left_lp_expr_arena: LpExprArena = self.into();
//         let right_lp_expr_arena: LpExprArena = right_unknown.into();
//         left_lp_expr_arena.merge(&right_lp_expr_arena, Subtraction)
//     }
// }
//
// impl<T: Into<LpExprArena> + Clone, O> Mul<T> for OperatorOverloadableType<O> where O: Into<LpExprArena> + Clone {
//     type Output = LpExprArena;
//     fn mul(self, right_unknown: T) -> LpExprArena {
//         let left_lp_expr_arena: LpExprArena = self.into();
//         let right_lp_expr_arena: LpExprArena = right_unknown.into();
//         left_lp_expr_arena.merge(&right_lp_expr_arena, Multiplication)
//     }
// }

//// Macro implementing binary operations for LpExprArena
//macro_rules! numeric_operation_for_expr {
//    ($trait_name: ident, $f_name: ident, $op: ident, $type_from_left: ty, $type_from_right: ty) => {
//        impl $trait_name<$type_from_left> for $type_from_right {
//            type Output = LpExprArena;
//            fn $f_name(self, right_unknown: $type_from_left) -> LpExprArena {
//                let left_lp_expr_arena: LpExprArena = self.into();
//                let right_lp_expr_arena: LpExprArena = right_unknown.into();
//                left_lp_expr_arena.merge(&right_lp_expr_arena, $op)
//            }
//        }
//        impl<'a> $trait_name<&'a $type_from_left> for $type_from_right {
//            type Output = LpExprArena;
//            fn $f_name(self, right_unknown: &'a $type_from_right) -> LpExprArena {
//                let left_lp_expr_arena: LpExprArena = self.into();
//                let right_lp_expr_arena: LpExprArena = right_unknown.into();
//                left_lp_expr_arena.merge(&right_lp_expr_arena, $op)
//            }
//        }
//    };
//}
//// Macro implementing add, mul and sub for LpExprArena
//
//macro_rules! all_numeric_operations {
//    ($type_left: ty, $type_right: ty) => {
//        numeric_operation_for_expr!(Add, add, Addition, $type_left, $type_right);
//        numeric_operation_for_expr!(Mul, mul, Multiplication, $type_left, $type_right);
//        numeric_operation_for_expr!(Sub, sub, Subtraction, $type_left, $type_right);
//    }
//}
//
//macro_rules! all_type_combinations_numeric_operations {
//    ($type_right: ty) => {
//        all_numeric_operations!(LpExpression, $type_right);
//        all_numeric_operations!(LpInteger, $type_right);
//        all_numeric_operations!(LpBinary, $type_right);
//        all_numeric_operations!(LpContinuous, $type_right);
//    }
//}
//
//all_type_combinations_numeric_operations!(LpExpression);
//all_type_combinations_numeric_operations!(LpInteger);
//all_type_combinations_numeric_operations!(LpBinary);
//all_type_combinations_numeric_operations!(LpContinuous);

/// Macro implementing binary operations for Into<LpExprArena> or &Into<LpExprArena>
macro_rules! operations_for_expr {
    ($trait_name: ident, $f_name: ident, $expr_type: ident) => {
        impl<T> $trait_name<T> for LpExprArena
        where
            T: Into<LpExprArena> + Clone,
        {
            type Output = LpExprArena;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExprArena {
                let mut new_lp_expr_arena = self.clone();
                new_lp_expr_arena.merge(&not_yet_lp_expr_arena.into(), $expr_type)
            }
        }
        impl<'a, T> $trait_name<T> for &'a LpExprArena
        where
            T: Into<LpExprArena> + Clone,
        {
            type Output = LpExprArena;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExprArena {
                let mut new_lp_expr_arena = (*self).clone();
                new_lp_expr_arena.merge(&not_yet_lp_expr_arena.into(), $expr_type)
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
            T: Into<LpExprArena> + Clone,
        {
            type Output = LpExprArena;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExprArena {
                let mut new_lp_expr_arena: LpExprArena = self.clone().into();
                new_lp_expr_arena.merge(&not_yet_lp_expr_arena.into(), $expr_type)
            }
        }
        impl<'a, T> $trait_name<T> for &'a $lp_type
        where
            T: Into<LpExprArena> + Clone,
        {
            type Output = LpExprArena;
            fn $f_name(self, not_yet_lp_expr_arena: T) -> LpExprArena {
                let mut new_lp_expr_arena: LpExprArena = (*self).clone().into();
                new_lp_expr_arena.merge(&not_yet_lp_expr_arena.into(), $expr_type)
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
        impl $trait_name<LpExprArena> for $num_type {
            type Output = LpExprArena;
            fn $f_name(self, lp_expr_arena: LpExprArena) -> LpExprArena {
                let mut new_lp_expr_arena: LpExprArena = lp_expr_arena.clone();
                new_lp_expr_arena.merge(&(self as f32).into(), $type_expr)
            }
        }
        impl<'a> $trait_name<&'a LpExprArena> for $num_type {
            type Output = LpExprArena;
            fn $f_name(self, lp_expr_arena: &'a LpExprArena) -> LpExprArena {
                let mut new_lp_expr_arena: LpExprArena = (*lp_expr_arena).clone();
                new_lp_expr_arena.merge(&(self as f32).into(), $type_expr)
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

/// &LpExprArena to LpExprArena
impl<'a> Into<LpExprArena> for &'a LpExprArena {
    fn into(self) -> LpExprArena {
        (*self).clone()
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
//            Multiplication,
//            LitVal(-1.0),
//            self
//        )
//    }
//}
//macro_rules! neg_operation_for_lpvars {
//    ($lp_var_type: ty, $constr_expr: ident) => {
//        impl<'a> Neg for &'a $lp_var_type {
//            type Output = LpExpression;
//            fn neg(self, lp_expr_arena: &mut LpExprArena) -> LpExpression {
//                let left_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::LitVal(-1.0));
//                let right_index = lp_expr_arena.add_lp_expr(LpAtomicExpr::$constr_expr(self.clone()));
//                LpCompExpr(
//                    Multiplication,
//                    left_index,
//                    right_index
//                )
//            }
//        }
//    };
//}
//neg_operation_for_lpvars!(LpInteger, ConsInt);
//neg_operation_for_lpvars!(LpContinuous, ConsCont);
//neg_operation_for_lpvars!(LpBinary, ConsBin);

/// Macro implementing binary operations for a numeric type
macro_rules! numeric_operation_for_lpvars {
    ($num_type: ty, $trait_name: ident, $f_name: ident, $type_expr: ident, $lp_type: ty) => {
        impl $trait_name<$lp_type> for $num_type {
            type Output = LpExprArena;
            fn $f_name(self, var: $lp_type) -> LpExprArena {
                let mut new_lp_expr_arena: LpExprArena = var.clone().into();
                new_lp_expr_arena.merge(&(self as f32).into(), $type_expr)
            }
        }
        impl<'a> $trait_name<&'a $lp_type> for $num_type {
            type Output = LpExprArena;
            fn $f_name(self, var: &'a $lp_type) -> LpExprArena {
                let mut new_lp_expr_arena: LpExprArena = (*var).clone().into();
                new_lp_expr_arena.merge(&(self as f32).into(), $type_expr)
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

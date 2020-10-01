/// # Module variables
///
use self::LpExpression::*;

use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};

use std::convert::Into;
use std::ops::{Add, Mul, Sub};
use std::collections::HashMap;

pub trait BoundableLp: PartialEq + Clone {
    fn lower_bound(&self, lw: f32) -> Self;
    fn upper_bound(&self, up: f32) -> Self;
}

// A binary variable is constrained to be either 1 or 0. Refer to the
// [LP format documentation](https://www.gurobi.com/documentation/8.0/refman/variables.html)
// for details.
#[derive(Debug, Clone, PartialEq)]
pub struct LpBinary {
    pub name: String,
}
impl LpBinary {
    pub fn new(name: &str) -> LpBinary {
        LpBinary {
            name: name.to_string(),
        }
    }
}

impl ToTokens for LpBinary {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let name = &self.name;
        stream.extend(quote! {
            LpBinary{
                name: #name,
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpInteger {
    pub name: String,
    pub lower_bound: Option<f32>,
    pub upper_bound: Option<f32>,
}
impl LpInteger {
    pub fn new(name: &str) -> LpInteger {
        LpInteger {
            name: name.to_string(),
            lower_bound: None,
            upper_bound: None,
        }
    }
}
impl ToTokens for LpInteger {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let name = &self.name;
        let lower_bound = match self.lower_bound {
            Some(ref v) => quote!(Some(#v)),
            None => quote!(None)
        };
        let upper_bound = match self.upper_bound {
            Some(ref v) => quote!(Some(#v)),
            None => quote!(None)
        };
        stream.extend(quote! {
            LpInteger{
                name: #name.to_string(),
                lower_bound: #lower_bound,
                upper_bound: #upper_bound
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpContinuous {
    pub name: String,
    pub lower_bound: Option<f32>,
    pub upper_bound: Option<f32>,
}
impl LpContinuous {
    pub fn new(name: &str) -> LpContinuous {
        LpContinuous {
            name: name.to_string(),
            lower_bound: None,
            upper_bound: None,
        }
    }
}
impl ToTokens for LpContinuous {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let name = &self.name;
        let lower_bound = match self.lower_bound {
            Some(ref v) => quote!(Some(#v)),
            None => quote!(None)
        };
        let upper_bound = match self.upper_bound {
            Some(ref v) => quote!(Some(#v)),
            None => quote!(None)
        };
        stream.extend(quote! {
            LpContinuous{
                name: #name.to_string(),
                lower_bound: #lower_bound,
                upper_bound: #upper_bound
            }
        });
    }
}

macro_rules! implement_boundable {
    ($lp_type: ident) => {
        impl BoundableLp for $lp_type {
            fn lower_bound(&self, lw: f32) -> $lp_type {
                $lp_type {
                    name: self.name.clone(),
                    lower_bound: Some(lw),
                    upper_bound: self.upper_bound,
                }
            }
            fn upper_bound(&self, up: f32) -> $lp_type {
                $lp_type {
                    name: self.name.clone(),
                    lower_bound: self.lower_bound,
                    upper_bound: Some(up),
                }
            }
        }
    };
}
implement_boundable!(LpInteger);
implement_boundable!(LpContinuous);

#[derive(Debug, Clone, PartialEq)]
pub enum LpExprOp {
    Multiplication,
    Addition,
    Subtraction
}

pub type LpExprArenaIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct LpCompExpr {
    operation: LpExprOp,
    left_index: LpExprArenaIndex,
    right_index: LpExprArenaIndex
}

/// ADT for Linear Programming Expression
#[derive(Debug, Clone, PartialEq)]
pub enum LpExpression {
    ConsInt(LpInteger),
    ConsBin(LpBinary),
    ConsCont(LpContinuous),
    LitVal(f32),
    EmptyExpr,
    LpCompExpr(LpExprOp, LpExprArenaIndex, LpExprArenaIndex)
}

impl LpExpression {
    /// Fix the numeric operand in a multiplication in an expression
    /// c * 4 must be considered as 4 c in a linear formulation lp file
    pub fn normalize(self, lp_expr_arena: &LpExprArena) -> LpExpression {
        if let LpCompExpr(LpExprOp::Multiplication, e1, e2) = self {
            if let LpExpression::LitVal(_) = lp_expr_arena.clone_expr_at(e1) {
                return self.clone();
            } else {
                if let LpExpression::LitVal(_) = lp_expr_arena.clone_expr_at(e2) {
                    return LpExpression::LpCompExpr(LpExprOp::Multiplication, e2, e1);
                } else {
                    return LpExpression::LpCompExpr(LpExprOp::Multiplication, e1, e2);
                }
            }
        } else {
            self
        }
    }
}

// Macro implementing Into<LpExpression> for types
macro_rules! cons_into_expr {
    ($type_from:ty, $wrapper: ident) => {
        impl From<$type_from> for LpExpression {
            fn from(from: $type_from) -> Self {
                $wrapper(from)
            }
        }
        impl<'a> From<&'a $type_from> for LpExpression {
            fn from(from: &'a $type_from) -> Self {
                $wrapper((*from).clone())
            }
        }
    };
}
cons_into_expr!(LpBinary, ConsBin);
cons_into_expr!(LpInteger, ConsInt);
cons_into_expr!(LpContinuous, ConsCont);

macro_rules! lit_into_expr {
    ($type_from:ty) => {
        impl From<$type_from> for LpExpression {
            fn from(from: $type_from) -> Self {
                    LitVal(from as f32)
            }
        }
        impl<'a> From<&'a $type_from> for LpExpression {
            fn from(from: &'a $type_from) -> Self {
                    LitVal((*from).clone() as f32)
            }
        }
    };
}
lit_into_expr!(f32);
lit_into_expr!(i32);

#[derive(Debug, Clone, PartialEq)]
pub struct LpExprArena {
    root: LpExprArenaIndex,
    array: Vec<LpExpression>
}

// Macro implementing Into<LpExprArena> for types
macro_rules! cons_into_expr_arena {
    ($type_from:ty, $wrapper: ident) => {
        impl From<$type_from> for LpExprArena {
            fn from(from: $type_from) -> Self {
                LpExprArena {
                    root: 0,
                    array: vec![$wrapper(from); 1]
                }
            }
        }
        impl<'a> From<&'a $type_from> for LpExprArena {
            fn from(from: &'a $type_from) -> Self {
                LpExprArena {
                    root: 0,
                    array: vec![$wrapper((*from).clone()); 1]
                }
            }
        }
    };
}
cons_into_expr_arena!(LpBinary, ConsBin);
cons_into_expr_arena!(LpInteger, ConsInt);
cons_into_expr_arena!(LpContinuous, ConsCont);

macro_rules! lit_into_expr_arena {
    ($type_from:ty) => {
        impl From<$type_from> for LpExprArena {
            fn from(from: $type_from) -> Self {
                LpExprArena {
                    root: 0,
                    array: vec![LitVal(from as f32); 1]
                }
            }
        }
        impl<'a> From<&'a $type_from> for LpExprArena {
            fn from(from: &'a $type_from) -> Self {
                LpExprArena {
                    root: 0,
                    array: vec![LitVal((*from).clone() as f32); 1]
                }
            }
        }
    };
}
lit_into_expr_arena!(f32);
lit_into_expr_arena!(i32);

impl From<LpExpression> for LpExprArena {
    fn from(expr: LpExpression) -> Self {
        LpExprArena {
            root: 0,
            array: vec![expr; 1]
        }
    }
}

impl From<&LpExpression> for LpExprArena {
    fn from(expr: &LpExpression) -> Self {
        LpExprArena {
            root: 0,
            array: vec![expr.clone(); 1]
        }
    }
}

impl LpExprArena {
    pub fn new() -> Self {
       LpExprArena {
           root: 0,
           array: Vec::new()
       }
    }

    pub fn get_root_index(&self) -> LpExprArenaIndex {
        self.root
    }

    pub fn set_root(&mut self, root_index: LpExprArenaIndex) {
        self.root = root_index;
    }

    pub fn add_lp_expr<T>(&mut self, lp_expr: &T) -> LpExprArenaIndex where T: Into<LpExpression> + Clone {
        let index = self.array.len();
        self.array.push(lp_expr.clone().into());
        return index
    }

    pub fn clone_and_push_from_index(&mut self, index: LpExprArenaIndex) -> LpExprArenaIndex {
        let new_index = self.array.len();
        self.array.push(self.clone_expr_at(index).clone());
        return new_index
    }

    pub fn change_lp_expr(&mut self, index: LpExprArenaIndex, lp_expr: LpExpression) {
       self.array[index] = lp_expr;
    }

    pub fn expr_ref_at(&self, index: LpExprArenaIndex) -> &LpExpression {
        match self.array.get(index) {
            Some(expr) => expr,
            None => panic!("Requested index out of bound of LpExprArena vector. This should not happen.")
        }
    }

    pub fn clone_expr_at(&self, index: LpExprArenaIndex) -> LpExpression {
        match self.array.get(index) {
            Some(expr) => expr.clone(),
            None => panic!("Requested index out of bound of LpExprArena vector. This should not happen.")
        }
    }

    pub fn get_root_expr(&self) -> LpExpression {
        self.clone_expr_at(self.root)
    }

    pub fn get_root_expr_ref(&self) -> &LpExpression {
        self.expr_ref_at(self.root)
    }

    pub fn split_off_constant(&mut self) -> f32 {
        match self.get_root_expr() {
            LpCompExpr(LpExprOp::Addition, e1, e2) => {
                if let LpExpression::LitVal(c) = self.clone_expr_at(e2) {
                    self.set_root(e1);
                    c
                } else {
                    0.0
                }
            },
            LpCompExpr(LpExprOp::Subtraction, e1, e2) => {
                if let LpExpression::LitVal(c) = self.clone_expr_at(e2) {
                    self.set_root(e1);
                    -c
                } else {
                    0.0
                }
            },
            _ => 0.0
        }
    }

    pub fn merge(&self, move_lp_expr_arena: &LpExprArena, operation: LpExprOp) -> Self {
        let move_root_expr_ref = move_lp_expr_arena.get_root_expr_ref();
        let mut new_lp_expr_arena = (*self).clone();
        let moved_root_index = new_lp_expr_arena.add_lp_expr(move_root_expr_ref);
        let new_root_index = new_lp_expr_arena.add_lp_expr(
            &LpExpression::LpCompExpr(
                operation,
                self.root,
                moved_root_index
            )
        );
        new_lp_expr_arena.set_root(new_root_index);
        let mut move_stack: Vec<LpExprArenaIndex> = Vec::new();
        move_stack.push(moved_root_index);
        while let Some(self_parent_index) = move_stack.pop() {
            match self.expr_ref_at(self_parent_index) {
                LpCompExpr(operation, move_left_index, move_right_index) => {
                    let new_left_index = new_lp_expr_arena.add_lp_expr(move_lp_expr_arena.expr_ref_at(*move_left_index));
                    let new_right_index = new_lp_expr_arena.add_lp_expr(move_lp_expr_arena.expr_ref_at(*move_right_index));
                    new_lp_expr_arena.change_lp_expr(
                        self_parent_index,
                        LpExpression::LpCompExpr(
                            operation.clone(),
                            new_left_index,
                            new_right_index
                        )
                    );
                    move_stack.push(new_left_index);
                    move_stack.push(new_right_index);
                },
                // done for this branch
                _ => {}
            }
        }
        new_lp_expr_arena
    }

    pub fn show(&self, e: &LpExprArenaIndex, with_parenthesis: bool) -> String {
        let str_left_mult = if with_parenthesis { "(" } else { "" };
        let str_right_mult = if with_parenthesis { ")" } else { "" };
        let str_op_mult = if with_parenthesis { " * " } else { " " };
        match self.expr_ref_at(*e) {
            LpExpression::LitVal(n) => n.to_string(),
            LpExpression::LpCompExpr(LpExprOp::Addition, e1, e2) => {
                str_left_mult.to_string()
                    + &self.show(e1, with_parenthesis)
                    + " + "
                    + &self.show(e2, with_parenthesis)
                    + str_right_mult
            }
            LpExpression::LpCompExpr(LpExprOp::Subtraction, e1, e2) => {
                str_left_mult.to_string()
                    + &self.show(e1, with_parenthesis)
                    + " - "
                    + &self.show(e2, with_parenthesis)
                    + str_right_mult
            }
            LpExpression::LpCompExpr(LpExprOp::Multiplication, e1, e2) => {
                match self.expr_ref_at(*e1) {
                    LpExpression::LitVal(1.0) => {
                        //e2.to_lp_file_format()
                        str_left_mult.to_string()
                            + &" ".to_string()
                            + &self.show(e2, with_parenthesis)
                            + str_right_mult
                    },
                    LpExpression::LitVal(-1.0) => {
                        //"-".to_string() + &e2.to_lp_file_format()
                        str_left_mult.to_string()
                            + &"-".to_string()
                            + &self.show(e2, with_parenthesis)
                            + str_right_mult
                    }
                    _ => {
                        str_left_mult.to_string()
                            + &self.show(e1, with_parenthesis)
                            + str_op_mult
                            + &self.show(e2, with_parenthesis)
                            + str_right_mult
                    }
                }
            }
            LpExpression::ConsBin(LpBinary { name: ref n, .. }) => n.to_string(),
            LpExpression::ConsInt(LpInteger { name: ref n, .. }) => n.to_string(),
            LpExpression::ConsCont(LpContinuous { name: ref n, .. }) => n.to_string(),
            _ => "EmptyExpr!!".to_string(),
        }
    }

    pub fn simplify(&mut self) -> &mut Self {
        // keep clone of the starting expression to compare once recursive iteration finishes
        let mut starting_root = self.get_root_expr();
        let mut first_round = true;

        let mut lp_expr_stack: Vec<LpExprArenaIndex> = Vec::new();

        while first_round
            // check whether the root has changed -- if yes, do another round
            || ( starting_root != self.get_root_expr()) {
            starting_root = self.get_root_expr();
            first_round = false;
            lp_expr_stack.push(self.get_root_index());
            while let Some(handled_expr_index) = lp_expr_stack.pop() {
                    match self.clone_expr_at(handled_expr_index) {
                        LpCompExpr(LpExprOp::Multiplication, left_index, right_index) => {
                            match (self.expr_ref_at(left_index), self.expr_ref_at(right_index)) {
                                // Trivial rule: 0 * x = 0
                                (_, LpExpression::LitVal(0.0))
                                | (LpExpression::LitVal(0.0), _) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LitVal(0.0)
                                    )
                                },

                                // Simplify two literals
                                (LpExpression::LitVal(c1), LpExpression::LitVal(c2)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LitVal(c1 * c2)
                                    )
                                },

                                // DISTRIBUTIVITY
                                // i*(a+b) = i*a+i*b
                                (i, &LpCompExpr(LpExprOp::Addition, a_index, b_index))
                                // (a+b)*i = i*a+i*b
                                | (&LpCompExpr(LpExprOp::Addition, a_index, b_index), i) => {
                                    let i_new_index = self.add_lp_expr(&i.clone());
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiplication, i_new_index, a_index)
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiplication, i_new_index, b_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Addition, left_index, right_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }
                                // i*(a-b) = i*a-i*b
                                (i, &LpCompExpr(LpExprOp::Subtraction, a_index, b_index))
                                // (a-b)*i = i*a-i*b
                                | (&LpCompExpr(LpExprOp::Subtraction, a_index, b_index), i) => {
                                    let i_new_index = self.add_lp_expr(&i.clone());
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiplication, i_new_index, a_index)
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiplication, i_new_index, b_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtraction, left_index, right_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // COMMUTATIVITY WITH CONSTANTS
                                // c1*Multiply
                                (&LpExpression::LitVal(c1), &LpCompExpr(LpExprOp::Multiplication, a_index, b_index)) => {
                                    match (self.expr_ref_at(a_index), self.expr_ref_at(b_index)) {
                                        // c1*(c2*x) = (c1*c2)*x
                                        (&LpExpression::LitVal(c2), _) => {
                                        // c1*(x*c2) = (c1*c2)*x
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LitVal(c1 * c2),
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(
                                                    LpExprOp::Multiplication,
                                                    left_index,
                                                    b_index
                                                )
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        (_, &LpExpression::LitVal(c2)) => {
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LitVal(c1 * c2),
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(
                                                    LpExprOp::Multiplication,
                                                    left_index,
                                                    a_index
                                                )
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        // c1*(a*b) = (c1*a)*b
                                        (_, _) => {
                                            let lit_new_index = self.add_lp_expr(&LpExpression::LitVal(c1));
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LpCompExpr(
                                                    LpExprOp::Multiplication,
                                                    lit_new_index,
                                                    a_index
                                                )
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(
                                                    LpExprOp::Multiplication,
                                                    left_index,
                                                    b_index
                                                )
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        }
                                    }
                                },
                                //TODO: (Pointless?)
                                // expr1*(c*expr) = c*(expr1*expr2)

                                // COMMUTATIVITY
                                // x*(a*b) = (x*a)*b
                                (_, &LpCompExpr(LpExprOp::Multiplication, a_index, b_index)) => {
                                    let left_new_index = self.clone_and_push_from_index(left_index);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiplication, left_new_index, a_index),
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        self.clone_expr_at(b_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Place literal first for *
                                (_, &LpExpression::LitVal(c1)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(
                                            LpExprOp::Multiplication,
                                            right_index,
                                            left_index
                                        )
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                (_, _) => {
                                    lp_expr_stack.push(left_index);
                                    lp_expr_stack.push(right_index);
                                },
                            }
                        },
                        LpCompExpr(LpExprOp::Addition, left_expr_index, right_expr_index) => {
                            match (self.expr_ref_at(left_expr_index), self.expr_ref_at(right_expr_index)) {
                                // Trivial rule: 0 + x = x
                                (&LpExpression::LitVal(0.0), a)
                                // Trivial rule: x + 0 = x
                                | (a, &LpExpression::LitVal(0.0)) => {
                                    self.change_lp_expr(handled_expr_index, a.clone());
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Simplify two literals
                                (&LpExpression::LitVal(c1), &LpExpression::LitVal(c2)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LitVal(c1 + c2)
                                    );
                                },

                                // ASSOCIATIVITY
                                // a + (b+c) = (a+b)+c
                                (a, &LpCompExpr(LpExprOp::Addition, b_index, c_index)) => {
                                    let new_a_index = self.add_lp_expr(&a.clone());
                                    self.change_lp_expr(
                                        left_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Addition, new_a_index, b_index),
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        self.clone_expr_at(c_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // a + (b-c) = (a+b) - c
                                (a, &LpCompExpr(LpExprOp::Subtraction, b_index, c_index)) => {
                                    let new_a_index = self.add_lp_expr(&a.clone());
                                    self.change_lp_expr(
                                        left_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Addition, new_a_index, b_index),
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        self.clone_expr_at(c_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtraction, left_expr_index, right_expr_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // Place literal at the end
                                (&LpExpression::LitVal(c), x) => {
                                    self.change_lp_expr(
                                        left_expr_index,
                                        x.clone()
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        LpExpression::LitVal(c)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                    lp_expr_stack.push(left_expr_index);
                                },

                                // Accumulate consts +/-
                                // (a+c1)+c2 = a+(c1+c2)
                                (&LpCompExpr(LpExprOp::Addition, a_index, b_index), &LpExpression::LitVal(c2)) => {
                                    match self.expr_ref_at(b_index) {
                                        &LpExpression::LitVal(c1) => {
                                            self.change_lp_expr(
                                                left_expr_index,
                                                self.clone_expr_at(a_index)
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LitVal(c1 + c2)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {
                                            lp_expr_stack.push(left_expr_index);
                                        },
                                    }
                                }
                                // (a-c1)+c2 = a+(c2-c1)
                                // (c1-b)+c2 = -b+(c1+c2)
                                (&LpCompExpr(LpExprOp::Subtraction, a_index, b_index), &LpExpression::LitVal(c2)) => {
                                    match (self.expr_ref_at(a_index), self.expr_ref_at(b_index)) {
                                        (a, &LpExpression::LitVal(c1)) => {
                                            self.change_lp_expr(
                                                left_expr_index,
                                                a.clone()
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LitVal(c2 - c1)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        }
                                        (&LpExpression::LitVal(c1), _) => {
                                            let lit_new_index = self.add_lp_expr(&LpExpression::LitVal(-1.0));
                                            self.change_lp_expr(
                                                left_expr_index,
                                                LpCompExpr(
                                                    LpExprOp::Subtraction,
                                                    lit_new_index,
                                                    b_index
                                                )
//                                                LpExpression::LpCompExpr(LpExprOp::Subtract, LpExpression::LitVal(0.0), b),
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LitVal(c1 + c2)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {
                                            lp_expr_stack.push(left_expr_index);
                                            // lp_expr_stack.push(&right_expr);
                                        },
                                    }
                                },

                                // Extract the const
                                // (a+c1)+b = (a+b)+c1
                                (&LpCompExpr(LpExprOp::Addition, a, c1_index), b) => {
                                    match self.expr_ref_at(c1_index) {
                                        &LpExpression::LitVal(c1) => {
                                            let new_b_index = self.add_lp_expr(&b.clone());
                                            self.change_lp_expr(
                                                left_expr_index,
                                                LpExpression::LpCompExpr(LpExprOp::Addition, a, new_b_index)
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LitVal(c1)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {}
                                    }
                                },
                                // (a-c1)+b = (a+b)-c1
                                (&LpCompExpr(LpExprOp::Subtraction, a, c1_index), b) => {
                                    match self.expr_ref_at(c1_index) {
                                        &LpExpression::LitVal(c1) => {
                                            let new_b_index = self.add_lp_expr(&b.clone());
                                            self.change_lp_expr(
                                                left_expr_index,
                                                LpExpression::LpCompExpr(LpExprOp::Addition, a, new_b_index)
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LitVal(c1),
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(LpExprOp::Subtraction, left_expr_index, right_expr_index)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {}
                                    }
                                },

                                // Accumulate consts +/-
                                // (expr+c1)+c2 = expr+(c1+c2) OK
                                // (expr-c1)+c2 = expr+(c2-c1) OK
                                // (c1-expr)+c2 = -expr+(c1+c2) OK

                                // (expr-c1)-c2 = expr-(c1+c2) OK
                                // (expr+c1)-c2 = expr+(c1-c2) OK
                                // (c1-expr)-c2 = -expr+(c1-c2) OK

                                // Extract the const
                                // (expr1+c)+expr2 = (expr1+expr2)+c OK
                                // (expr1-c)+expr2 = (expr1+expr2)-c OK
                                // (expr1+c)-expr2 = (expr1-expr2)+c OK
                                // (expr1-c)-expr2 = (expr1-expr2)-c OK
                                _ => {
                                    lp_expr_stack.push(left_expr_index);
                                    lp_expr_stack.push(right_expr_index);
                                },
                            }
                        },
                        LpCompExpr(LpExprOp::Subtraction, left_index, right_index) => {
                            match (self.expr_ref_at(left_index), self.expr_ref_at(right_index)) {
                                // Trivial rule: x - 0 = x
                                (a, &LpExpression::LitVal(0.0)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        a.clone()
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // a - (b + c) = (a-b)-c
                                (_, &LpCompExpr(LpExprOp::Addition, b_index, c_index)) => {
                                    let a_new_index = self.clone_and_push_from_index(left_index);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtraction, a_new_index, b_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtraction, left_index, c_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // a - (b - c) = (a-b)+c
                                (_, &LpCompExpr(LpExprOp::Subtraction, b_index, c_index)) => {
                                    let a_new_index = self.clone_and_push_from_index(left_index);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtraction, a_new_index, b_index),

                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Addition, left_index, c_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Place literal at the end
                                // c1 - b = -b + c1
                                (&LpExpression::LitVal(c1), _) => {
                                    let lit_new_index = self.add_lp_expr(&LpExpression::LitVal(-1.0));
                                    let new_index = self.add_lp_expr(
                                        &LpCompExpr(
                                            LpExprOp::Multiplication,
                                            lit_new_index,
                                            right_index
                                        )
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Addition, new_index, left_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Accumulate consts +/-
                                // (a-c1)-c2 = a-(c1+c2)
                                // (c1-b)-c2 = -b+(c1-c2)
                                (&LpCompExpr(LpExprOp::Subtraction, a_index, b_index), &LpExpression::LitVal(c2)) => {
                                    match (self.expr_ref_at(a_index), self.expr_ref_at(b_index)) {
                                        (a, &LpExpression::LitVal(c1)) => {
                                            self.change_lp_expr(
                                                left_index,
                                                a.clone()
                                            );
                                            self.change_lp_expr(
                                                right_index,
                                                LpExpression::LitVal(c1 + c2)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        (&LpExpression::LitVal(c1), _) => {
                                            let lit_new_index = self.add_lp_expr(&LpExpression::LitVal(-1.0));
                                            self.change_lp_expr(
                                                left_index,
                                                LpCompExpr(
                                                    LpExprOp::Multiplication,
                                                    lit_new_index,
                                                    b_index
                                                )
                                            );
                                            self.change_lp_expr(
                                                right_index,
                                                LpExpression::LitVal(c1 - c2)
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpCompExpr(
                                                    LpExprOp::Addition,
                                                    left_index,
                                                    right_index
                                                )
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {
                                            lp_expr_stack.push(left_index);
                                        },
                                    }
                                }

                                // (a+c1)-c2 = a+(c1-c2)
                                (&LpCompExpr(LpExprOp::Addition, a_index, c1_index), &LpExpression::LitVal(c2)) => {
                                    match self.expr_ref_at(c1_index) {
                                        &LpExpression::LitVal(c1) => {
                                            self.change_lp_expr(
                                                right_index,
                                                LpExpression::LitVal(c1 - c2)
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(LpExprOp::Addition, a_index, right_index)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {}
                                    }

                                },

                                // Extract the const:
                                // (a+c1)-b = (a-b)+c1
                                (&LpCompExpr(LpExprOp::Addition, a_index, c1_index), _) => {
                                    match self.expr_ref_at(c1_index) {
                                        &LpExpression::LitVal(c1) => {
                                            let c1_new_index = self.add_lp_expr(&LpExpression::LitVal(c1));
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LpCompExpr(LpExprOp::Subtraction, a_index, right_index),
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(LpExprOp::Addition, left_index, c1_new_index)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {}
                                    }
                                },
                                // (a-c1)-b = (a-b)-c1
                                (&LpCompExpr(LpExprOp::Subtraction, a_index, c1_index), _) => {
                                    match self.expr_ref_at(c1_index) {
                                        &LpExpression::LitVal(c1) => {
                                            let b_new_index = self.clone_and_push_from_index(right_index);
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LpCompExpr(
                                                    LpExprOp::Subtraction,
                                                    a_index,
                                                    b_new_index
                                                )
                                            );
                                            self.change_lp_expr(
                                                right_index,
                                                LpExpression::LitVal(c1),
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {}
                                    }
                                },

                                _ => {
                                    lp_expr_stack.push(left_index);
                                    lp_expr_stack.push(right_index);
                                },
                            }
                        }
                        LpExpression::ConsBin(_)
                        | LpExpression::ConsInt(_)
                        | LpExpression::ConsCont(_)
                        | LpExpression::LitVal(_)
                        | _ => {},
                    };
            }
        }
        self
    }
}

impl ToTokens for LpExpression {
    fn to_tokens(&self, stream: &mut TokenStream) {
        use self::LpExpression::*;
        stream.extend(
            match self {
                LpExpression::ConsInt(v) => quote!(LpExpression::ConsInt(#v)),
                LpExpression::ConsBin(v) => quote!(LpExpression::ConsBin(#v)),
                LpExpression::ConsCont(v) => quote!(LpExpression::ConsCont(#v)),
                LpCompExpr(LpExprOp::Multiplication, lhs, rhs) => quote!(LpExpression::LpCompExpr(LpExprOp::Multiply, #lhs, #rhs)),
                LpCompExpr(LpExprOp::Addition, lhs, rhs) => quote!(LpExpression::LpCompExpr(LpExprOp::Add, #lhs, #rhs)),
                LpCompExpr(LpExprOp::Subtraction, lhs, rhs) => quote!(LpExpression::LpCompExpr(LpExprOp::Subtract, #lhs, #rhs)),
                LpExpression::LitVal(v) =>  quote!(LpExpression::LitVal(#v)),
                LpExpression::EmptyExpr => quote!(LpExpression::EmptyExpr),
            }
        );
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /* Not supported by solver format files (lp file or mps file) !
    Greater,
    Less,
    */
    GreaterOrEqual,
    LessOrEqual,
    Equal,
}

impl ToTokens for Constraint {
    fn to_tokens(&self, stream: &mut TokenStream) {
        stream.extend(
        match self {
            Constraint::GreaterOrEqual => quote!(Constraint::GreaterOrEqual),
            Constraint::LessOrEqual => quote!(Constraint::LessOrEqual),
            Constraint::Equal => quote!(Constraint::Equal),
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LpConstraint(pub LpExprArena, pub Constraint, pub LpExprArena);

impl LpConstraint {
    pub fn generalize(&self) -> LpConstraint {
        // TODO: Optimize tailrec
        let &LpConstraint(ref lhs, ref op, ref rhs) = self;
        let mut new_lhs_expr = lhs.merge(rhs, LpExprOp::Subtraction);
        let constant = new_lhs_expr.simplify().split_off_constant();
        let new_rhs_expr_arena: LpExprArena= LitVal(-constant).into();
        LpConstraint(new_lhs_expr, (*op).clone(), new_rhs_expr_arena)
    }

    pub fn var(&self, expr_index: LpExprArenaIndex, constraint_index: usize, lst: &mut HashMap<String, (usize, LpExprArenaIndex)>) {
        match self.0.expr_ref_at(expr_index) {
            LpExpression::ConsBin(LpBinary { ref name, .. })
            | LpExpression::ConsInt(LpInteger { ref name, .. })
            | LpExpression::ConsCont(LpContinuous { ref name, .. }) => {
                lst.insert(name.clone(), (constraint_index, expr_index));
            },
            LpExpression::LpCompExpr(LpExprOp::Multiplication, _, e) => {
                self.var(*e, constraint_index, lst);
            },
            LpExpression::LpCompExpr(LpExprOp::Addition, e1, e2)
            | LpExpression::LpCompExpr(LpExprOp::Subtraction, e1, e2) => {
                self.var(*e1, constraint_index, lst);
                self.var(*e2, constraint_index, lst);
            }
            _ => (),
        }
    }
}

impl ToTokens for LpConstraint {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let lhs = &self.0.get_root_expr();
        let constraint = &self.1;
        let rhs = &self.2.get_root_expr();
        stream.extend(quote!(
            LpConstraint(
                #lhs, #constraint, #rhs
            )
        ));
    }
}



/// make a complete expression or a constraint with a vector of expressions
///
/// # Examples
///
/// ```
/// use lp_modeler::dsl::*;
///
/// let mut problem = LpProblem::new("My Problem", LpObjective::Maximize);
/// let ref a = LpBinary::new("a");
/// let ref b = LpBinary::new("b");
/// let ref c = LpBinary::new("c");
///
/// let ref v = vec!(a, b, c);
/// problem += lp_sum(v).equal(10.0);
/// ```
///
pub fn lp_sum<T>(expr: &Vec<T>) -> LpExprArena where T: Into<LpExpression> + Clone {
    let mut arena = LpExprArena::new();
    match expr.first() {
        Some(first) => {
            arena.add_lp_expr(first);
            for a in expr[1..].iter() {
                let index = arena.add_lp_expr(a);
                arena.add_lp_expr(
                    &LpExpression::LpCompExpr(
                        LpExprOp::Addition,
                        index - 1,
                        index
                    )
                );
            }
            arena.set_root(arena.array.len() - 1);
            arena
        },
        None => {
            panic!("vector should have at least one element")
        }
    }
}

pub fn sum<'a, T: 'a,U: 'a>(expr: &'a Vec<T>, f: impl Fn(&'a T) -> U) -> LpExprArena where U: Into<LpExpression> + Clone {
    return lp_sum(&expr.iter().map(|t| f(t.into())).collect());
}

pub trait SummableExp {
    fn sum(&self) -> LpExprArena;
}

/// make a complete expression or a constraint with a vector of expressions
///
/// # Examples
///
/// ```
/// use lp_modeler::dsl::*;
///
/// let mut problem = LpProblem::new("My Problem", LpObjective::Maximize);
/// let ref a = LpBinary::new("a");
/// let ref b = LpBinary::new("b");
/// let ref c = LpBinary::new("c");
///
/// problem += vec!(a,b,c).sum().equal(10.0);
/// ```
///
impl<T> SummableExp for Vec<T> where T: Into<LpExpression> + Clone {
    fn sum(&self) -> LpExprArena {
       lp_sum(self)
    }
}

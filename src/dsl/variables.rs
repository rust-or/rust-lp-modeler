/// # Module variables
///
use self::LpExpression::*;
use util::is_zero;

use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};

use std::convert::Into;
use std::borrow::Borrow;

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

/// ADT for Linear Programming Expression
#[derive(Debug, Clone, PartialEq)]
pub enum LpAtomicExpr {
    ConsInt(LpInteger),
    ConsBin(LpBinary),
    ConsCont(LpContinuous),
    LitVal(f32),
    EmptyExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LpExprOp {
    Multiply,
    Add,
    Subtract
}

pub type LpExprArenaIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct LpCompExpr {
    operation: LpExprOp,
    left_index: LpExprArenaIndex,
    right_index: LpExprArenaIndex
}

#[derive(Debug, Clone, PartialEq)]
pub enum LpExpression {
    LpAtomicExpr,
    LpCompExpr(LpExprOp, LpExprArenaIndex, LpExprArenaIndex)
}


impl LpExpression {
    /// Fix the numeric operand in a multiplication in an expression
    /// c * 4 must be considered as 4 c in a linear formulation lp file
    pub fn normalize(self) -> LpExpression {
        if let LpCompExpr(LpExprOp::Multiply, e1, e2) = self {
            if let LpAtomicExpr::LitVal(_) = e1 {
                return self.clone();
            } else {
                if let LpAtomicExpr::LitVal(_) = e2 {
                    return LpExpression::LpCompExpr(LpExprOp::Multiply, e2, e1);
                } else {
                    return LpExpression::LpCompExpr(LpExprOp::Multiply, e1, e2);
                }
            }
        } else {
            self
        }
    }

    pub fn split_constant_and_expr(self) -> (f32, LpExpression) {
        match self {
            LpCompExpr(LpExprOp::Add, &e1, &e2) => {
                if let LpAtomicExpr::LitVal(c) = e2 {
                    (c, e1)
                } else {
                    (0.0, self.clone())
                }
            }
            LpCompExpr(LpExprOp::Subtract, &e1, &e2) => {
                if let LpAtomicExpr::LitVal(c) = e2 {
                    (-c, e1)
                } else {
                    (0.0, self.clone())
                }
            }
            _ => (0.0, self.clone()),
        }
    }
}

pub struct LpExprArena {
    root: LpExprArenaIndex,
    array: Vec<LpExpression>
}

impl LpExprArena {

    pub fn new() -> Self {
       LpExprArena {
           root: 0,
           array: Vec::new()
       }
    }

    pub fn set_root(&mut self, root_index: LpExprArenaIndex) {
        self.root = root_index;
    }

    pub fn add_lp_expr(&mut self, lp_expr: LpExpression) -> LpExprArenaIndex {
        let index = self.len();
        self.array.push(lp_expr);
        return index
    }

    pub fn change_lp_expr(&mut self, index: LpExprArenaIndex, lp_expr: LpExpression) {
       self.array[index] = lp_expr;
    }

    pub fn index_expr_ref(&self, index: LpExprArenaIndex) -> &LpExpression {
        match self.array.get(index) {
            Some(expr) => expr,
            None => panic!("Requested index out of bound of LpExprArena vector. This should not happen.")
        }
    }

    pub fn index_expr(&self, index: LpExprArenaIndex) -> LpExpression {
        match self.array.get(index) {
            Some(&expr) => expr,
            None => panic!("Requested index out of bound of LpExprArena vector. This should not happen.")
        }
    }

    pub fn get_root_expr(&self) -> LpExpression {
        index_expr_ref(self, self.root).clone()
    }

    pub fn simplify(mut self) -> self {
        // keep clone of the starting expression to compare once recursive iteration finishes
        let mut starting_root = self.get_root_expr();
        let mut first_round = true;

        let mut lp_expr_stack: Vec<LpExprArenaIndex> = Vec::new();

        while first_round
            // check whether the root has changed -- if yes, do another round
            || ( starting_root != self.get_root_expr()) {
            starting_root = self.get_root_expr();
            first_round = false;
            lp_expr_stack.push(self.root.unwrap());
            while let Some(handled_expr_index) = lp_expr_stack.pop() {
                    match self.index_expr(handled_expr_index) {
                        LpCompExpr(LpExprOp::Multiply, left_index, right_index) => {
                            match (self.index_expr_ref(left_index), self.index_expr_ref(right_index)) {
                                // Trivial rule: 0 * x = 0
                                (_, LpAtomicExpr::LitVal(0.0))
                                | (LpAtomicExpr::LitVal(0.0), _) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpAtomicExpr::LitVal(0.0)
                                    )
                                },

                                // Simplify two literals
                                (LpAtomicExpr::LitVal(c1), LpAtomicExpr::LitVal(c2)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1 * c2)
                                    )
                                },

                                // DISTRIBUTIVITY
                                // i*(a+b) = i*a+i*b
                                (LpExpression(i), &LpCompExpr(LpExprOp::Add, a, b))
                                // (a+b)*i = i*a+i*b
                                | (&LpCompExpr(LpExprOp::Add, a, b), &i) => {
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiply, i, a)
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiply, i, b)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, left_index, right_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },
                                // i*(a-b) = i*a-i*b
                                (&i, &LpCompExpr(LpExprOp::Subtract, a_index, b_index))
                                // (a-b)*i = i*a-i*b
                                | (&LpCompExpr(LpExprOp::Subtract, a_index, b_index), &i) => {
                                    let i_new_index = self.add_lp_expr(i);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiply, i_new_index, a_index)
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiply, i_new_index, b_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract,left_index, right_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // COMMUTATIVITY WITH CONSTANTS
                                // c1*Multiply
                                (&LpAtomicExpr::LitVal(c1), &LpCompExpr(LpExprOp::Multiply, a_index, b_index)) => {
                                    match (self.index_expr_ref(a_index), self.index_expr_ref(b_index)) {
                                        // c1*(c2*x) = (c1*c2)*x
                                        (&LpAtomicExpr::LitVal(c2), &x)
                                        // c1*(expr*c2) = (c1*c2)*expr
                                        | (LpExpression(x), &LpAtomicExpr::LitVal(c2)) => {
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LpAtomicExpr::LitVal(c1 * c2),
                                            );
                                            self.change_lp_expr(right_index, x);
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        // c1*(a*b) = (c1*a)*b
                                        (&a, &b) => {
                                            self.change_lp_expr(
                                                left_index,
                                                LpExpression::LpCompExpr(
                                                    LpExprOp::Multiply,
                                                    LpExpression::LpAtomicExpr::LitVal(c1),
                                                    a_index
                                                )
                                            );
                                            self.change_lp_expr(
                                                right_index,
                                                b
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        }
                                    }
                                },
                                //TODO: (Pointless?)
                                // expr1*(c*expr) = c*(expr1*expr2)

                                // COMMUTATIVITY
                                // x*(a*b) = (x*a)*b
                                (&x, &LpCompExpr(LpExprOp::Multiply, a_index, b_index)) => {
                                    let new_x_index = self.add_lp_expr(x);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Multiply, new_x_index, a_index),
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        self.index_expr(b_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // Place literal first for *
                                (&a, &LpAtomicExpr::LitVal(c1)) => {
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1)
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        a
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                (_, _) => {
                                    lp_expr_stack.push(left_index);
                                    lp_expr_stack.push(right_index);
                                },
                            }
                        },
                        LpCompExpr(LpExprOp::Add, left_expr_index, right_expr_index) => {
                            match (self.index_expr_ref(left_expr_index), self.index_expr_ref(right_expr_index)) {
                                // Trivial rule: 0 + x = x
                                (&LpAtomicExpr::LitVal(0.0), &a)
                                // Trivial rule: x + 0 = x
                                | (&a, &LpAtomicExpr::LitVal(0.0)) => {
                                    self.change_lp_expr(handled_expr_index, a);
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Simplify two literals
                                (&LpAtomicExpr::LitVal(c1), &LpAtomicExpr::LitVal(c2)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1 + c2)
                                    );
                                },

                                // ASSOCIATIVITY
                                // a + (b+c) = (a+b)+c
                                (&a, &LpCompExpr(LpExprOp::Add, b_index, c_index)) => {
                                    let new_a_index = self.add_lp_expr(a);
                                    self.change_lp_expr(
                                        left_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, new_a_index, b_index),
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        self.index_expr(c_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // a + (b-c) = (a+b) - c
                                (&a, &LpCompExpr(LpExprOp::Subtract, b_index, c_index)) => {
                                    let new_a_index = self.add_lp_expr(a);
                                    self.change_lp_expr(
                                        left_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, new_a_index, b_index),
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        self.index_expr(c_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract, left_expr_index, right_expr_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // Place literal at the end
                                (&LpAtomicExpr::LitVal(c), &x) => {
                                    self.change_lp_expr(
                                        left_expr_index,
                                        x
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        LpExpression::LpAtomicExpr::LitVal(c)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                    lp_expr_stack.push(left_expr_index);
                                },

                                // Accumulate consts +/-
                                // (a+c1)+c2 = a+(c1+c2)
                                (&LpCompExpr(LpExprOp::Add, a_index, b_index), &LpAtomicExpr::LitVal(c2)) => {
                                    match self.index_expr_ref(b_index) {
                                        &LpAtomicExpr::LitVal(c1) => {
                                            self.change_lp_expr(
                                                left_expr_index,
                                                self.index_expr(a_index)
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LpAtomicExpr::LitVal(c1 + c2)
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
                                (&LpCompExpr(LpExprOp::Subtract, a_index, b_index), &LpAtomicExpr::LitVal(c2)) => {
                                    match (self.index_expr_ref(a_index), self.index_expr_ref(b_index)) {
                                        (&a, &LpAtomicExpr::LitVal(c1)) => {
                                            self.change_lp_expr(
                                                left_expr_index,
                                                a
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LpAtomicExpr::LitVal(c2 - c1)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        }
                                        (&LpAtomicExpr::LitVal(c1), &b) => {
                                            self.change_lp_expr(
                                                left_expr_index,
                                                -b
//                                                LpExpression::LpCompExpr(LpExprOp::Subtract, LpExpression::LpAtomicExpr::LitVal(0.0), b),
                                            );
                                            self.change_lp_expr(
                                                right_expr_index,
                                                LpExpression::LpAtomicExpr::LitVal(c1 + c2)
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
                                (&LpCompExpr(LpExprOp::Add, a, &LpAtomicExpr::LitVal(c1)), &b) => {
                                    let new_b_index = self.add_lp_expr(b);
                                    self.change_lp_expr(
                                        left_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, a, new_b_index)
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },
                                // (a-c1)+b = (a+b)-c1
                                (&LpCompExpr(LpExprOp::Subtract, a, &LpAtomicExpr::LitVal(c1)), &b) => {
                                    let new_b_index = self.add_lp_expr(b);
                                    self.change_lp_expr(
                                        left_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, a, new_b_index)
                                    );
                                    self.change_lp_expr(
                                        right_expr_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1),
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract, left_expr_index, right_expr_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
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
                        &LpCompExpr(LpExprOp::Subtract, left_index, right_index) => {
                            match (self.index_expr_ref(left_index), self.index_expr_ref(right_index)) {
                                // Trivial rule: x - 0 = x
                                (&a, &LpAtomicExpr::LitVal(0.0)) => {
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        a
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // a - (b + c) = (a-b)-c
                                (&a, &LpCompExpr(LpExprOp::Add, b_index, c_index)) => {
                                    let a_new_index = self.add_lp_expr(a);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract, a_new_index, b_index)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract, left_index, c_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // a - (b - c) = (a-b)+c
                                (&a, &LpCompExpr(LpExprOp::Subtract, b_index, c_index)) => {
                                    let a_new_index = self.add_lp_expr(a);
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract, a_new_index, b_index),

                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, left_index, c_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Place literal at the end
                                // c1 - b = -b + c1
                                (&LpAtomicExpr::LitVal(c1), &b) => {
                                    let b_new_index = self.add_lp_expr(-b);
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, b_new_index, left_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                },

                                // Accumulate consts +/-
                                // (a-c1)-c2 = a-(c1+c2)
                                // (c1-b)-c2 = -b+(c1-c2)
                                (&LpCompExpr(LpExprOp::Subtract, a, b), &LpAtomicExpr::LitVal(c2)) => {
                                    match (self.index_expr_ref(a), self.index_expr_ref(b)) {
                                        (&a, &LpAtomicExpr::LitVal(c1)) => {
                                            self.change_lp_expr(
                                                left_index,
                                                a
                                            );
                                            self.change_lp_expr(
                                                right_index,
                                                LpExpression::LpAtomicExpr::LitVal(c1 + c2)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        (&LpAtomicExpr::LitVal(c1), &b) => {
                                            self.change_lp_expr(
                                                left_index,
                                                -b
                                            );
                                            self.change_lp_expr(
                                                right_index,
                                                LpExpression::LpAtomicExpr::LitVal(c1 - c2)
                                            );
                                            self.change_lp_expr(
                                                handled_expr_index,
                                                LpExpression::LpCompExpr(LpExprOp::Add, left_index, right_index)
                                            );
                                            lp_expr_stack.push(handled_expr_index);
                                        },
                                        _ => {
                                            lp_expr_stack.push(left_index);
                                        },
                                    }
                                },

                                // (a+c1)-c2 = a+(c1-c2)
                                (&LpCompExpr(LpExprOp::Add, a_index, &LpAtomicExpr::LitVal(c1)), &LpAtomicExpr::LitVal(c2)) => {
                                    self.change_lp_expr(
                                        right_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1 - c2)
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, a_index, right_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                // Extract the const:
                                // (a+c1)-b = (a-b)+c1
                                (&LpCompExpr(LpExprOp::Add, a_index, &LpAtomicExpr::LitVal(c1)), &b) => {
                                    let c1_new_index = self.add_lp_expr(LpExpression::LpAtomicExpr::LitVal(c1));
                                    self.change_lp_expr(
                                        left_index,
                                        LpExpression::LpCompExpr(LpExprOp::Subtract, a_index, right_index),
                                    );
                                    self.change_lp_expr(
                                        handled_expr_index,
                                        LpExpression::LpCompExpr(LpExprOp::Add, left_index, c1_new_index)
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }
                                // (a-c1)-b = (a-b)-c1
                                (&LpCompExpr(LpExprOp::Subtract, a_index, &LpAtomicExpr::LitVal(c1)), &b) => {
                                    self.change_lp_expr(
                                        left_index,
                                        self.index_expr(a_index) - b
                                    );
                                    self.change_lp_expr(
                                        right_index,
                                        LpExpression::LpAtomicExpr::LitVal(c1),
                                    );
                                    lp_expr_stack.push(handled_expr_index);
                                }

                                _ => {
                                    lp_expr_stack.push(left_index);
                                    lp_expr_stack.push(right_index);
                                },
                            }
                        }
                        &LpAtomicExpr::ConsBin(_)
                        | &LpAtomicExpr::ConsInt(_)
                        | &LpAtomicExpr::ConsCont(_)
                        | &LpAtomicExpr::LpAtomicExpr::LitVal(_)
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
                LpAtomicExpr::ConsInt(v) => quote!(LpExpression::LpAtomicExpr::ConsInt(#v)),
                LpAtomicExpr::ConsBin(v) => quote!(LpExpression::LpAtomicExpr::ConsBin(#v)),
                LpAtomicExpr::ConsCont(v) => quote!(LpExpression::LpAtomicExpr::ConsCont(#v)),
                LpCompExpr(LpExprOp::Multiply, lhs, rhs) => quote!(LpExpression::LpCompExpr(LpExprOp::Multiply, #lhs, #rhs)),
                LpCompExpr(LpExprOp::Add, lhs, rhs) => quote!(LpExpression::LpCompExpr(LpExprOp::Add, #lhs, #rhs)),
                LpCompExpr(LpExprOp::Subtract, lhs, rhs) => quote!(LpExpression::LpCompExpr(LpExprOp::Subtract, #lhs, #rhs)),
                LpAtomicExpr::LitVal(v) =>  quote!(LpExpression::LpAtomicExpr(LitVal(#v))),
                LpAtomicExpr::EmptyExpr => quote!(LpExpression::LpAtomicExpr::EmptyExpr),
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
        let (constant, lhs_expr) = (lhs - rhs).simplify().split_constant_and_expr();
        LpConstraint(lhs_expr, op.clone(), LpExpression::LpAtomicExpr::LitVal(-constant))
    }
}

impl ToTokens for LpConstraint {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let lhs = &self.0;
        let constraint = &self.1;
        let rhs = &self.2;
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
            arena.add_lp_expr(first.clone());
            expr[1..].iter().map(|&a| {
                    let index = arena.add_lp_expr(a);
                    arena.add_lp_expr(
                        LpExpression(
                            LpCompExpr {
                                operation: LpExprOp::Add,
                                left_index: (index - 1),
                                right_index: index
                            }
                        )
                    )
                }
            );
            arena.set_root(arena.array.len() - 1);
            arena
        }
        None => {
            panic!("vector should have at least one element");
        }
    }
}

pub fn sum<'a, T: 'a,U: 'a>(expr: &'a Vec<T>, f: impl Fn(&'a T) -> U) -> LpExprArena where U: Into<LpExpression> + Clone {
    return lp_sum(&expr.iter().map(|t| f(t.into())).collect());
}

pub trait SummableExp {
    fn sum(&self) -> LpExpression;
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

/// # Module variables
///
use self::LpExprNode::*;
use self::LpExprOp::*;

use proc_macro2::{TokenStream};
use quote::{quote, ToTokens};

use std::convert::Into;
use std::collections::HashMap;
use std::fmt::Write;

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
    pub(crate) name: String,
    pub(crate) lower_bound: Option<f32>,
    pub(crate) upper_bound: Option<f32>,
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
pub(crate) enum LpExprOp {
    Multiplication,
    Addition,
    Subtraction
}

impl ToTokens for LpExprOp {
    fn to_tokens(&self, stream: &mut TokenStream) {
        stream.extend(
            match self {
                LpExprOp::Multiplication => quote!(LpExprOp::Multiplication),
                LpExprOp::Addition => quote!(LpExprOp::Addition),
                LpExprOp::Subtraction => quote!(LpExprOp::Subtraction),
            }
        );
    }
}

pub(crate) type LpExprArenaIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LpCompExpr {
    operation: LpExprOp,
    left_index: LpExprArenaIndex,
    right_index: LpExprArenaIndex
}

/// ADT for Linear Programming Expression
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LpExprNode {
    ConsInt(LpInteger),
    ConsBin(LpBinary),
    ConsCont(LpContinuous),
    LitVal(f32),
    EmptyExpr,
    LpCompExpr(LpExprOp, LpExprArenaIndex, LpExprArenaIndex)
}

impl ToTokens for LpExprNode {
    fn to_tokens(&self, stream: &mut TokenStream) {
        stream.extend(
            match self {
                LpExprNode::ConsInt(v) => quote!(LpExprNode::ConsInt(#v)),
                LpExprNode::ConsBin(v) => quote!(LpExprNode::ConsBin(#v)),
                LpExprNode::ConsCont(v) => quote!(LpExprNode::ConsCont(#v)),
                LpExprNode::LpCompExpr(op, lhs, rhs) => quote!(LpExprNode::LpCompExpr(#op, #lhs, #rhs)),
                LpExprNode::LitVal(v) =>  quote!(LpExprNode::LitVal(#v)),
                LpExprNode::EmptyExpr => quote!(LpExprNode::EmptyExpr),
            }
        );
    }
}

// Macro implementing Into<LpExprNode> for types
macro_rules! cons_into_expr {
    ($type_from:ty, $wrapper: ident) => {
        impl From<$type_from> for LpExprNode {
            fn from(from: $type_from) -> Self {
                $wrapper(from)
            }
        }
        impl<'a> From<&'a $type_from> for LpExprNode {
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
        impl From<$type_from> for LpExprNode {
            fn from(from: $type_from) -> Self {
                    LitVal(from as f32)
            }
        }
        impl<'a> From<&'a $type_from> for LpExprNode {
            fn from(from: &'a $type_from) -> Self {
                    LitVal((*from).clone() as f32)
            }
        }
    };
}
lit_into_expr!(f32);
lit_into_expr!(i32);

#[derive(Debug, Clone, PartialEq)]
pub struct LpExpression {
    root: LpExprArenaIndex,
    arena: Vec<LpExprNode>
}

impl ToTokens for LpExpression {
    fn to_tokens(&self, stream: &mut TokenStream) {
        let root = self.get_root_index();
        let arena = self.arena.clone();
        stream.extend( quote! {
            LpExpression {
                root: #root,
                arena: #( struct #arena;),*
            }
        });
    }
}

// Macro implementing Into<LpExpression> for types
macro_rules! cons_into_expr_arena {
    ($type_from:ty, $wrapper: ident) => {
        impl From<$type_from> for LpExpression {
            fn from(from: $type_from) -> Self {
                LpExpression {
                    root: 0,
                    arena: vec![$wrapper(from); 1]
                }
            }
        }
        impl<'a> From<&'a $type_from> for LpExpression {
            fn from(from: &'a $type_from) -> Self {
                LpExpression {
                    root: 0,
                    arena: vec![$wrapper((*from).clone()); 1]
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
        impl From<$type_from> for LpExpression {
            fn from(from: $type_from) -> Self {
                LpExpression {
                    root: 0,
                    arena: vec![LitVal(from as f32); 1]
                }
            }
        }
        impl<'a> From<&'a $type_from> for LpExpression {
            fn from(from: &'a $type_from) -> Self {
                LpExpression {
                    root: 0,
                    arena: vec![LitVal((*from).clone() as f32); 1]
                }
            }
        }
    };
}
lit_into_expr_arena!(f32);
lit_into_expr_arena!(i32);

impl From<LpExprNode> for LpExpression {
    fn from(expr: LpExprNode) -> Self {
        LpExpression {
            root: 0,
            arena: vec![expr; 1]
        }
    }
}

impl From<&LpExprNode> for LpExpression {
    fn from(expr: &LpExprNode) -> Self {
        LpExpression {
            root: 0,
            arena: vec![expr.clone(); 1]
        }
    }
}

impl LpExpression {
    fn new() -> Self {
       LpExpression {
           root: 0,
           arena: Vec::new()
       }
    }

    #[cfg(test)]
    fn build(root: LpExprArenaIndex, arena: Vec<LpExprNode>) -> Self {
        LpExpression {
            root: root,
            arena: arena
        }
    }

    pub(crate) fn get_root_index(&self) -> LpExprArenaIndex {
        self.root
    }

    fn set_root_to_index(&mut self, root_index: LpExprArenaIndex) {
        self.root = root_index;
    }

    fn push_as_expr<T>(&mut self, lp_expr: &T) -> LpExprArenaIndex where T: Into<LpExprNode> + Clone {
        let index = self.arena.len();
        self.arena.push(lp_expr.clone().into());
        return index
    }

    fn clone_expr_at_and_push(&mut self, index: LpExprArenaIndex) -> LpExprArenaIndex {
        let new_index = self.arena.len();
        self.arena.push(self.expr_clone_at(index));
        return new_index
    }

    fn overwrite_expr_at(&mut self, index: LpExprArenaIndex, lp_expr: LpExprNode) {
       self.arena[index] = lp_expr;
    }

    pub(crate) fn expr_ref_at(&self, index: LpExprArenaIndex) -> &LpExprNode {
        match self.arena.get(index) {
            Some(expr) => expr,
            None => panic!("Requested index out of bound of LpExpression vector. This should not happen.")
        }
    }

    fn expr_clone_at(&self, index: LpExprArenaIndex) -> LpExprNode {
        match self.arena.get(index) {
            Some(expr) => expr.clone(),
            None => panic!("Requested index out of bound of LpExpression vector. This should not happen.")
        }
    }

    pub(crate) fn get_root_expr_ref(&self) -> &LpExprNode {
        self.expr_ref_at(self.root)
    }

    pub(crate) fn split_off_constant(&mut self) -> f32 {
        match self.expr_clone_at(self.root) {
            LitVal(c) => {
                self.clone_from(&LpExpression::new());
                c
            },
            LpCompExpr(Addition, e1, e2) => {
                if let LitVal(c) = self.expr_clone_at(e2) {
                    self.set_root_to_index(e1);
                    c
                } else {
                    0.0
                }
            },
            LpCompExpr(Subtraction, e1, e2) => {
                if let LitVal(c) = self.expr_clone_at(e2) {
                    self.set_root_to_index(e1);
                    -c
                } else {
                    0.0
                }
            },
            _ => 0.0
        }
    }

    pub(crate) fn merge_cloned_arenas(&self, right_lp_expr_arena: &LpExpression, operation: LpExprOp) -> Self {
        let mut new_arena = self.clone();
        let index_at_insertion = new_arena.push_arena_at_root(right_lp_expr_arena);
        let new_root = new_arena.push_as_expr(
            &LpCompExpr(operation, new_arena.get_root_index(), index_at_insertion)
        );
        new_arena.set_root_to_index(new_root);
        new_arena
    }

    fn push_arena_at_root(&mut self, right_lp_expr_arena: &LpExpression) -> LpExprArenaIndex {
        let right_root_expr_ref = right_lp_expr_arena.get_root_expr_ref();
        let new_index_right_root = self.push_as_expr(right_root_expr_ref);
        let mut move_stack: Vec<LpExprArenaIndex> = Vec::new();
        move_stack.push(new_index_right_root);
        while let Some(new_parent_index) = move_stack.pop() {
            let lp_expr_arena = self.expr_clone_at(new_parent_index);
            if let LpCompExpr(operation, right_arena_left_index, right_arena_right_index) = lp_expr_arena {
                    let new_left_index = self.push_as_expr(right_lp_expr_arena.expr_ref_at(right_arena_left_index));
                    let new_right_index = self.push_as_expr(right_lp_expr_arena.expr_ref_at(right_arena_right_index));
                    self.overwrite_expr_at(
                        new_parent_index,
                        LpCompExpr(
                            operation.clone(),
                            new_left_index,
                            new_right_index
                        )
                    );
                    move_stack.push(new_left_index);
                    move_stack.push(new_right_index);
            }
        }
        new_index_right_root
    }

    fn clone_subtree_at_index_and_push(&mut self, index: LpExprArenaIndex) -> LpExpression {
        let mut clone_stack: Vec<LpExprNode> = vec![self.expr_clone_at(index)];
        let mut cloned_subtree = LpExpression::new();
        let mut new_left_index: LpExprArenaIndex;
        let mut new_right_index_stack: Vec<LpExprArenaIndex> = Vec::new();
        let mut left_stack: Vec<bool> = vec![false];
        while let (Some(expr), Some(mut left)) = (clone_stack.pop(), left_stack.pop()) {
            if let LpCompExpr(op, left_index, right_index) = expr {
                clone_stack.push(LpCompExpr(op, left_index, right_index));
                left_stack.push(left);
                clone_stack.push(self.expr_clone_at(left_index));
                left_stack.push(true);
                clone_stack.push(self.expr_clone_at(right_index));
                left_stack.push(false);
            } else {
                if left {
                    new_left_index = cloned_subtree.push_as_expr(&expr);
                    while left {
                        if let (Some(LpCompExpr(op, _, _)), Some(local_left)) = (clone_stack.pop(), left_stack.pop()) {
                            if let Some(new_right_index) = new_right_index_stack.pop() {
                                left = local_left;
                                if left {
                                    new_left_index = cloned_subtree.push_as_expr(
                                        &LpCompExpr(op, new_left_index, new_right_index)
                                    );
                                } else {
                                    new_right_index_stack.push(
                                        cloned_subtree.push_as_expr(
                                            &LpCompExpr(op, new_left_index, new_right_index)
                                        )
                                    );
                                }
                            } else {
                                panic!("Found no remaining right index to match the left.")
                            }
                        } else {
                            panic!("Found no parent node to match two new indices I have.");
                        }
                    }
                } else {
                    new_right_index_stack.push( cloned_subtree.push_as_expr(&expr) );
                }
            }
        }

        if let Some(root_index)  = new_right_index_stack.pop() {
            cloned_subtree.set_root_to_index(root_index);
            cloned_subtree
        } else {
            panic!("Got an empty new_right_index_stack. This is a bug.");
        }
    }

    pub(crate) fn show(&self, e: &LpExprArenaIndex, with_parenthesis: bool) -> String {
        #[derive(Clone, Copy)]
        enum Part {
            None,
            Char(char),
            Str(&'static str),
            Elem(LpExprArenaIndex),
        }
        let left = if with_parenthesis { Part::Char('(') } else { Part::None };
        let right = if with_parenthesis { Part::Char(')') } else { Part::None };
        let op_mult = if with_parenthesis { Part::Str(" * ") } else { Part::Char(' ') };

        let mut remaining: Vec<Part> = vec![Part::Elem(*e)];
        let mut result = String::new();
        while let Some(e) = remaining.pop() {
            match e {
                Part::None => {}
                Part::Char(c) => { result.push(c) }
                Part::Str(s) => { result.push_str(s) }
                Part::Elem(e) => {
                    match self.expr_ref_at(e) {
                        LitVal(n) => write!(result, "{}", n).unwrap(),
                        LpCompExpr(Addition, e1, e2) => {
                            remaining.push(right);
                            remaining.push(Part::Elem(*e2));
                            remaining.push(Part::Str(" + "));
                            remaining.push(Part::Elem(*e1));
                            remaining.push(left);
                        }
                        LpCompExpr(Subtraction, e1, e2) => {
                            remaining.push(right);
                            remaining.push(Part::Elem(*e2));
                            remaining.push(Part::Str(" - "));
                            remaining.push(Part::Elem(*e1));
                            remaining.push(left);
                        }
                        LpCompExpr(Multiplication, e1, e2) => {
                            match self.expr_ref_at(*e1) {
                                &LitVal(c) if c == 1.0 => {
                                    remaining.push(right);
                                    remaining.push(Part::Elem(*e2));
                                    remaining.push(Part::Char(' '));
                                    remaining.push(left);
                                },
                                &LitVal(c) if c == -1.0 => {
                                    remaining.push(right);
                                    remaining.push(Part::Elem(*e2));
                                    remaining.push(Part::Str("-"));
                                    remaining.push(left);
                                }
                                _ => {
                                    remaining.push(right);
                                    remaining.push(Part::Elem(*e2));
                                    remaining.push(op_mult);
                                    remaining.push(Part::Elem(*e1));
                                    remaining.push(left);
                                }
                            }
                        }
                        ConsBin(LpBinary { name: ref n, .. }) => result += n,
                        ConsInt(LpInteger { name: ref n, .. }) => result += n,
                        ConsCont(LpContinuous { name: ref n, .. }) => result += n,
                        EmptyExpr => { result += "EmptyExpr!!!" }
                    }
                }
            }
        }
        result
    }

    pub(crate) fn simplify(&mut self) -> &mut Self {
        // keep clone of the root expression at the start of each round to compare
        // once one round of recursive iteration finishes
        let mut show_at_start = self.show(&self.get_root_index(), true);
        let mut first_round = true;

        let mut lp_expr_stack: Vec<LpExprArenaIndex> = Vec::new();

        while first_round
            // check whether the root has changed -- if yes, do another round
            || ( show_at_start != self.show(&self.get_root_index(), true)) {
            show_at_start = self.show(&self.get_root_index(), true);
            first_round = false;
            lp_expr_stack.push(self.get_root_index());
            while let Some(handled_expr_index) = lp_expr_stack.pop() {
                match self.expr_clone_at(handled_expr_index) {
                    LpCompExpr(Multiplication, left_index, right_index) => {
                        match (self.expr_clone_at(left_index), self.expr_clone_at(right_index)) {
                            // Trivial rule: 0 * x = 0
                            (_, LitVal(c))
                            | (LitVal(c), _) if c == 0.0 => {
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LitVal(0.0)
                                )
                            },

                            // Simplify two literals
                            (LitVal(c1), LitVal(c2)) => {
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LitVal(c1 * c2)
                                )
                            },

                            // DISTRIBUTIVITY
                            // i*(a+b) = i*a+i*b
                            (i, LpCompExpr(Addition, a_index, b_index)) => {
                                let i_new_index: LpExprArenaIndex;
                                if let LpCompExpr(_, _, _) = i {
                                    let new_subtree = self.clone_subtree_at_index_and_push(left_index);
                                    i_new_index = self.push_arena_at_root(&new_subtree);
                                } else {
                                    // Cons or LitVal type
                                    i_new_index = self.clone_expr_at_and_push(left_index);
                                }
                                let new_left_index = self.push_as_expr(
                                    &LpCompExpr(Multiplication, left_index, a_index)
                                );
                                self.overwrite_expr_at(
                                    right_index,
                                    LpCompExpr(Multiplication, i_new_index, b_index)
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Addition, new_left_index, right_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },
                            // (a+b)*i = i*a+i*b
                            (LpCompExpr(Addition, a_index, b_index), i) => {
                                let i_new_index: LpExprArenaIndex;
                                if let LpCompExpr(_, _, _) = i {
                                    let new_subtree = self.clone_subtree_at_index_and_push(right_index);
                                    i_new_index = self.push_arena_at_root(&new_subtree);
                                } else {
                                    // Cons or LitVal type
                                    i_new_index = self.clone_expr_at_and_push(right_index);
                                }
                                let new_right_index = self.push_as_expr(
                                    &LpCompExpr(Multiplication, right_index, b_index)
                                );
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Multiplication, i_new_index, a_index)
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Addition, left_index, new_right_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // (a-b)*i = i*a-i*b
                            (LpCompExpr(Subtraction, a_index, b_index), i) => {
                                let i_new_index: LpExprArenaIndex;
                                if let LpCompExpr(_, _, _) = i {
                                    let new_subtree = self.clone_subtree_at_index_and_push(right_index);
                                    i_new_index = self.push_arena_at_root(&new_subtree);
                                } else {
                                    // Cons or LitVal type
                                    i_new_index = self.clone_expr_at_and_push(right_index);
                                }
                                let new_right_index = self.push_as_expr(
                                    &LpCompExpr(Multiplication, right_index, b_index)
                                );
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Multiplication, i_new_index, a_index)
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Subtraction, left_index, new_right_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },
                            // i*(a-b) = i*a-i*b
                            (i, LpCompExpr(Subtraction, a_index, b_index)) => {
                                let i_new_index: LpExprArenaIndex;
                                if let LpCompExpr(_, _, _) = i {
                                    let new_subtree = self.clone_subtree_at_index_and_push(left_index);
                                    i_new_index = self.push_arena_at_root(&new_subtree);
                                } else {
                                    // Cons or LitVal type
                                    i_new_index = self.clone_expr_at_and_push(left_index);
                                }
                                let new_left_index = self.push_as_expr(
                                    &LpCompExpr(Multiplication, left_index, a_index)
                                );
                                self.overwrite_expr_at(
                                    right_index,
                                    LpCompExpr(Multiplication, i_new_index, b_index)
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Subtraction, new_left_index, right_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },


                            // COMMUTATIVITY WITH CONSTANTS
                            // c1*(a*b)
                            (LitVal(c1), LpCompExpr(Multiplication, a_index, b_index)) => {
                                match (self.expr_clone_at(a_index), self.expr_clone_at(b_index)) {
                                    // c1*(c2*b) = (c1*c2)*b
                                    (LitVal(c2), _) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LitVal(c1 * c2),
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(
                                                Multiplication,
                                                left_index,
                                                b_index
                                            )
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    // c1*(a*c2) = (c1*c2)*a
                                    (_, LitVal(c2)) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LitVal(c1 * c2),
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(
                                                Multiplication,
                                                left_index,
                                                a_index
                                            )
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    // c1*(a*b) = (c1*a)*b
                                    (_, _) => {
                                        let lit_new_index = self.push_as_expr(&LitVal(c1));
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(
                                                Multiplication,
                                                lit_new_index,
                                                a_index
                                            )
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(
                                                Multiplication,
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
                            (_, LpCompExpr(Multiplication, a_index, b_index)) => {
                                let left_new_index = self.clone_expr_at_and_push(left_index);
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Multiplication, left_new_index, a_index),
                                );
                                self.overwrite_expr_at(
                                    right_index,
                                    self.expr_clone_at(b_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // Place literal first for *
                            (_, LitVal(_)) => {
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(
                                        Multiplication,
                                        right_index,
                                        left_index
                                    )
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // When literal is first and right side is not LpCompExpr (LpCompExpr case handled above), stop
                            (LitVal(_c1), _) => {
                            },
                            // recurse deeper and come back for any complex expressions not handled above
                            (LpCompExpr(_, _, _), _) => {
                                lp_expr_stack.push(left_index);
                            },
                            (_, _) => {}
                        }
                    },
                    LpCompExpr(Addition, left_index, right_index) => {
                        match (self.expr_clone_at(left_index), self.expr_clone_at(right_index)) {
                            // Trivial rule: 0 + x = x
                            (LitVal(c), a)
                            // Trivial rule: x + 0 = x
                            | (a, LitVal(c)) if c == 0.0 => {
                                self.overwrite_expr_at(handled_expr_index, a.clone());
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // Simplify two literals
                            (LitVal(c1), LitVal(c2)) => {
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LitVal(c1 + c2)
                                );
                            },

                            // Place literal at the end
                            (LitVal(_c), _x) => {
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Addition, right_index, left_index)
                                );
                                lp_expr_stack.push(right_index);
                            },

                            // ASSOCIATIVITY
                            // a + (b+c) = (a+b)+c
                            (a, LpCompExpr(Addition, b_index, c_index)) => {
                                let new_a_index = self.push_as_expr(&a.clone());
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Addition, new_a_index, b_index),
                                );
                                self.overwrite_expr_at(
                                    right_index,
                                    self.expr_clone_at(c_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // a + (b-c) = (a+b) - c
                            (a, LpCompExpr(Subtraction, b_index, c_index)) => {
                                let new_a_index = self.push_as_expr(&a.clone());
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Addition, new_a_index, b_index),
                                );
                                self.overwrite_expr_at(
                                    right_index,
                                    self.expr_clone_at(c_index)
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Subtraction, left_index, right_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // Accumulate consts +/-
                            // (a+c1)+c2 = a+(c1+c2)
                            (LpCompExpr(Addition, a_index, b_index), LitVal(c2)) => {
                                match self.expr_clone_at(b_index) {
                                    LitVal(c1) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            self.expr_clone_at(a_index)
                                        );
                                        self.overwrite_expr_at(
                                            right_index,
                                            LitVal(c1 + c2)
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    _ => {
                                        lp_expr_stack.push(left_index);
                                    },
                                }
                            },
                            (LpCompExpr(Subtraction, a_index, b_index), LitVal(c2)) => {
                                match (self.expr_clone_at(a_index), self.expr_clone_at(b_index)) {
                                    // (a-c1)+c2 = a+(c2-c1)
                                    (_a, LitVal(c1)) => {
                                        self.overwrite_expr_at(
                                            right_index,
                                            LitVal(c2 - c1)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Addition, a_index, right_index)
                                        );
                                        lp_expr_stack.push(a_index);
                                    },
                                    // (c1-b)+c2 = -b+(c1+c2)
                                    (LitVal(c1), _b) => {
                                        let lit_new_index = self.push_as_expr(&LitVal(-1.0));
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(
                                                Multiplication,
                                                lit_new_index,
                                                b_index
                                            )
                                        );
                                        self.overwrite_expr_at(
                                            right_index,
                                            LitVal(c1 + c2)
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    _ => {
                                        lp_expr_stack.push(left_index);
                                        // lp_expr_stack.push(&right_expr);
                                    },
                                }
                            },

                            // Extract the const
                            (LpCompExpr(Addition, a_index, b_index), _x) => {
                                match (self.expr_clone_at(a_index), self.expr_clone_at(b_index)) {
                                    // (a+c1)+x = (a+x)+c1
                                    (_, LitVal(_c1)) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(Addition, a_index, right_index)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Addition, left_index, b_index)
                                        );
                                        lp_expr_stack.push(left_index);
                                    },
                                    //// (c1+b)+x = (x+b)+c1
                                    //(LitVal(_c1), _) => {
                                    //    self.change_lp_expr(
                                    //        left_index,
                                    //        LpCompExpr(Addition, right_index, b_index)
                                    //    );
                                    //    self.change_lp_expr(
                                    //        handled_expr_index,
                                    //        LpCompExpr(Addition, left_index, a_index)
                                    //    );
                                    //    lp_expr_stack.push(left_index);
                                    //},
                                    _ => {
                                        lp_expr_stack.push(right_index);
                                        lp_expr_stack.push(left_index);
                                    }
                                }
                            },
                            (LpCompExpr(Subtraction, a_index, b_index), _x) => {
                                match (self.expr_clone_at(a_index), self.expr_clone_at(b_index)) {
                                    // (a-c1)+x = (a+x)-c1
                                    (_a, LitVal(_c1)) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(Addition, a_index, right_index)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Subtraction, left_index, b_index)
                                        );
                                        lp_expr_stack.push(left_index);
                                    },
                                    // (c1-b)+x = (x-b)+c1
                                    (LitVal(_c1), _b) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(Subtraction, right_index, b_index)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Addition, left_index, a_index)
                                        );
                                        lp_expr_stack.push(left_index);
                                    },
                                    (_a, _b) => {
                                        lp_expr_stack.push(left_index);
                                        lp_expr_stack.push(right_index);
                                    }
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

                            (a, b) => {
                                // a + a = 2 * a
                                if a == b {
                                    let new_lit_index = self.push_as_expr(&LitVal(2.0));
                                    self.overwrite_expr_at(
                                        handled_expr_index,
                                        LpCompExpr(
                                            Multiplication,
                                            new_lit_index,
                                            left_index
                                        )
                                    );
                                    lp_expr_stack.push(left_index);
                                } else {
                                    match (a, b) {
                                        (LpCompExpr(_, _, _), LpCompExpr(_, _, _)) => {
                                            lp_expr_stack.push(left_index);
                                            lp_expr_stack.push(right_index);
                                        },
                                        (LpCompExpr(_, _, _), _) => {
                                            lp_expr_stack.push(left_index);
                                        },
                                        (_, LpCompExpr(_, _, _)) => {
                                            lp_expr_stack.push(right_index);
                                        },
                                        (_, _) => {}
                                    }
                                }
                            }
                        }
                    },
                    LpCompExpr(Subtraction, left_index, right_index) => {
                        match (self.expr_clone_at(left_index), self.expr_clone_at(right_index)) {
                            // Trivial rule: x - 0 = x
                            (a, LitVal(c)) if c == 0.0 => {
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    a.clone()
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // a - (b + c) = (a-b)-c
                            (_, LpCompExpr(Addition, b_index, c_index)) => {
                                let a_new_index = self.clone_expr_at_and_push(left_index);
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Subtraction, a_new_index, b_index)
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Subtraction, left_index, c_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // a - (b - c) = (a-b)+c
                            (_, LpCompExpr(Subtraction, b_index, c_index)) => {
                                let a_new_index = self.clone_expr_at_and_push(left_index);
                                self.overwrite_expr_at(
                                    left_index,
                                    LpCompExpr(Subtraction, a_new_index, b_index),

                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Addition, left_index, c_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // Place literal at the end
                            // c1 - b = -b + c1
                            (LitVal(_), _) => {
                                let lit_new_index = self.push_as_expr(&LitVal(-1.0));
                                let new_index = self.push_as_expr(
                                    &LpCompExpr(
                                        Multiplication,
                                        lit_new_index,
                                        right_index
                                    )
                                );
                                self.overwrite_expr_at(
                                    handled_expr_index,
                                    LpCompExpr(Addition, new_index, left_index)
                                );
                                lp_expr_stack.push(handled_expr_index);
                            },

                            // Accumulate consts +/-
                            // (a-c1)-c2 = a-(c1+c2)
                            // (c1-b)-c2 = -b+(c1-c2)
                            (LpCompExpr(Subtraction, a_index, b_index), LitVal(c2)) => {
                                match (self.expr_clone_at(a_index), self.expr_clone_at(b_index)) {
                                    (a, LitVal(c1)) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            a.clone()
                                        );
                                        self.overwrite_expr_at(
                                            right_index,
                                            LitVal(c1 + c2)
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    (LitVal(c1), _) => {
                                        let lit_new_index = self.push_as_expr(&LitVal(-1.0));
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(
                                                Multiplication,
                                                lit_new_index,
                                                b_index
                                            )
                                        );
                                        self.overwrite_expr_at(
                                            right_index,
                                            LitVal(c1 - c2)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(
                                                Addition,
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
                            },

                            // (a+c1)-c2 = a+(c1-c2)
                            (LpCompExpr(Addition, a_index, c1_index), LitVal(c2)) => {
                                match self.expr_clone_at(c1_index) {
                                    LitVal(c1) => {
                                        self.overwrite_expr_at(
                                            right_index,
                                            LitVal(c1 - c2)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Addition, a_index, right_index)
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    _ => {
                                        lp_expr_stack.push(left_index);
                                    }
                                }

                            },

                            // Extract the const:
                            // (a+c1)-x = (a-x)+c1
                            (LpCompExpr(Addition, a_index, b_index), _x) => {
                                match self.expr_clone_at(b_index) {
                                    LitVal(_c1) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(Subtraction, a_index, right_index),
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Addition, left_index, b_index)
                                        );
                                        lp_expr_stack.push(handled_expr_index);
                                    },
                                    _ => {
                                        lp_expr_stack.push(left_index);
                                        lp_expr_stack.push(right_index);
                                    }
                                }
                            }
                            (LpCompExpr(Subtraction, a_index, b_index), _x) => {
                                match (self.expr_clone_at(a_index), self.expr_clone_at(b_index)) {
                                    // (a-c1)-x = (a-x)-c1
                                    (_a, LitVal(_c1)) => {
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(Subtraction, a_index, right_index)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Subtraction, left_index, b_index)
                                        );
                                        lp_expr_stack.push(left_index);
                                    },
                                    // (c1-b)-x = (-b-x)+c1
                                    (LitVal(_c1), _b) => {
                                        let minus_one_new_index = self.push_as_expr(&LitVal(-1.0));
                                        let minus_b_new_index = self.push_as_expr(
                                            &LpCompExpr(Multiplication, minus_one_new_index, b_index)
                                        );
                                        self.overwrite_expr_at(
                                            left_index,
                                            LpCompExpr(Subtraction, minus_b_new_index, right_index)
                                        );
                                        self.overwrite_expr_at(
                                            handled_expr_index,
                                            LpCompExpr(Addition, left_index, a_index)
                                        );
                                        lp_expr_stack.push(left_index);
                                    },
                                    _ => {
                                        lp_expr_stack.push(right_index);
                                        lp_expr_stack.push(left_index);
                                    }
                                }
                            },
                            (a, b) => {
                                // a - a = 0
                                if a == b {
                                    self.overwrite_expr_at(
                                        handled_expr_index,
                                        LitVal(0.0)
                                    );
                                } else {
                                    match (a, b) {
                                        // recurse deeper and come back for any complex expressions not handled above
                                        (LpCompExpr(_, _, _), LpCompExpr(_, _, _)) => {
                                            lp_expr_stack.push(left_index);
                                            lp_expr_stack.push(right_index);
                                        },
                                        (LpCompExpr(_, _, _), _) => {
                                            lp_expr_stack.push(left_index);
                                        },
                                        (_, LpCompExpr(_, _, _)) => {
                                            lp_expr_stack.push(right_index);
                                        },
                                        (_, _) => {}
                                    }
                                }
                            }
                        }
                    },
                    ConsBin(_)
                    | ConsInt(_)
                    | ConsCont(_)
                    | LitVal(_)
                    | LpExprNode::EmptyExpr => {}
                };
            }
        }
        self
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
pub struct LpConstraint(pub LpExpression, pub Constraint, pub LpExpression);

impl LpConstraint {
    pub(crate) fn generalize(&self) -> LpConstraint {
        // TODO: Optimize tailrec
        let &LpConstraint(ref lhs, ref op, ref rhs) = self;
        let mut new_lhs_expr = lhs.merge_cloned_arenas(rhs, Subtraction);
        let constant = new_lhs_expr.simplify().split_off_constant();
        let new_rhs_expr_arena: LpExpression = LitVal(-constant).into();
        LpConstraint(new_lhs_expr, (*op).clone(), new_rhs_expr_arena)
    }

    pub(crate) fn var(&self, expr_index: LpExprArenaIndex, constraint_index: usize, lst: &mut HashMap<String, (usize, LpExprArenaIndex)>) {
        match self.0.expr_ref_at(expr_index) {
            ConsBin(LpBinary { ref name, .. })
            | ConsInt(LpInteger { ref name, .. })
            | ConsCont(LpContinuous { ref name, .. }) => {
                lst.insert(name.clone(), (constraint_index, expr_index) );
            },
            LpCompExpr(Multiplication, _, e) => {
                self.var(*e, constraint_index, lst);
            },
            LpCompExpr(Addition, e1, e2)
            | LpCompExpr(Subtraction, e1, e2) => {
                self.var(*e1, constraint_index, lst);
                self.var(*e2, constraint_index, lst);
            }
            _ => (),
        }
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
pub fn lp_sum<T>(not_yet_lp_expr_arenas: &Vec<T>) -> LpExpression where T: Into<LpExpression> + Clone {
    match not_yet_lp_expr_arenas.first() {
        Some(first) => {
            let mut arena: LpExpression = first.clone().into();
            for a in not_yet_lp_expr_arenas[1..].iter() {
                arena = arena.merge_cloned_arenas(&a.clone().into(), Addition);
            }
            arena
        },
        None => {
            panic!("vector should have at least one element")
        }
    }
}

pub fn sum<'a, T: 'a,U: 'a>(expr: &'a Vec<T>, f: impl Fn(&'a T) -> U) -> LpExpression where U: Into<LpExpression> + Clone {
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
    fn sum(&self) -> LpExpression {
       lp_sum(self)
    }
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn expressions_creation() {
        let ref a = LpInteger::new("a").lower_bound(10.0).upper_bound(20.0);
        let ref b = LpInteger::new("b");

        assert_eq!(
            a + b,
            LpExpression::build(
                2,
                vec![LpExprNode::ConsInt(a.clone()), LpExprNode::ConsInt(b.clone()), LpExprNode::LpCompExpr(LpExprOp::Addition, 0, 1)]
            )
        );
    }

    #[test]
    fn simplifications() {
        let a = &LpInteger::new("a");

        let expr1 = a - 2f32;
        let expr2 = 1f32 - a;

        let c = (expr1.clone() + expr2.clone()).simplify().split_off_constant();
        assert_eq!(c, -1f32);

        let c = (expr2.clone() + expr1.clone()).simplify().split_off_constant();
        assert_eq!(c, -1f32);
    }

    #[test]
    fn simplify_deep_sum() {
        let count = 1000;
        let vars: Vec<LpExpression> = (0..count)
            .map(|i|
                &LpInteger::new(&format!("v{}", i)) * 2 + 1
            )
            .collect();
        let mut sum = lp_sum(&vars);
        assert_eq!(sum.simplify().split_off_constant(), count as f32);
    }

    #[test]
    fn test_quotations() {
        let a = LpInteger { name: "a".to_string(), lower_bound: None, upper_bound: None };
        let quoted_a = quote!(#a);
        let quoted_a_str = "LpInteger { name : \"a\" . to_string () , lower_bound : None , upper_bound : None }";
        assert_eq!(quoted_a.to_string(), quoted_a_str);

        let exp: LpExprNode = a.clone().into();
        let quoted_exp = quote!(#exp);
        let quoted_exp_str = "LpExprNode :: ConsInt (".to_owned() + quoted_a_str + ")";
        assert_eq!(quoted_exp.to_string(), quoted_exp_str);

        let full_exp_arena = LpExpression::build (0, vec![LpExprNode:: LpCompExpr (LpExprOp :: Multiplication, 1, 2), LpExprNode:: LpCompExpr (LpExprOp :: Subtraction, 3, 4 ), LpExprNode:: LpCompExpr (LpExprOp :: Addition, 5, 6), LpExprNode:: LitVal (1f32), LpExprNode:: EmptyExpr, LpExprNode:: ConsCont (LpContinuous { name : "x".to_string() , lower_bound : None , upper_bound : None }), LpExprNode:: ConsInt (LpInteger { name : "y".to_string() , lower_bound : None , upper_bound : None }) ] );


        let full_exp_quoted = quote!(#full_exp_arena);
        let full_exp_str = "LpExpression { root : 0usize , arena : struct LpExprNode :: LpCompExpr (LpExprOp :: Multiplication , 1usize , 2usize) ; , struct LpExprNode :: LpCompExpr (LpExprOp :: Subtraction , 3usize , 4usize) ; , struct LpExprNode :: LpCompExpr (LpExprOp :: Addition , 5usize , 6usize) ; , struct LpExprNode :: LitVal (1f32) ; , struct LpExprNode :: EmptyExpr ; , struct LpExprNode :: ConsCont (LpContinuous { name : \"x\" . to_string () , lower_bound : None , upper_bound : None }) ; , struct LpExprNode :: ConsInt (LpInteger { name : \"y\" . to_string () , lower_bound : None , upper_bound : None }) ; }";
        assert_eq!(full_exp_quoted.to_string(), full_exp_str);

        // a.equal(&b);
        let a_eq_b = LpConstraint(LpExpression::build(0, vec![LpExprNode:: LpCompExpr(LpExprOp :: Subtraction, 1, 2), LpExprNode::ConsInt (LpInteger { name : "a".to_string() , lower_bound : None , upper_bound : None }), LpExprNode::ConsInt (LpInteger { name : "b".to_string() , lower_bound : None , upper_bound : None }) ] ), Constraint::Equal, LitVal(0f32).into());

        let quoted_a_eq_b = quote!(#a_eq_b);
        let a_eq_b_str = "LpConstraint (LpExpression { root : 0usize , arena : struct LpExprNode :: LpCompExpr (LpExprOp :: Subtraction , 1usize , 2usize) ; , struct LpExprNode :: ConsInt (LpInteger { name : \"a\" . to_string () , lower_bound : None , upper_bound : None }) ; , struct LpExprNode :: ConsInt (LpInteger { name : \"b\" . to_string () , lower_bound : None , upper_bound : None }) ; } , Constraint :: Equal , LpExpression { root : 0usize , arena : struct LpExprNode :: LitVal (0f32) ; })";
        assert_eq!(quoted_a_eq_b.to_string(), a_eq_b_str);
    }
}
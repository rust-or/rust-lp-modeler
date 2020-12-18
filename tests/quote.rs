extern crate lp_modeler;
extern crate quote;
use quote::quote;

use lp_modeler::dsl::*;

#[test]
fn test_quotations() {
  use LpExprNode::*;
  let a = LpInteger { name : "a" . to_string () , lower_bound : None , upper_bound : None };
  let quoted_a = quote!(#a);
  let quoted_a_str = "LpInteger { name : \"a\" . to_string () , lower_bound : None , upper_bound : None }";
  assert_eq!(quoted_a.to_string(), quoted_a_str);

  let exp : LpExprNode = a.clone().into();
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

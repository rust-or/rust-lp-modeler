extern crate lp_modeler;

use lp_modeler::solvers::*;
use std::fs;

#[test]
fn cbc_optimal() {
    let _ = fs::copy("tests/solution_files/cbc_optimal.sol", "cbc_optimal.sol");
    let solver = CbcSolver::new().with_temp_solution_file("cbc_optimal.sol".to_string());
    let (status, mut variables) = solver.read_solution(&"cbc_optimal.sol".to_string()).unwrap();
    assert_eq!(status, Status::Optimal);
    assert_eq!(variables.remove("a"), Some(5f32));
    assert_eq!(variables.remove("b"), Some(6f32));
    assert_eq!(variables.remove("c"), Some(0f32));
}

#[test]
fn cbc_infeasible() {
    let _ = fs::copy(
        "tests/solution_files/cbc_infeasible.sol",
        "cbc_infeasible.sol",
    );
    let solver = CbcSolver::new().with_temp_solution_file("cbc_infeasible.sol".to_string());
    let (status, _) = solver.read_solution(&"cbc_infeasible.sol".to_string()).unwrap();
    assert_eq!(status, Status::Infeasible);
}

#[test]
// created from:
// minimize
//   obj: a + b
// subject to
//   c1: a + b <= 1
//   c2: a + b >= 2
// binaries
//   a b
// end
fn cbc_infeasible_alternative_format() {
    let _ = fs::copy(
        "tests/solution_files/cbc_infeasible_alternative_format.sol",
        "cbc_infeasible_alternative_format.sol",
    );
    let (status, mut variables) = CbcSolver::new()
        .with_temp_solution_file("cbc_infeasible_alternative_format.sol".to_string())
        .read_solution(&"cbc_infeasible_alternative_format.sol".to_string())
        .unwrap();
    assert_eq!(status, Status::Infeasible);
    assert_eq!(variables.remove("a"), Some(2f32));
    assert_eq!(variables.remove("b"), Some(0f32));
}

#[test]
fn cbc_unbounded() {
    let _ = fs::copy(
        "tests/solution_files/cbc_unbounded.sol",
        "cbc_unbounded.sol",
    );
    let solver = CbcSolver::new().with_temp_solution_file("cbc_unbounded.sol".to_string());
    let (status, _) = solver.read_solution(&"cbc_unbounded.sol".to_string()).unwrap();
    assert_eq!(status, Status::Unbounded);
}

#[test]
fn glpk_optimal() {
    let _ = fs::copy("tests/solution_files/glpk_optimal.sol", "glpk_optimal.sol");
    let solver = GlpkSolver::new().with_temp_solution_file("glpk_optimal.sol".to_string());
    let (status, mut variables) = solver.read_solution(&"glpk_optimal.sol".to_string()).unwrap();
    assert_eq!(status, Status::Optimal);
    assert_eq!(variables.remove("a"), Some(0f32));
    assert_eq!(variables.remove("b"), Some(5f32));
    assert_eq!(variables.remove("c"), Some(0f32));
}

#[test]
fn glpk_infeasible() {
    let _ = fs::copy(
        "tests/solution_files/glpk_infeasible.sol",
        "glpk_infeasible.sol",
    );
    let solver = GlpkSolver::new().with_temp_solution_file("glpk_infeasible.sol".to_string());
    let (status, _) = solver.read_solution(&"glpk_infeasible.sol".to_string()).unwrap();
    assert_eq!(status, Status::Infeasible);
}

#[test]
fn glpk_unbounded() {
    let _ = fs::copy(
        "tests/solution_files/glpk_unbounded.sol",
        "glpk_unbounded.sol",
    );
    let solver = GlpkSolver::new().with_temp_solution_file("glpk_unbounded.sol".to_string());
    let (status, _) = solver.read_solution(&"glpk_unbounded.sol".to_string()).unwrap();
    assert_eq!(status, Status::Unbounded);
}

#[test]
fn glpk_empty_col_bounds() {
    let _ = fs::copy(
        "tests/solution_files/glpk_empty_col_bounds.sol",
        "glpk_empty_col_bounds.sol",
    );
    let solver = GlpkSolver::new().with_temp_solution_file("glpk_empty_col_bounds.sol".to_string());
    let (status, solution) = solver.read_solution(&"glpk_empty_col_bounds.sol".to_string()).unwrap();
    assert_eq!(status, Status::Optimal);
    assert_eq!(1.0, *solution.get("a").unwrap());
    assert_eq!(0.0, *solution.get("b").unwrap());
}

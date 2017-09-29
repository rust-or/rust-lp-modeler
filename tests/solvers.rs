extern crate lp_modeler;

use lp_modeler::solvers::CbcSolver;
use lp_modeler::solvers::GlpkSolver;
use lp_modeler::solvers::Status;
use std::fs;

#[test]
fn cbc_optimal() {
    let _ = fs::copy("tests/solution_files/cbc_optimal.sol", "cbc_optimal.sol");
    let solver = CbcSolver::new();
    let solver2 = solver.temp_solution_file("cbc_optimal.sol".to_string());
    let (status, mut variables) = solver2.read_solution().unwrap();
    assert_eq!(status, Status::Optimal);
    assert_eq!(variables.remove("a"), Some(5f32));
    assert_eq!(variables.remove("b"), Some(6f32));
    assert_eq!(variables.remove("c"), Some(0f32));
}

#[test]
fn cbc_infeasible() {
    let _ = fs::copy("tests/solution_files/cbc_infeasible.sol", "cbc_infeasible.sol");
    let solver = CbcSolver::new();
    let solver2 = solver.temp_solution_file("cbc_infeasible.sol".to_string());
    let (status, _) = solver2.read_solution().unwrap();
    assert_eq!(status, Status::Infeasible);
}

#[test]
fn cbc_unbounded() {
    let _ = fs::copy("tests/solution_files/cbc_unbounded.sol", "cbc_unbounded.sol");
    let solver = CbcSolver::new();
    let solver2 = solver.temp_solution_file("cbc_unbounded.sol".to_string());
    let (status, _) = solver2.read_solution().unwrap();
    assert_eq!(status, Status::Unbounded);
}

#[test]
fn glpk_optimal() {
    let _ = fs::copy("tests/solution_files/glpk_optimal.sol", "glpk_optimal.sol");
    let solver = GlpkSolver::new();
    let solver2 = solver.temp_solution_file("glpk_optimal.sol".to_string());
    let (status, mut variables) = solver2.read_solution().unwrap();
    assert_eq!(status, Status::Optimal);
    assert_eq!(variables.remove("a"), Some(0f32));
    assert_eq!(variables.remove("b"), Some(5f32));
    assert_eq!(variables.remove("c"), Some(0f32));
}

#[test]
fn glpk_infeasible() {
    let _ = fs::copy("tests/solution_files/glpk_infeasible.sol", "glpk_infeasible.sol");
    let solver = GlpkSolver::new();
    let solver2 = solver.temp_solution_file("glpk_infeasible.sol".to_string());
    let (status, _) = solver2.read_solution().unwrap();
    assert_eq!(status, Status::Infeasible);
}

#[test]
fn glpk_unbounded() {
    let _ = fs::copy("tests/solution_files/glpk_unbounded.sol", "glpk_unbounded.sol");
    let solver = GlpkSolver::new();
    let solver2 = solver.temp_solution_file("glpk_unbounded.sol".to_string());
    let (status, _) = solver2.read_solution().unwrap();
    assert_eq!(status, Status::Unbounded);
}
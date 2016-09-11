use variables::*;
use std::ops::{AddAssign};

use std::collections::HashMap;

#[derive(Debug)]
pub enum Objective {
    Minimize,
    Maximize
}

#[derive(Debug)]
pub struct LpProblem {
    name: &'static str,
    objective_type: Objective,
    obj_expr: Option<LpExpression>,
    constraints: Vec<LpConstraint>

}

impl LpProblem {
    pub fn new(name: &'static str, objective: Objective) -> LpProblem {
        LpProblem { name: name, objective_type: objective, obj_expr: None, constraints: Vec::new() }
    }
}

impl AddAssign<LpConstraint> for LpProblem {
    fn add_assign(&mut self, _rhs: LpConstraint) {
        self.constraints.push(_rhs);
    }
}

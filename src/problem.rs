pub enum Objective {
   Minimize,
   Maximize
}

pub struct LpProblem {
   name: String,
   objective_type: Objective
}


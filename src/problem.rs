pub enum Objective {
    Minimize,
    Maximize
}

pub struct LpProblem {
    name: String,
    objective_type: Objective
}

impl LpProblem {
    pub fn new(name: &str, objective: Objective) -> LpProblem {
        LpProblem { name: name.to_string(), objective_type: objective }
    }
}



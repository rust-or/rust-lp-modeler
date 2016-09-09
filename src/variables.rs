use std::ops::Add;

pub enum LpType {
    Binary,
    Integer,
    Continuous
}

#[derive(Debug)]
pub enum LpVariable {
    BinaryVariable {
        name: String,
    },
    IntegerVariable {
        name: String,
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    },
    ContinuousVariable {
        name: String,
        lower_bound: Option<i32>,
        upper_bound: Option<i32>,
    }
}

impl LpVariable {
    pub fn new(name: &str, var_type: LpType) -> LpVariable {
        match var_type {
            LpType::Binary => LpVariable::BinaryVariable { name: name.to_string() },
            LpType::Integer => LpVariable::IntegerVariable { name: name.to_string(), lower_bound: None, upper_bound: None },
            LpType::Continuous => LpVariable::ContinuousVariable { name: name.to_string(), lower_bound: None, upper_bound: None }

        }
    }
    fn lower_bound(&self, lw: i32) -> LpVariable {
        match self {
            &LpVariable::BinaryVariable { name: ref n } =>
                LpVariable::BinaryVariable {
                    name: n.clone()
                },
            &LpVariable::IntegerVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                LpVariable::IntegerVariable {
                    name: n.clone(),
                    lower_bound: Some(lw),
                    upper_bound: u
                },
            &LpVariable::ContinuousVariable { name: ref n, lower_bound: _, upper_bound: u } =>
                LpVariable::ContinuousVariable {
                    name: n.clone(),
                    lower_bound: Some(lw),
                    upper_bound: u
                }
        }
    }
    fn upper_bound(&self, up: i32) -> LpVariable {
        match self {
            &LpVariable::BinaryVariable { name: ref n } =>
                LpVariable::BinaryVariable {
                    name: n.clone()
                },
            &LpVariable::IntegerVariable { name: ref n, lower_bound: l, upper_bound: _ } =>
                LpVariable::IntegerVariable {
                    name: n.clone(),
                    lower_bound: l,
                    upper_bound: Some(up)
                },
            &LpVariable::ContinuousVariable { name: ref n, lower_bound: l, upper_bound: _ } =>
                LpVariable::ContinuousVariable {
                    name: n.clone(),
                    lower_bound: l,
                    upper_bound: Some(up)
                }
        }
    }
}

enum LpExpression {
    LpElement,
    Expr(Box<LpExpression>)
}

impl Add for LpVariable {
    type Output = LpVariable;
    fn add(self, _rhs: LpVariable) -> LpVariable {
        println!("Adding!");
        self
    }
}

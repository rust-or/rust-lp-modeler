pub trait LpElement {
    fn new(name: &str) -> Self;
    fn lower_bound(&self, up: i32) -> Self;
    fn upper_bound(&self, lw: i32) -> Self;
}

pub enum LpVariable {
    BinaryVariable,
    IntegerVariable,
    ContinuousVariable
}

#[derive(Debug)]
pub struct BinaryVariable {
    name: String,
}

#[derive(Debug)]
pub struct IntegerVariable {
    name: String,
    lower_bound: Option<i32>,
    upper_bound: Option<i32>,
}

#[derive(Debug)]
pub struct ContinuousVariable {
    name: String,
    lower_bound: Option<i32>,
    upper_bound: Option<i32>,
}

impl LpElement for BinaryVariable {
    fn new(name: &str) -> Self {
        BinaryVariable { name: name.to_string() }
    }
    fn lower_bound(&self, _lw: i32) -> Self {
        BinaryVariable { name: self.name.clone() }
    }
    fn upper_bound(&self, _up: i32) -> Self {
        BinaryVariable { name: self.name.clone() }
    }
}

impl LpElement for IntegerVariable {
    fn new(name: &str) -> Self {
        IntegerVariable { name: name.to_string(), lower_bound: None, upper_bound: None }
    }
    fn lower_bound(&self, lw: i32) -> Self {
        IntegerVariable { name: self.name.clone(), lower_bound: Some(lw), upper_bound: self.upper_bound}
    }
    fn upper_bound(&self, up: i32) -> Self {
        IntegerVariable { name: self.name.clone(), lower_bound: self.lower_bound, upper_bound: Some(up)}
    }
}

impl LpElement for ContinuousVariable {
    fn new(name: &str) -> Self {
        ContinuousVariable { name: name.to_string(), lower_bound: None, upper_bound: None}
    }
    fn lower_bound(&self, lw: i32) -> Self {
        ContinuousVariable { name: self.name.clone(), lower_bound: Some(lw), upper_bound: self.upper_bound}
    }
    fn upper_bound(&self, up: i32) -> Self {
        ContinuousVariable { name: self.name.clone(), lower_bound: self.lower_bound, upper_bound: Some(up)}
    }
}


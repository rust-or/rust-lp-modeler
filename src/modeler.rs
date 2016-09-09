#[derive(Debug)]
/*
pub enum Category {
    Integer,
    Binary,
    Continuous
}


trait LpElement {
}
*/

/*
#[derive(Debug)]
pub struct LpVariable {
    name: String,
    lower_bound: Option<i32>,
    upper_bound: Option<i32>,
    category: Category
}

impl LpVariable {
    pub fn new(name: String) -> LpVariable {
        LpVariable { name: name, lower_bound: None, upper_bound: None, category: Category::Continuous }
    }

    pub fn new(name: String, category: Category) -> LpVariable {
        match category {
            Category::Integer => LpVariable { name: name, lower_bound: None, upper_bound: None, category: category },
            Category::Binary => LpVariable { name: name, lower_bound: Some(0), upper_bound: Some(1), category: category },
            Category::Continuous => LpVariable { name: name, lower_bound: None, upper_bound: None, category: category }
        }
    }
}
*/

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

impl BinaryVariable {
    pub fn new(name: &str) -> BinaryVariable {
        BinaryVariable { name: name.to_string() }
    }
}

impl IntegerVariable {
    pub fn new(name: &str, lower_bound: Option<i32>, upper_bound: Option<i32>) -> IntegerVariable {
        IntegerVariable { name: name.to_string(), lower_bound: lower_bound, upper_bound: upper_bound }
    }
}

impl ContinuousVariable {
    pub fn new(name: &str, lower_bound: Option<i32>, upper_bound: Option<i32>) -> ContinuousVariable {
        ContinuousVariable { name: name.to_string(), lower_bound: lower_bound, upper_bound: upper_bound }
    }
}

/* TODO: use a builder */

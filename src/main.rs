extern crate lp_modeler;

use lp_modeler::modeler::*;

fn main() {

    let b = BinaryVariable::new("toto");
    let c = ContinuousVariable::new("t", None, None);
    let c1 = ContinuousVariable::new("t", None, Some(2));

//    let l1 = LpVariable::new("l1".to_string(), Category::Binary);
    println!("{:?}", c1);
}

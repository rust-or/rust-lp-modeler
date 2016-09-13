use std;
use variables::*;
use variables::LpExpression::*;

//use variables::LpExpression::{AddExpr, MulExpr};
use std::ops::{AddAssign};

use std::collections::HashMap;

/// Enum helping to specify the objective function of the linear problem.
///
/// # Exemples:
///
/// ```
/// let mut problem = LpProblem::new("One Problem", Objective::Maximize);
/// ```
#[derive(Debug)]
pub enum Objective {
    Minimize,
    Maximize
}

/// Structure used for creating the model and solving a linear problem.
///
/// # Exemples:
///
/// ```
/// let mut problem = LpProblem::new("One Problem", Objective::Maximize);
/// problem += (a + b).lt(100);
/// problem += a.gt(b);
/// problem += 2*a + 3*b;
///
/// problem.solve();
/// ```
#[derive(Debug)]
pub struct LpProblem {
    name: &'static str,
    objective_type: Objective,
    obj_expr: Option<LpExpression>,
    constraints: Vec<LpConstraint>

}

impl LpProblem {

    /// Create a new problem
    pub fn new(name: &'static str, objective: Objective) -> LpProblem {
        LpProblem { name: name, objective_type: objective, obj_expr: None, constraints: Vec::new() }
    }

    /// Solve the LP model
    pub fn solve(&self) {
        println!("Mhmmmm, solving :-)");
    }

    pub fn variables(&self) -> Vec<&LpExpression> {

        fn var<'a>(expr: &'a LpExpression, lst: &mut Vec<&'a LpExpression>) {
            match expr {
                &BinaryVariable {..} | &IntegerVariable {..} | &ContinuousVariable {..} => { lst.push(expr); },
                &MulExpr(_, ref e) => { var(&*e, lst); },
                &AddExpr(ref e1, ref e2) => {
                    var(&*e1, lst);
                    var(&*e2, lst);
                },
                _ => ()
            }
        }

        let mut lst = Vec::new();
        for e in &self.constraints {
            var(&e.0, &mut lst);
        }

        lst
    }

    fn objective_string(&self) -> String {
        println!("{:?}", self.obj_expr);
        "Toto".to_string()
    }
    pub fn write_lp(&self) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut buffer = try!(File::create("foo.txt"));

        match self.objective_type {
            Objective::Maximize => { try!(buffer.write(b"Maximize")); },
            Objective::Minimize => { try!(buffer.write(b"Minimize")); },
        }

        self.objective_string();





        Ok(())



        /*
        let path = Path::new("test.lp");
        let display = path.display();

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
            Ok(file) => file,
        };

        // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
        match file.write("toto".as_bytes()) {
            Err(why) => {
                panic!("couldn't write to {}: {}", display, why.description())
            },
            Ok(_) => (),
        }
        */


    }
}

impl AddAssign<LpConstraint> for LpProblem {
    fn add_assign(&mut self, _rhs: LpConstraint) {
        self.constraints.push(_rhs);
    }
}

impl<T> AddAssign<T> for LpProblem where T: Into<LpExpression>{
    fn add_assign(&mut self, _rhs: T) {
        //TODO: improve without cloning
        if let Some(e) = self.obj_expr.clone() {
            self.obj_expr = Some(AddExpr(Box::new(_rhs.into()), Box::new(e.clone())));
        } else {
            self.obj_expr = Some(_rhs.into());
        }
    }
}


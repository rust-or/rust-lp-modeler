# lp-modeler
[![MIT license](http://img.shields.io/badge/license-MIT-brightgreen.svg)](http://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/jcavat/rust-lp-modeler.svg?branch=master)](https://travis-ci.org/jcavat/rust-lp-modeler)
[![Build status](https://ci.appveyor.com/api/projects/status/5i63bu7rn3m5d4l3?svg=true)](https://ci.appveyor.com/project/jcavat/rust-lp-modeler)

This project provides a mathematical programming modeling library for the Rust language (v1.22+).

An optimization problem (e.g. an integer or linear programme) can be formulated using familiar Rust syntax (see examples), and written into a universal [LP model format](https://www.gurobi.com/documentation/8.0/refman/lp_format.html). This can then be processed by a mixed integer programming solver. Presently supported solvers are; COIN-OR CBC, Gurobi and GLPK.

This project is inspired by [COIN-OR PuLP](http://www.coin-or.org/PuLP/ "Coin-Or PuLP website") which provides
such a library for Python.

## Usage

These examples present a formulation (in LP model format), and demonstrate the Rust code required to generate this formulation. Code can be found in [tests/problems.rs](tests/problems.rs).

### Example 1 - Simple model

#### Formulation
```
\ One Problem

Maximize
  10 a + 20 b

Subject To
  c1: 500 a + 1200 b <= 10000
  c2: a - b <= 0

Generals
  a c b 

End
```

#### Rust code
```rust
use lp_modeler::problem::{LpObjective, Problem, LpProblem};
use lp_modeler::operations::{LpOperations};
use lp_modeler::variables::LpInteger;
use lp_modeler::solvers::{SolverTrait, CbcSolver};

// Define problem variables
let ref a = LpInteger::new("a");
let ref b = LpInteger::new("b");
let ref c = LpInteger::new("c");

// Define problem and objective sense
let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);

// Objective Function: Maximize 10*a + 20*b
problem += 10.0 * a + 20.0 * b;

// Constraint: 500*a + 1200*b + 1500*c <= 10000
problem += (500*a + 1200*b + 1500*c).le(10000);

// Constraint: a <= b
problem += (a).le(b);

// Specify solver
let solver = CbcSolver::new();

// Run optimisation and process output hashmap
match solver.run(&problem) {
    Ok((status, var_values)) => {
        println!("Status {:?}", status);
        for (name, value) in var_values.iter() {
            println!("value of {} = {}", name, value);
        }
    },
    Err(msg) => println!("{}", msg),
}
```

To generate the LP file which is shown above:
```rust
problem.write_lp("problem.lp")
```

### Example 2 - An Assignment model

#### Formulation

This more complex formulation programmatically generates the expressions for the objective and constraints.

We wish to maximise the quality of the pairing between a group of men and women, based on their mutual compatibility score. Each man must be assigned to exactly one woman, and vice versa.

###### Compatibility Score Matrix

| | Abe | Ben | Cam |
| --- | --- | --- | --- |
| **Deb** | 50 | 60 | 60 |
| **Eve** | 75 | 95 | 70 |
| **Fay** | 75 | 80 | 80 |

This problem is formulated as an [Assignment Problem](https://en.wikipedia.org/wiki/Assignment_problem).

#### Rust code
```rust
extern crate lp_modeler;
#[macro_use] extern crate maplit;

use std::collections::HashMap;

use lp_modeler::variables::*;
use lp_modeler::variables::LpExpression::*;
use lp_modeler::operations::LpOperations;
use lp_modeler::problem::{LpObjective, LpProblem, LpFileFormat};
use lp_modeler::solvers::{SolverTrait, CbcSolver};

// Problem Data
let men = vec!["A", "B", "C"];
let women = vec!["D", "E", "F"];
let compat_scores = hashmap!{
    ("A", "D") => 50.0,
    ("A", "E") => 75.0,
    ("A", "F") => 75.0,
    ("B", "D") => 60.0,
    ("B", "E") => 95.0,
    ("B", "F") => 80.0,
    ("C", "D") => 60.0,
    ("C", "E") => 70.0,
    ("C", "F") => 80.0,
};

// Define Problem
let mut problem = LpProblem::new("Matchmaking", LpObjective::Maximize);

// Define Variables
let mut vars = HashMap::new();
for m in &men{
    for w in &women{
        vars.insert((m, w), LpBinary::new(&format!("{}_{}", m, w)));
    }
}

// Define Objective Function
let mut obj_vec: Vec<LpExpression> = Vec::new();
for (&(&m, &w), var) in &vars{
    let obj_coef = compat_scores.get(&(m, w)).unwrap();
    obj_vec.push(*obj_coef * var);
}
problem += lp_sum(&obj_vec);

// Define Constraints
// Constraint 1: Each man must be assigned to exactly one woman
for m in &men{
    let mut constr_vec = Vec::new();

    for w in &women{
        constr_vec.push(1.0 * vars.get(&(m, w)).unwrap());
    }

    problem += lp_sum(&constr_vec).equal(1);
}

// Constraint 2: Each woman must be assigned to exactly one man
for w in &women{
    let mut constr_vec = Vec::new();

    for m in &men{
        constr_vec.push(1.0 * vars.get(&(m, w)).unwrap());
    }

    problem += lp_sum(&constr_vec).equal(1);
}

// Run Solver
let solver = CbcSolver::new();
let result = solver.run(&problem);

// Terminate if error, or assign status & variable values
assert!(result.is_ok(), result.unwrap_err());
let (solver_status, var_values) = result.unwrap();

// Compute final objective function value
let mut obj_value = 0f32;
for (&(&m, &w), var) in &vars{
    let obj_coef = compat_scores.get(&(m, w)).unwrap();
    let var_value = var_values.get(&var.name).unwrap();
    
    obj_value += obj_coef * var_value;
}

// Print output
println!("Status: {:?}", solver_status);
println!("Objective Value: {}", obj_value);
// println!("{:?}", var_values);
for (var_name, var_value) in &var_values{
    let int_var_value = *var_value as u32;
    if int_var_value == 1{
        println!("{} = {}", var_name, int_var_value);
    }
}
```

This code computes the objective function value and processes the output to print the chosen pairing of men and women:
```
Status: Optimal
Objective Value: 230
B_E = 1
C_D = 1
A_F = 1
```

## New features
### 0.3.1
* Add distributive property (ex: `3 * (a + b + 2) = 3*a + 3*b + 6`)
* Add trivial rules (ex: `3 * a * 0 = 0` or `3 + 0 = 3`)
* Add commutative property to simplify some computations
* Support for GLPK

### 0.3.0
* Functional lib with simple algebra properties

## Contributors
* Joel Cavat [(jcavat)](https://github.com/jcavat)
* Thomas Vincent [(tvincent2)](https://github.com/tvincent2)
* Antony Phillips [(aphi)](https://github.com/aphi)

## Further work
* Config for lp_solve and CPLEX
* 'complex' algebra operations such as commutative and distributivity
* It would be great to use some constraint for binary variables such as 
    * a && b which is the constraint a + b = 2
    * a || b which is the constraint a + b >= 1
    * a <=> b which is the constraint a = b
    * a => b which is the constraint a <= b
    * All these cases are easy with two constraints but more complex with expressions
    * ...
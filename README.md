# lp-modeler
[![MIT license](http://img.shields.io/badge/license-MIT-brightgreen.svg)](http://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/jcavat/rust-lp-modeler.svg?branch=master)](https://travis-ci.org/jcavat/rust-lp-modeler)
[![Build status](https://ci.appveyor.com/api/projects/status/5i63bu7rn3m5d4l3?svg=true)](https://ci.appveyor.com/project/jcavat/rust-lp-modeler)

A linear programming modeller written in Rust. This api helps to write LP model and 
use solver such as CBC, Gurobi, lp\_solve, ...*

This library is inspired by [coin-or PuLP](http://www.coin-or.org/PuLP/ "Coin-Or PuLP website") which provide
such an API for python 2.x.

## Usage
Dev in progress.


This first alpha version provide this SDL to make a LP Model :
```rust

use lp_modeler::operations::{LpOperations};
use lp_modeler::problem::{LpProblem, LpObjective};
use lp_modeler::variables::{LpVariable, LpType};
use lp_modeler::solvers::*;

let ref a = LpVariable::new("a", LpType::Integer);
let ref b = LpVariable::new("b", LpType::Integer);

let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);

problem += 10 * a + 20 * b; // Add the objective function (expression)
problem += (500 * a + 1200 * b).le(10000); // Add a constraint expression . 500a + 1200b <= 10000
problem += (a).le(b); // Add a constraint expression a <= b

match problem.solve(GurobiSolver) {
    Ok((status, res)) => {
        println!("Status {:?}", status);
        for (name, value) in res.iter() {
            println!("value of {} = {}", name, value);
        }
    },
    Err(msg) => println!("{}", msg),
}

```

This version are tested with coinor-cbc.

It is already possible to export this model 
into the [lp file format](https://www.gurobi.com/documentation/6.5/refman/lp_format.html "lp file format on Gurobi website"). 
```
problem.write_lp("problem.lp") 
```

will produce :

```
\ One Problem

Maximize
  10 a + 20 b

Subject To
  c1: 500 a + 1200 b <= -10000
  c2: a - b <= 0

Generals
  b a 

End
```

With this file, you can directly use it 
with a solver supporting lp file format :
* open source solvers :
    * lp_solve
    * glpk
    * cbc
* commercial solvers :
    * Gurobi
    * Cplex
    
## Limitation
* Use with CBC for now

## Todo
* Config for GLPK, lp_solve and CPLEX

## Further work
* it would be great to use some constraint for binary variable like 
    * a && b which is the constraint a + b = 2
    * a || b which is the constraint a + b >= 1
    * a <=> b which is the constraint a = b
    * a => b which is the constraint a <= b
    * All these cases is easy with two constraints but more complex with expressions
    * ...


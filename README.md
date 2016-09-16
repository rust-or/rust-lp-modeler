# lp-modeler
* A linear programming modeler written in Rust. This api helps to write LP model and use solver such as CBC, Gurobi, lp\_solve, ...*

## Usage
Dev in progress.


The first usable version will provide this SDL to make a LP Model :
```rust
use lp_modeler::problem::{LpObjective, LpProblem};
use lp_modeler::variables::{LpVariable, LpType, LpOperations};

let ref a = LpVariable::new("a", LpType::Integer);
let ref b = LpVariable::new("b", LpType::Integer);

let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);
problem += (a + b).lt(100);
problem += a.gt(b);
problem += 2*a + 3*b;

problem.solve();
```
# lp-modeler
[![MIT license](http://img.shields.io/badge/license-MIT-brightgreen.svg)](http://opensource.org/licenses/MIT)
[![Build Status](https://travis-ci.org/jcavat/rust-lp-modeler.svg?branch=master)](https://travis-ci.org/jcavat/rust-lp-modeler)
[![Gitter](https://badges.gitter.im/rust-lp-modeler/community.svg)](https://gitter.im/rust-lp-modeler/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
[![Github Actions](https://github.com/jcavat/rust-lp-modeler/workflows/Rust/badge.svg)](https://github.com/jcavat/rust-lp-modeler/actions)
This project provides a mathematical programming modeling library for Rust.

An optimization problem (e.g. an integer or linear programme) can be formulated using familiar Rust syntax (see examples), and written into a universal [LP model format](https://www.gurobi.com/documentation/8.0/refman/lp_format.html).
This can then be processed by a mixed integer programming solver.
Presently supported solvers that require a [separate installation (see below the examples section)](#installing-external-solvers) to be present at runtime of your `lp_modeler`-based project are:
* [COIN-OR cbc](https://github.com/coin-or/Cbc)
* [Gurobi](https://www.gurobi.com/documentation/)
* [GLPK](https://www.gnu.org/software/glpk/)

Presently supported solvers that you can import as Rust crates (as [optional features](https://doc.rust-lang.org/cargo/reference/features.html)) are:
* [minilp](https://docs.rs/minilp/latest/minilp/)
* [coin_cbc](https://docs.rs/coin_cbc/latest/coin_cbc/) (requires the `Cbc` library files to be present at compile time of your `lp_modeler`-based project, see the [`coin_cbc` project README](https://github.com/KardinalAI/coin_cbc) for how to do this)

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
  c1: 500 a + 1200 b + 1500 c <= 10000
  c2: a - b <= 0

Generals
  a c b 

End
```

#### Rust code
```rust
extern crate lp_modeler;

use lp_modeler::solvers::{CbcSolver, SolverTrait};
use lp_modeler::dsl::*;
use lp_modeler::constraint;

fn main() {
    // Define problem variables
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");
    let ref c = LpInteger::new("c");

    // Define problem and objective sense
    let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);

    // Objective Function: Maximize 10*a + 20*b
    problem += 10.0 * a + 20.0 * b;

    // Constraint: 500*a + 1200*b + 1500*c <= 10000
    problem += constraint!(500*a + 1200*b + 1500*c <= 10000);

    // Constraint: a <= b
    problem += constraint!(a <= b);

    // Specify solver
    let solver = CbcSolver::new();

    // Run optimisation and process output hashmap
    match solver.run(&problem) {
        Ok(solution) => {
            println!("Status {:?}", solution.status);
            for (name, value) in solution.results.iter() {
                println!("value of {} = {}", name, value);
            }
        },
        Err(msg) => println!("{}", msg),
    }
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

use std::collections::HashMap;

use lp_modeler::dsl::*;
use lp_modeler::solvers::{SolverTrait, CbcSolver};

fn main() {
    // Problem Data
    let men = vec!["A", "B", "C"];
    let women = vec!["D", "E", "F"];
    let compatibility_score: HashMap<(&str, &str),f32> = vec![
        (("A", "D"), 50.0),
        (("A", "E"), 75.0),
        (("A", "F"), 75.0),
        (("B", "D"), 60.0),
        (("B", "E"), 95.0),
        (("B", "F"), 80.0),
        (("C", "D"), 60.0),
        (("C", "E"), 70.0),
        (("C", "F"), 80.0),
    ].into_iter().collect();

    // Define Problem
    let mut problem = LpProblem::new("Matchmaking", LpObjective::Maximize);

    // Define Variables
    let vars: HashMap<(&str,&str), LpBinary> =
        men.iter()
            .flat_map(|&m| women.iter()
            .map(move |&w| {
                let key = (m,w);
                let value = LpBinary::new(&format!("{}_{}", m,w));
                (key, value)
            }))
            .collect();

    // Define Objective Function
    let obj_vec: Vec<LpExpression> = {
       vars.iter().map( |(&(m,w), bin)| {
           let &coef = compatibility_score.get(&(m, w)).unwrap();
           coef * bin
       } )
    }.collect();
    problem += obj_vec.sum();

    // Define Constraints
    // - constraint 1: Each man must be assigned to exactly one woman
    for &m in &men{
        problem += sum(&women, |&w| vars.get(&(m,w)).unwrap() ).equal(1);
    }

    // - constraint 2: Each woman must be assigned to exactly one man
    for &w in &women{
        problem += sum(&men, |&m| vars.get(&(m,w)).unwrap() ).equal(1);
    }

    // Run Solver
    let solver = CbcSolver::new();
    let result = solver.run(&problem);

    // Compute final objective function value
    // (terminate if error, or assign status & variable values)
    assert!(result.is_ok(), result.unwrap_err());
    let (status, results) = result.unwrap();
    let mut obj_value = 0f32;
    for (&(m, w), var) in &vars{
        let obj_coef = compatibility_score.get(&(m, w)).unwrap();
        let var_value = results.get(&var.name).unwrap();

        obj_value += obj_coef * var_value;
    }

    // Print output
    println!("Status: {:?}", status);
    println!("Objective Value: {}", obj_value);
    for (var_name, var_value) in &results{
        let int_var_value = *var_value as u32;
        if int_var_value == 1{
            println!("{} = {}", var_name, int_var_value);
        }
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

## installing external solvers

### installing conda (package manager)

If you want the latest release version of Cbc, Gurobi or GLPK, the easiest cross-platform installation pathway should be via [conda](https://docs.conda.io/en/latest/). 
Importantly, this does not require admin rights on the system you want to install it on.
All you need to do is [install conda](https://docs.conda.io/projects/conda/en/latest/user-guide/install/index.html).
Once this is done, use the respective conda command for the solver you want to use (see below).

### COIN-OR Cbc

#### latest release (via conda)

To get the latest Cbc release for your system with conda (installation [see above](#installing-conda-package-manager)), use this command:
```
conda create -n coin-or-cbc -c conda-forge coin-or-cbc
```
Then activating the newly created environment will make the `cbc` executable available:
```
conda activate coin-or-cbc
```

#### latest release (via coinbrew)

To get the latest Cbc release, including the .
We recommend using COIN-OR's `coinbrew`, as described here:
https://coin-or.github.io/user_introduction#building-from-source

#### latest commit (via coinbrew)

To get the very latest Cbc version, including unreleased bug fixes, you will need to [build it from source](https://coin-or.github.io/user_introduction#building-from-source).
We recommend using COIN-OR's `coinbrew`, as described here:
https://coin-or.github.io/user_introduction#building-from-source


### GLPK

#### recent release (via conda)

To get a recent release of GLPK for your system with conda, use this command:
```
conda create -n glpk -c conda-forge glpk
```
Then activating the newly created environment will make the `glpsol` executable available:
```
conda activate glpk
```

### Gurobi

#### latest release (via conda)

To use Gurobi, **you need to have a valid [license key](https://www.gurobi.com/downloads/)** and have it in a location that Gurobi can find it.
Once you have a valid license, you can get the latest Gurobi release for your system with conda, use this command:
```
conda create -n gurobi -c gurobi gurobi
```
Then activating the newly created environment will make the `gurobi_cl` executable available:
```
conda activate gurobi
```

## Changelog

### 0.5.0

* Add a native `minilp` impl to call the Rust native solver `minilp`
* Changed `coin_cbc`-based `NativeCbcSolver` to an optional feature
* Fix adding upper bounds to `NativeCbc`
* Add a `coinstraint!()` macro
* Add `AddAssign`, `SubAssign` and `MulAssign` traits
* Reworked various internal functions to remove recursions (fixes related stack overflows)
* Add install infos for the solvers to the docs

### 0.4.3

* Add a native coin-or impl (NativeCBCSolver) to call CoinOR CBC trough the C API.

### 0.4.2

* Fix incorrect simplification of (expr-c1)+c2 

### 0.4.1

* Fix failed cbc parsing on infeasible solution

### 0.4.0

* Improve modules
  * Remove maplit dependency
  * All the features to write expressions and constraints are put into `dsl` module
  * `use lp_modeler::dsl::*` is enough to write a system
  * `use lp_modeler::solvers::*` is always used to choose a solver
* Add a `sum()` method for vector of `LpExpression`/`Into<LpExpression>` instead of `lp_sum()` function
* Add a `sum()` function used in the form:
  
  ```rust
  problem += sum(&vars, |&v| v * 10.0) ).le(10.0);
  ```

### 0.3.3

* Fix and improve error message for GLPK solution parsing
* Format code with rust fmt

### 0.3.3

* Add new examples in documentation
* Improve 0.0 comparison

### 0.3.1
* Add distributive property (ex: `3 * (a + b + 2) = 3*a + 3*b + 6`)
* Add trivial rules (ex: `3 * a * 0 = 0` or `3 + 0 = 3`)
* Add commutative property to simplify some computations
* Support for GLPK

### 0.3.0
* Functional lib with simple algebra properties

## Contributors

### Main contributor

* Joel Cavat [(jcavat)](https://github.com/jcavat)

### All contributions :heart:

* Thomas Vincent [(tvincent2)](https://github.com/tvincent2)
* Antony Phillips [(aphi)](https://github.com/aphi)
* Florian B. [(Lesstat)](https://github.com/Lesstat)
* Amila Welihinda [(amilajack)](https://github.com/amilajack)
* [(zappolowski)](https://github.com/zappolowski)
* Yisu Rem Wang [(remysucre)](https://github.com/remysucre)
* Tony Cox [(tony-cox)](https://github.com/tony-cox)
* [EdorianDark](https://github.com/EdorianDark)
* Colman Humphrey [(ColmanHumphrey)](https://github.com/ColmanHumphrey)
* Stephan Beyer [sbeyer](https://github.com/sbeyer)
* Ophir Lojkine [lovasoa](https://github.com/lovasoa)
* David Lähnemann [dlaehnemann](https://github.com/dlaehnemann)

## Further work

* Parse and provide the objective value 
* Config for lp_solve and CPLEX
* It would be great to use some constraint for binary variables such as 
    * a && b which is the constraint a + b = 2
    * a || b which is the constraint a + b >= 1
    * a <=> b which is the constraint a = b
    * a => b which is the constraint a <= b
    * All these cases are easy with two constraints but more complex with expressions
    * ...

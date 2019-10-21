extern crate lp_modeler;

use lp_modeler::solvers::*;
use lp_modeler::dsl::*;

fn main() {
    let ref a = LpInteger::new("a");
    let ref b = LpInteger::new("b");

    let mut problem = LpProblem::new("Problem", LpObjective::Maximize);

    // Objective Function: Maximize 10*a + 20*b
    problem += 10.0 * a + 20.0 * b;

    // Constraint: 500*a + 1200*b + 1500*c <= 10000
    problem += (500*a + 1200*b).le(10000);

    // Constraint: a <= b
    problem += (a).le(b);

    //let _ = problem.write_lp("toto.lp");

//    let solver = GurobiSolver::new();
    let solver = CbcSolver::new().with_nb_threads(2);

    match solver.run(&problem) {
        Ok(solution) => {
            println!("Status {:?}", solution.status);
            for (name, value) in solution.results.iter() {
                println!("value of {} = {}", name, value);
            }
            println!("Value a: {}", solution.get_int(a) )
        }
        Err(msg) => println!("{}", msg),
    }
}

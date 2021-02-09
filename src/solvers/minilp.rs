use dsl::{LpObjective, LpProblem, LpConstraint, LpExpression, LpContinuous, simplify, Constraint};
use std::collections::HashMap;
use dsl::LpExpression::LitVal;
use solvers::{ SolverTrait, Solution, Status};

fn direction_to_minilp(objective: &LpObjective) -> minilp::OptimizationDirection {
    match objective {
        LpObjective::Maximize => minilp::OptimizationDirection::Maximize,
        LpObjective::Minimize => minilp::OptimizationDirection::Minimize,
    }
}

fn add_constraint_to_minilp(
    constraint: &LpConstraint,
    variables: &mut HashMap<String, minilp::Variable>,
    pb: &mut minilp::Problem,
) -> Result<(), String> {
    match constraint.generalize() {
        LpConstraint(expr, op, LitVal(constant)) => {
            let expr_variables = decompose_expression(&expr)?;
            let mut expr = minilp::LinearExpr::empty();
            for (name, coefficient) in expr_variables.0 {
                let var = variables.entry(name).or_insert_with(|| {
                    pb.add_var(0., (f64::NEG_INFINITY, f64::INFINITY))
                }).clone();
                expr.add(var, coefficient.coefficient.into());
            }
            let op = comparison_to_minilp(op);
            pb.add_constraint(expr, op, f64::from(constant));
            Ok(())
        }
        _ => Err("constraint generalization failed".into())
    }
}

fn comparison_to_minilp(op: Constraint) -> minilp::ComparisonOp {
    match op {
        Constraint::GreaterOrEqual => minilp::ComparisonOp::Ge,
        Constraint::LessOrEqual => minilp::ComparisonOp::Le,
        Constraint::Equal => minilp::ComparisonOp::Eq
    }
}

#[derive(Debug, PartialEq)]
struct VarWithCoeff {
    coefficient: f32,
    min: f32,
    max: f32,
}

impl Default for VarWithCoeff {
    fn default() -> Self {
        VarWithCoeff { coefficient: 0., min: f32::NEG_INFINITY, max: f32::INFINITY }
    }
}

#[derive(Debug, Default, PartialEq)]
struct VarList(HashMap<String, VarWithCoeff>);

impl VarList {
    fn add(&mut self, var: LpContinuous, coefficient: f32) {
        let LpContinuous { name, lower_bound, upper_bound } = var;
        let prev = self.0.entry(name).or_default();
        prev.coefficient += coefficient;
        if let Some(lower) = lower_bound {
            prev.min = prev.min.max(lower);
        }
        if let Some(upper) = upper_bound {
            prev.max = prev.max.min(upper);
        }
    }
}

fn decompose_expression(
    expr: &LpExpression,
) -> Result<VarList, String> {
    fn decompose_expression_recursive(
        expr: LpExpression,
        mut decomposed: VarList,
    ) -> Result<VarList, String> {
        match expr {
            LpExpression::ConsCont(var) => { decomposed.add(var, 1.) }
            LpExpression::MulExpr(lhs, rhs) => {
                match (*lhs, *rhs) {
                    (LpExpression::LitVal(lit), LpExpression::ConsCont(var)) => {
                        decomposed.add(var, lit)
                    }
                    (a, b) => {
                        return Err(format!("Non-simplified multiplication: {:?} * {:?}", a, b));
                    }
                }
            }
            LpExpression::AddExpr(left, right) => {
                decomposed = decompose_expression_recursive(*left, decomposed)?;
                decomposed = decompose_expression_recursive(*right, decomposed)?;
            }
            x => return Err(format!("Unsupported expression: {:?}", x))
        };
        Ok(decomposed)
    }

    let simplified = simplify(expr);
    decompose_expression_recursive(simplified, VarList::default())
}


/// Returns a map from dsl variable name to minilp variable
fn add_objective_to_minilp(
    objective: &LpExpression,
    pb: &mut minilp::Problem,
) -> Result<HashMap<String, minilp::Variable>, String> {
    let vars = decompose_expression(objective)?;
    Ok(vars.0.into_iter()
        .map(|(name, VarWithCoeff { coefficient, min, max })| {
            let var = pb.add_var(
                coefficient.into(),
                (min.into(), max.into()),
            );
            (name, var)
        }).collect()
    )
}

fn problem_to_minilp(pb: &LpProblem) -> Result<(minilp::Problem, Vec<Option<String>>), String> {
    let objective = direction_to_minilp(&pb.objective_type);
    let mut minilp_pb = minilp::Problem::new(objective);
    let objective = pb.obj_expr.as_ref().ok_or("Missing objective")?;
    let mut minilp_variables = add_objective_to_minilp(objective, &mut minilp_pb)?;
    for constraint in &pb.constraints {
        add_constraint_to_minilp(
            constraint,
            &mut minilp_variables,
            &mut minilp_pb,
        )?;
    }
    let mut ordered_vars = vec![None; minilp_variables.len()];
    for (name, var) in minilp_variables {
        ordered_vars[var.idx()] = Some(name);
    }
    Ok((minilp_pb, ordered_vars))
}

pub struct MiniLpSolver;

impl MiniLpSolver {
    pub fn new() -> Self { Self }
}

impl SolverTrait for MiniLpSolver {
    type P = LpProblem;

    fn run<'a>(&self, problem: &'a Self::P) -> Result<Solution<'a>, String> {
        let (minilp_pb, variable_names) = problem_to_minilp(problem)?;
        let minilp_result = minilp_pb.solve();
        solution_from_minilp(minilp_result, variable_names)
    }
}

fn solution_from_minilp(
    result: Result<minilp::Solution, minilp::Error>,
    mut variable_names: Vec<Option<String>>,
) -> Result<Solution<'static>, String> {
    match result {
        Ok(solution) => {
            let results: Option<HashMap<String, f32>> = solution.iter()
                .map(|(var, &value)| {
                    std::mem::take(&mut variable_names[var.idx()]).map(|name| {
                        (name, value as f32)
                    })
                })
                .collect();
            if let Some(results) = results {
                Ok(Solution::new(Status::Optimal, results))
            } else {
                Err("missing variable name".into())
            }
        }
        Err(minilp::Error::Unbounded) => {
            Ok(Solution::new(Status::Unbounded, HashMap::new()))
        }
        Err(minilp::Error::Infeasible) => {
            Ok(Solution::new(Status::Infeasible, HashMap::new()))
        }
    }
}

#[test]
fn test_decompose() {
    let ref a = LpContinuous::new("a");
    let ref b = LpContinuous::new("b");
    let expr = (4 * (3 * a - b * 2 + a)) * 1 + b;
    let decomposed = decompose_expression(&expr);
    let mut expected = VarList::default();
    expected.add(a.clone(), 4. * 3. + 4.);
    expected.add(b.clone(), 4. * (-2.) + 1.);
    assert_eq!(decomposed, Ok(expected));
}

#[test]
fn test_solve() {
    use dsl::operations::LpOperations;
    let ref a = LpContinuous::new("a");
    let ref b = LpContinuous::new("b");

    // Define problem and objective sense
    let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);
    problem += 10 * a + 20 * b;
    problem += (500 * a - 1000 * b).ge(10000);
    problem += (a).le(b);

    let expected: HashMap<String, f32> = vec![
        ("a".into(), -20.),
        ("b".into(), -20.)
    ].into_iter().collect();
    let actual = MiniLpSolver::new().run(&problem).expect("could not solve").results;
    assert_eq!(actual, expected);
}
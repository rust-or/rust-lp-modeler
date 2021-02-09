pub fn is_zero(n: f32) -> bool {
    n.abs() < 0.00001
}

/// This macro allows defining constraints using 'expression1 <= expression2'
/// instead of `expression1.le(expression2)`. 
/// 
/// # Example:
///
/// ```
/// use lp_modeler::dsl::*;
/// use lp_modeler::constraint;
///
/// let ref a = LpInteger::new("a");
/// let ref b = LpInteger::new("b");
///
/// let mut problem = LpProblem::new("One Problem", LpObjective::Maximize);
/// problem += 5*a + 3*b;
/// problem += constraint!(a + b*2 <= 10);
/// problem += constraint!(b >= a);
/// ```
#[macro_export]
macro_rules! constraint {
    ([$($left:tt)*] <= $($right:tt)*) => {
        ($($left)*).le($($right)*)
    };
    ([$($left:tt)*] >= $($right:tt)*) => {
        ($($left)*).ge($($right)*)
    };
    // Stop condition: all token have been processed
    ([$($left:tt)*]) => {
        $($left:tt)*
    };
    // The next token is not a special one
    ([$($left:tt)*] $next:tt $($right:tt)*) => {
        constraint!([$($left)* $next] $($right)*)
    };
    // Initial rule: start the recursive calls
    ($($all:tt)*) => {
        constraint!([] $($all)*)
    };
}
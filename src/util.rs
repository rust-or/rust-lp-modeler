pub fn is_zero(n: f32) -> bool {
    n.abs() < 0.00001
}


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
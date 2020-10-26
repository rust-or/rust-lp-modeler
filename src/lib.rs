extern crate uuid;
extern crate proc_macro2;
extern crate quote;
extern crate coin_cbc;

pub mod util;

pub mod dsl {
    pub mod variables;
    pub use self::variables::*;
    pub mod operations;
    pub use self::operations::*;
    pub mod problem;
    pub use self::problem::*;
}

pub mod format {
   pub mod lp_format;
}

pub mod solvers;

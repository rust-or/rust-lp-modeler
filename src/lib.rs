extern crate uuid;
extern crate proc_macro2;
extern crate quote;

#[cfg(feature = "native_coin_cbc")]
extern crate coin_cbc;
#[cfg(feature = "minilp")]
extern crate minilp;

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

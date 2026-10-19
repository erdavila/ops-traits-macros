#![no_std]

//! ...
//!
//! # Macro return
//!
//! The return from macros is handled by the [`__return`] macro.
//!
//! There are two supported methods of return. For the examples, we'll use a macro
//! that generates a bound that requires the passed type argument to implement the `Clone` trait:
//!
//! ```
//! macro_rules! with_clone_bound {
//!     ($type:ty, $($return:tt)+) => {
//!         $crate::__return!([$type: Clone] $($return)+)
//!     };
//! }
//! ```
//!
//! ## Callback
//!
//! The caller includes a macro arm as argument to the called macro. The arm is
//! then "executed", receiving as the patterns the output of the called macro.
//!
//! ```ignore
//! with_clone_bound!(
//!     T,
//!     ($($bounds:tt)*) => {
//!         fn f<T>(x: &T) -> T
//!         where
//!             $($bounds)*
//!         {
//!             x.clone()
//!         }
//!     }
//! );
//! ```
//!
//! This generates the following:
//!
//! ```ignore
//! macro_rules! __emit__ { ... }
//! fn f<T>(x: &T) -> T where T: Clone {
//!     x.clone()
//! };
//! ```
//!
//! Notice that it includes a one-shot macro `__emit__`.
//!
//! ## CPS (Continuation-Passing Style)
//!
//! The caller includes the name of a macro that will receive the return, and any additional arguments.
//!
//! In the example below, the line with `define_function!(@return f)` defines that the macro
//! `define_function` is to be called with the arguments `@return f` followed by the bounds generated
//! by `with_clone_bound`.
//!
//! ```ignore
//! macro_rules! define_function {
//!     ($name:ident) => {
//!         with_clone_bound!(
//!             T,
//!             define_function!(@return $name)
//!         );
//!     };
//!
//!     (@return $name:ident $($bounds:tt)*) => {
//!         fn $name<T>(x: &T) -> T
//!         where
//!             $($bounds)*
//!         {
//!             x.clone()
//!         }
//!     };
//! }
//!
//! define_function!(g);
//! ```
//!
//! This generates the following:
//!
//! ```ignore
//! fn g<T>(x: &T) -> T where T: Clone {
//!     x.clone()
//! };
//! ```

// clauses: https://doc.rust-lang.org/reference/items/generics.html#grammar-WhereClauseItem
#[macro_export]
macro_rules! with_ops_bounds {
    (( $($bounds:tt)* ), $($return:tt)+) => {
        compile_error!("Not implemented")
    };

    ({ $($bounds:tt)* }, $($return:tt)+) => {
        compile_error!("Not implemented")
    };

    ([ $($bounds:tt)* ], $($return:tt)+) => {
        compile_error!("Not implemented")
    };
}

#[macro_export]
macro_rules! ops_type {
    () => {
        compile_error!("Not implemented")
    };
}

#[macro_export]
macro_rules! __return {
    // Callback
    ([$($returned:tt)*] ($($macro_pattern:tt)*) $($macro_rest:tt)*) => {
        macro_rules! __emit__ { ($($macro_pattern)*) $($macro_rest)* }
        __emit__!($($returned)*)
    };

    // CPS
    ([$($returned:tt)*] $macro_name:ident $(::$path:ident)* ! ($($macro_args:tt)*)) => {
        $macro_name $(::$path)* !($($macro_args)* $($returned)*)
    };
}

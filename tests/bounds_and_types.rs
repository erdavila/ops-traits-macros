// Import with different names so that the macro doesn't make use of the imports.
use std::ops::{Neg as Neg_, Sub as Sub_};

use ops_traits_macros::{ops_bounds, ops_type};

#[test]
fn empty() {
    #[ops_bounds]
    fn identity<T>(x: T) -> T {
        x
    }

    let result = identity(7);
    assert_eq!(result, 7);
}

#[test]
fn unop() {
    #[ops_bounds(-T)]
    fn neg<T>(x: T) -> ops_type!(-T) {
        -x
    }

    let result = neg(2);
    assert_eq!(result, -2);
}

#[test]
fn unop_with_output() {
    #[ops_bounds(-T = O)]
    fn neg<T, O>(x: T) -> O {
        -x
    }

    let result = neg(2);
    assert_eq!(result, -2);
}

#[test]
fn unop_with_nested_unop() {
    #[ops_bounds(-(-T))]
    fn denegated<T>(x: T) -> ops_type!(-(-T)) {
        -(-x)
    }

    let result = denegated(3);
    assert_eq!(result, 3);
}

#[test]
fn unop_with_nested_binop() {
    #[ops_bounds(-(T * U))]
    fn calculate<T, U>(x: T, y: U) -> ops_type!(-(T * U)) {
        -(x * y)
    }

    let result = calculate(2, 3);
    assert_eq!(result, -6);
}

#[test]
fn unop_with_nested_output() {
    #[ops_bounds(-(T * U) = (U * T))]
    fn calculate<T, U>(x: T, y: U) -> ops_type!(U * T) {
        -(x * y)
    }

    let result = calculate(2, 3);
    assert_eq!(result, -6);
}

#[test]
fn binop_with_self() {
    #[ops_bounds(T * _)]
    fn square<T>(x: T) -> ops_type!(T * _)
    where
        T: Copy,
    {
        x * x
    }

    let result = square(2);
    assert_eq!(result, 4);
}

#[test]
fn binop_with_self_and_output() {
    #[ops_bounds(T * _ = O)]
    fn square<T, O>(x: T) -> O
    where
        T: Copy,
    {
        x * x
    }

    let result = square(2);
    assert_eq!(result, 4);
}

#[test]
fn binop_with_other() {
    #[ops_bounds(T * U)]
    fn multiply<T, U>(x: T, y: U) -> ops_type!(T * U) {
        x * y
    }

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

#[test]
fn binop_with_other_and_output() {
    #[ops_bounds(T * U = O)]
    fn multiply<T, U, O>(x: T, y: U) -> O {
        x * y
    }

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

#[test]
fn complex_type() {
    #[ops_bounds(<T as Neg_>::Output * <U as Sub_<V>>::Output)]
    fn calculate<T, U, V>(
        x: T,
        y: U,
        z: V,
    ) -> ops_type!(<T as Neg_>::Output * <U as Sub_<V>>::Output)
    where
        T: Neg_,
        U: Sub_<V>,
    {
        -x * (y - z)
    }

    let result = calculate(3, 7, 2);
    assert_eq!(result, -15);
}

#[test]
fn multiple_clauses() {
    #[ops_bounds(T * U = O, O + _)]
    fn calculate<T, U, O>(x: T, y: U) -> ops_type!(O + _)
    where
        O: Copy,
    {
        let z = x * y;
        z + z
    }

    let result = calculate(2, 3);
    assert_eq!(result, 12);
}

#[test]
fn nested_unop_at_left() {
    #[ops_bounds((-T) * U)]
    fn calculate<T, U>(x: T, y: U) -> ops_type!((-T) * U) {
        -x * y
    }

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_unop_at_left_with_output() {
    #[ops_bounds((-T) * U = O)]
    fn calculate<T, U, O>(x: T, y: U) -> O {
        -x * y
    }

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_unop_at_right() {
    #[ops_bounds(T * (-U))]
    fn calculate<T, U>(x: T, y: U) -> ops_type!(T * (-U)) {
        x * -y
    }

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_unop_at_right_with_output() {
    #[ops_bounds(T * (-U) = O)]
    fn calculate<T, U, O>(x: T, y: U) -> O {
        x * -y
    }

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_binop_at_left() {
    #[ops_bounds((T - U) * V)]
    fn calculate<T, U, V>(x: T, y: U, z: V) -> ops_type!((T - U) * V) {
        (x - y) * z
    }

    let result = calculate(5, 3, 4);
    assert_eq!(result, 8);
}

#[test]
fn nested_binop_at_left_with_output() {
    #[ops_bounds((T - U) * V = O)]
    fn calculate<T, U, V, O>(x: T, y: U, z: V) -> O {
        (x - y) * z
    }

    let result = calculate(5, 3, 4);
    assert_eq!(result, 8);
}

#[test]
fn nested_binop_at_right() {
    #[ops_bounds(T - (U * V))]
    fn calculate<T, U, V>(x: T, y: U, z: V) -> ops_type!(T - (U * V)) {
        x - (y * z)
    }

    let result = calculate(7, 3, 2);
    assert_eq!(result, 1);
}

#[test]
fn nested_binop_at_right_with_output() {
    #[ops_bounds(T - (U * V) = O)]
    fn calculate<T, U, V, O>(x: T, y: U, z: V) -> O {
        x - (y * z)
    }

    let result = calculate(7, 3, 2);
    assert_eq!(result, 1);
}

#[test]
fn binop_with_nested_output() {
    #[ops_bounds(T * U = (U * T))]
    fn calculate<T, U>(x: T, y: U) -> ops_type!(U * T) {
        x * y
    }

    let result = calculate(3, 2);
    assert_eq!(result, 6);
}

#[test]
fn parenthesized_non_expr() {
    #[ops_bounds((T) * (U))]
    fn calculate<T, U>(x: T, y: U) -> ops_type!(T * U) {
        x * y
    }

    let result = calculate(3, 2);
    assert_eq!(result, 6);
}

#[test]
fn deeper_level() {
    #[ops_bounds(V - ((T * (-U)) + V) = ((T * (U)) - (-V)))]
    fn calculate<T, U, V>(x: T, y: U, z: V) -> ops_type!((T * U) - (-V))
    where
        V: Copy,
    {
        z - ((x * -y) + z)
    }

    let result = calculate(2, 3, 4);
    assert_eq!(result, 6);
}

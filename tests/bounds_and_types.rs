mod sqr;

// Import with different names so that the macro doesn't make use of the imports.
use std::ops::{Index, Neg as Neg_, Sub as Sub_};

use ops_traits_macros::{ops_bounds, ops_type};

use crate::sqr::Sqr;

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
fn sqrop() {
    #[ops_bounds(T^2)]
    fn sqr<T>(x: T) -> ops_type!(T ^ 2) {
        x.sqr()
    }

    let result = sqr(3);
    assert_eq!(result, 9);
}

#[test]
fn sqrop_with_output() {
    #[ops_bounds(T^2 = O)]
    fn sqr<T, O>(x: T) -> O {
        x.sqr()
    }

    let result = sqr(3);
    assert_eq!(result, 9);
}

#[test]
fn sqrop_with_nested_sqrop() {
    #[ops_bounds((T^2)^2)]
    fn double_squared<T>(x: T) -> ops_type!((T ^ 2) ^ 2) {
        x.sqr().sqr()
    }

    let result = double_squared(2);
    assert_eq!(result, 16);
}

#[test]
fn sqrop_with_nested_unop() {
    #[ops_bounds((-T)^2)]
    fn calculate<T>(x: T) -> ops_type!((-T) ^ 2) {
        (-x).sqr()
    }

    let result = calculate(3);
    assert_eq!(result, 9);
}

#[test]
fn sqrop_with_nested_binop() {
    #[ops_bounds((T * U)^2)]
    fn calculate<T, U>(x: T, y: U) -> ops_type!((T * U) ^ 2) {
        (x * y).sqr()
    }

    let result = calculate(2, 3);
    assert_eq!(result, 36);
}

#[test]
fn sqrop_with_nested_output() {
    #[ops_bounds((T * U)^2 = (U * T))]
    fn calculate<T, U>(x: T, y: U) -> ops_type!(U * T) {
        (x * y).sqr()
    }

    let result = calculate(2, 3);
    assert_eq!(result, 36);
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
fn unop_with_nested_sqrop() {
    #[ops_bounds(-(T^2))]
    fn calculate<T>(x: T) -> ops_type!(-(T ^ 2)) {
        -(x.sqr())
    }

    let result = calculate(3);
    assert_eq!(result, -9);
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
fn parenthesized_non_operation() {
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

#[test]
fn standard_type() {
    fn f<T>(x: &T, index: usize) -> ops_type!(&<T as Index<usize>>::Output)
    where
        T: Index<usize>,
    {
        &x[index]
    }

    let result = f(&[7], 0);
    assert_eq!(result, &7);
}

#[test]
fn standard_bound() {
    #[ops_bounds(
        T * T: Clone + Into<T>,
        T + T,
    )]
    fn calculate<T>(x: T, y: T) -> ops_type!(T + T) {
        let val1 = x * y;
        let val2 = val1.clone();
        val1.into() + val2.into()
    }

    let result = calculate(3, 4);
    assert_eq!(result, 24);
}

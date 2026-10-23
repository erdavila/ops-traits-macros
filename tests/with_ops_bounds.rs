#![recursion_limit = "256"]

// Import with different names so that the macro doesn't make use of the imports.
use std::ops::{Add as Add_, Mul as Mul_, Neg as Neg_, Sub as Sub_};

use ops_traits_macros::with_ops_bounds;

macro_rules! assert_ops_bounds {
    ($ops_bounds:tt, [$($expected:tt)*]) => {{
        with_ops_bounds!(
            $ops_bounds,
            ($($expected)*) => {};
        );
    }};
}

#[test]
fn empty() {
    assert_ops_bounds!([], []);

    with_ops_bounds!(
        { /*empty*/ },
        ($($bounds:tt)*) => {
            fn square<T>(x: T) -> <T as Mul_>::Output
            where
                T: Mul_ + Copy,
                $($bounds)*
            {
                x * x
            }
        };
    );

    let result = square(2);
    assert_eq!(result, 4);
}

#[test]
fn unop() {
    assert_ops_bounds!([
        -T
    ], [
        T: ::core::ops::Neg,
    ]);

    with_ops_bounds!(
        {
            -T,
        },
        ($($bounds:tt)*) => {
            fn neg<T>(x: T) -> <T as Neg_>::Output
            where
                $($bounds)*
            {
                -x
            }
        };
    );

    let result = neg(2);
    assert_eq!(result, -2);
}

#[test]
fn unop_with_output() {
    assert_ops_bounds!([
        -T = O
    ], [
        T: ::core::ops::Neg<Output = O>,
    ]);

    with_ops_bounds!(
        {
            -T = O,
        },
        ($($bounds:tt)*) => {
            fn neg<T, O>(x: T) -> O
            where
                $($bounds)*
            {
                -x
            }
        };
    );

    let result = neg(2);
    assert_eq!(result, -2);
}

#[test]
fn unop_with_nested_unop() {
    with_ops_bounds!(
        {
            -T,
            -(-T),
        },
        ($($bounds:tt)*) => {
            fn denegated<T>(x: T) -> <<T as Neg_>::Output as Neg_>::Output
            where
                $($bounds)*
            {
                -(-x)
            }
        };
    );

    let result = denegated(3);
    assert_eq!(result, 3);
}

#[test]
fn unop_with_nested_binop() {
    with_ops_bounds!(
        {
            T * U,
            -(T * U),
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U>(x: T, y: U) -> <<T as Mul_<U>>::Output as Neg_>::Output
            where
                $($bounds)*
            {
                -(x * y)
            }
        };
    );

    let result = calculate(2, 3);
    assert_eq!(result, -6);
}

#[test]
fn unop_with_nested_output() {
    with_ops_bounds!(
        {
            T * U,
            -(T * U) = (T * U),
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U>(x: T, y: U) -> <T as Mul_<U>>::Output
            where
                $($bounds)*
            {
                -(x * y)
            }
        };
    );

    let result = calculate(2, 3);
    assert_eq!(result, -6);
}

#[test]
fn binop_with_self() {
    assert_ops_bounds!([
        T * _
    ], [
        T: ::core::ops::Mul,
    ]);

    with_ops_bounds!(
        {
            T * _,
        },
        ($($bounds:tt)*) => {
            fn square<T>(x: T) -> <T as Mul_>::Output
            where
                T: Copy,
                $($bounds)*
            {
                x * x
            }
        };
    );

    let result = square(2);
    assert_eq!(result, 4);
}

#[test]
fn binop_with_other() {
    assert_ops_bounds!([
        T * U
    ], [
        T: ::core::ops::Mul<U>,
    ]);

    with_ops_bounds!(
        {
            T * U,
        },
        ($($bounds:tt)*) => {
            fn multiply<T, U>(x: T, y: U) -> <T as Mul_<U>>::Output
            where
                $($bounds)*
            {
                x * y
            }
        };
    );

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

#[test]
fn binop_with_self_and_output() {
    assert_ops_bounds!([
        T * _ = O
    ], [
        T: ::core::ops::Mul<Output = O>,
    ]);

    with_ops_bounds!(
        {
            T * _ = O,
        },
        ($($bounds:tt)*) => {
            fn square<T, O>(x: T) -> O
            where
                T: Copy,
                $($bounds)*
            {
                x * x
            }
        };
    );

    let result = square(2);
    assert_eq!(result, 4);
}

#[test]
fn binop_with_other_and_output() {
    assert_ops_bounds!([
        T * U = O
    ], [
        T: ::core::ops::Mul<U, Output = O>,
    ]);

    with_ops_bounds!(
        {
            T * U = O,
        },
        ($($bounds:tt)*) => {
            fn multiply<T, U, O>(x: T, y: U) -> O
            where
                $($bounds)*
            {
                x * y
            }
        };
    );

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

#[test]
fn regular_bounds() {
    assert_ops_bounds!([
        T: ::core::ops::Mul<U, Output = O>,
    ], [
        T: ::core::ops::Mul<U, Output = O>,
    ]);

    with_ops_bounds!(
        {
            T: Mul_<U, Output = O>,
        },
        ($($bounds:tt)*) => {
            fn multiply<T, U, O>(x: T, y: U) -> O
            where $($bounds)*
            {
                x * y
            }
        };
    );

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

#[test]
fn multiple_clauses() {
    assert_ops_bounds!([
        T * U = O,
        O + _,
        O: Copy,
    ], [
        T: ::core::ops::Mul<U, Output = O>,
        O: ::core::ops::Add,
        O: Copy,
    ]);

    with_ops_bounds!(
        {
            T * U = O,
            O + _,
            O: Copy,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, O>(x: T, y: U) -> <O as Add_>::Output
            where $($bounds)*
            {
                let z = x * y;
                z + z
            }
        };
    );

    let result = calculate(2, 3);
    assert_eq!(result, 12);
}

#[test]
fn nested_unop_at_left() {
    with_ops_bounds!(
        {
            -T,
            (-T) * U,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U>(x: T, y: U) -> <<T as Neg_>::Output as Mul_<U>>::Output
            where
                $($bounds)*
            {
                -x * y
            }
        };
    );

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_unop_at_right() {
    with_ops_bounds!(
        {
            -U,
            T * (-U),
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U>(x: T, y: U) -> <T as Mul_<<U as Neg_>::Output>>::Output
            where
                $($bounds)*
            {
                x * -y
            }
        };
    );

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_unop_at_left_with_output() {
    with_ops_bounds!(
        {
            -T,
            (-T) * U = O,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, O>(x: T, y: U) -> O
            where
                $($bounds)*
            {
                -x * y
            }
        };
    );

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_unop_at_right_with_output() {
    with_ops_bounds!(
        {
            -U,
            T * (-U) = O,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, O>(x: T, y: U) -> O
            where
                $($bounds)*
            {
                x * -y
            }
        };
    );

    let result = calculate(3, 4);
    assert_eq!(result, -12);
}

#[test]
fn nested_binop_at_left() {
    with_ops_bounds!(
        {
            T - U,
            (T - U) * V,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, V>(x: T, y: U, z: V) -> <<T as Sub_<U>>::Output as Mul_<V>>::Output
            where
                $($bounds)*
            {
                (x - y) * z
            }
        };
    );

    let result = calculate(5, 3, 4);
    assert_eq!(result, 8);
}

#[test]
fn nested_binop_at_left_with_output() {
    with_ops_bounds!(
        {
            T - U,
            (T - U) * V = O,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, V, O>(x: T, y: U, z: V) -> O
            where
                $($bounds)*
            {
                (x - y) * z
            }
        };
    );

    let result = calculate(5, 3, 4);
    assert_eq!(result, 8);
}

#[test]
fn nested_binop_at_right() {
    with_ops_bounds!(
        {
            U * V,
            T - (U * V),
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, V>(x: T, y: U, z: V) -> <T as Sub_<<U as Mul_<V>>::Output>>::Output
            where
                $($bounds)*
            {
                x - (y * z)
            }
        };
    );

    let result = calculate(7, 3, 2);
    assert_eq!(result, 1);
}

#[test]
fn nested_binop_at_right_with_output() {
    with_ops_bounds!(
        {
            U * V,
            T - (U * V) = O,
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U, V, O>(x: T, y: U, z: V) -> O
            where
                $($bounds)*
            {
                x - (y * z)
            }
        };
    );

    let result = calculate(7, 3, 2);
    assert_eq!(result, 1);
}

#[test]
fn binop_with_nested_output() {
    with_ops_bounds!(
        {
            U * T,
            T * U = (U * T),
        },
        ($($bounds:tt)*) => {
            fn calculate<T, U>(x: T, y: U) -> <U as Mul_<T>>::Output
            where
                $($bounds)*
            {
                x * y
            }
        };
    );

    let result = calculate(3, 2);
    assert_eq!(result, 6);
}

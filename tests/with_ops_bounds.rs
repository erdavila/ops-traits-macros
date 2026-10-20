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
            fn square<T>(x: T) -> <T as ::core::ops::Mul>::Output
            where
                T: ::core::ops::Mul + Copy,
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
fn op_with_self() {
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
            fn square<T>(x: T) -> <T as ::core::ops::Mul>::Output
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
fn op_with_other() {
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
            fn multiply<T, U>(x: T, y: U) -> <T as ::core::ops::Mul<U>>::Output
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
fn op_with_self_and_output() {
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
fn op_with_other_and_output() {
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
            T: ::core::ops::Mul<U, Output = O>,
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
            fn calculate<T, U, O>(x: T, y: U) -> <O as ::core::ops::Add>::Output
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

use ops_traits_macros::{ops_bounds, ops_type};

#[test]
fn empty() {
    #[ops_bounds]
    fn ident<T>(x: T) -> T {
        x
    }

    let result = ident(7);
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

// #[test]
// fn regular_bounds() {
//     assert_ops_bounds!([
//         T: ::core::ops::Mul<U, Output = O>,
//     ], [
//         T: ::core::ops::Mul<U, Output = O>,
//     ]);

//     with_ops_bounds!(
//         {
//             T: Mul_<U, Output = O>,
//         },
//         ($($bounds:tt)*) => {
//             fn multiply<T, U, O>(x: T, y: U) -> O
//             where $($bounds)*
//             {
//                 x * y
//             }
//         };
//     );

//     let result = multiply(2, 3);
//     assert_eq!(result, 6);
// }

// #[test]
// fn multiple_clauses() {
//     assert_ops_bounds!([
//         T * U = O,
//         O + _,
//         O: Copy,
//     ], [
//         T: ::core::ops::Mul<U, Output = O>,
//         O: ::core::ops::Add,
//         O: Copy,
//     ]);

//     with_ops_bounds!(
//         {
//             T * U = O,
//             O + _,
//             O: Copy,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, O>(x: T, y: U) -> <O as Add_>::Output
//             where $($bounds)*
//             {
//                 let z = x * y;
//                 z + z
//             }
//         };
//     );

//     let result = calculate(2, 3);
//     assert_eq!(result, 12);
// }

// #[test]
// fn nested_unop_at_left() {
//     with_ops_bounds!(
//         {
//             -T,
//             (-T) * U,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U>(x: T, y: U) -> <<T as Neg_>::Output as Mul_<U>>::Output
//             where
//                 $($bounds)*
//             {
//                 -x * y
//             }
//         };
//     );

//     let result = calculate(3, 4);
//     assert_eq!(result, -12);
// }

// #[test]
// fn nested_unop_at_right() {
//     with_ops_bounds!(
//         {
//             -U,
//             T * (-U),
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U>(x: T, y: U) -> <T as Mul_<<U as Neg_>::Output>>::Output
//             where
//                 $($bounds)*
//             {
//                 x * -y
//             }
//         };
//     );

//     let result = calculate(3, 4);
//     assert_eq!(result, -12);
// }

// #[test]
// fn nested_unop_at_left_with_output() {
//     with_ops_bounds!(
//         {
//             -T,
//             (-T) * U = O,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, O>(x: T, y: U) -> O
//             where
//                 $($bounds)*
//             {
//                 -x * y
//             }
//         };
//     );

//     let result = calculate(3, 4);
//     assert_eq!(result, -12);
// }

// #[test]
// fn nested_unop_at_right_with_output() {
//     with_ops_bounds!(
//         {
//             -U,
//             T * (-U) = O,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, O>(x: T, y: U) -> O
//             where
//                 $($bounds)*
//             {
//                 x * -y
//             }
//         };
//     );

//     let result = calculate(3, 4);
//     assert_eq!(result, -12);
// }

// #[test]
// fn nested_binop_at_left() {
//     with_ops_bounds!(
//         {
//             T - U,
//             (T - U) * V,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, V>(x: T, y: U, z: V) -> <<T as Sub_<U>>::Output as
// Mul_<V>>::Output             where
//                 $($bounds)*
//             {
//                 (x - y) * z
//             }
//         };
//     );

//     let result = calculate(5, 3, 4);
//     assert_eq!(result, 8);
// }

// #[test]
// fn nested_binop_at_left_with_output() {
//     with_ops_bounds!(
//         {
//             T - U,
//             (T - U) * V = O,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, V, O>(x: T, y: U, z: V) -> O
//             where
//                 $($bounds)*
//             {
//                 (x - y) * z
//             }
//         };
//     );

//     let result = calculate(5, 3, 4);
//     assert_eq!(result, 8);
// }

// #[test]
// fn nested_binop_at_right() {
//     with_ops_bounds!(
//         {
//             U * V,
//             T - (U * V),
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, V>(x: T, y: U, z: V) -> <T as Sub_<<U as
// Mul_<V>>::Output>>::Output             where
//                 $($bounds)*
//             {
//                 x - (y * z)
//             }
//         };
//     );

//     let result = calculate(7, 3, 2);
//     assert_eq!(result, 1);
// }

// #[test]
// fn nested_binop_at_right_with_output() {
//     with_ops_bounds!(
//         {
//             U * V,
//             T - (U * V) = O,
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U, V, O>(x: T, y: U, z: V) -> O
//             where
//                 $($bounds)*
//             {
//                 x - (y * z)
//             }
//         };
//     );

//     let result = calculate(7, 3, 2);
//     assert_eq!(result, 1);
// }

// #[test]
// fn binop_with_nested_output() {
//     with_ops_bounds!(
//         {
//             U * T,
//             T * U = (U * T),
//         },
//         ($($bounds:tt)*) => {
//             fn calculate<T, U>(x: T, y: U) -> <U as Mul_<T>>::Output
//             where
//                 $($bounds)*
//             {
//                 x * y
//             }
//         };
//     );

//     let result = calculate(3, 2);
//     assert_eq!(result, 6);
// }

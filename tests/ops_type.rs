use ops_traits_macros::ops_type;

#[test]
fn op_with_self() {
    fn square<T>(x: T) -> ops_type!(T * _)
    where
        T: ::core::ops::Mul + Copy,
    {
        x * x
    }

    let result = square(2);
    assert_eq!(result, 4);
}

#[test]
fn op_with_other() {
    fn multiply<T, U>(x: T, y: U) -> ops_type!(T * U)
    where
        T: ::core::ops::Mul<U>,
    {
        x * y
    }

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

#[test]
fn regular_type() {
    fn multiply<T, U>(x: T, y: U) -> ops_type!(<T as ::core::ops::Mul<U>>::Output)
    where
        T: ::core::ops::Mul<U>,
    {
        x * y
    }

    let result = multiply(2, 3);
    assert_eq!(result, 6);
}

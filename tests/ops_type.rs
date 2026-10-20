// Import with different names so that the macro doesn't make use of the imports.
use std::ops::{Mul as Mul_, Sub as Sub_};

mod same_type_eq {
    pub(crate) trait _TypeEq {}

    impl<T> _TypeEq for (T, T) {}

    #[allow(dead_code)]
    pub(crate) fn assert_same_type<T, U>()
    where
        (T, U): _TypeEq,
    {
    }
}

macro_rules! assert_same_type {
    (
        ops_type!( $($op_type:tt)+ ), $rust_type:ty $(,)?
        $(
            {
                $( < $($gen:ident),* > )?
                $( where $($where:tt)* )?
            }
        )?
    ) => {
        {
            fn _f $($( < $($gen),* > )?)? () $($( where $($where)* )?)? {
                use ops_traits_macros::ops_type;
                crate::same_type_eq::assert_same_type::<ops_type!( $($op_type)+ ), $rust_type>();
            }
        }
    };
}

#[test]
fn op_with_self() {
    assert_same_type!(
        ops_type!(T * _),
        <T as Mul_>::Output,
        { <T> where T: Mul_ }
    );
}

#[test]
fn op_with_other() {
    assert_same_type!(
        ops_type!(T * U),
        <T as Mul_<U>>::Output
        { <T, U> where T: Mul_<U> }
    );
}

#[test]
fn nested_operation_at_left() {
    assert_same_type!(
        ops_type!((T - U) * V),
        <<T as Sub_<U>>::Output as Mul_<V>>::Output
        {
            <T, U, V>
            where
                T: Sub_<U>,
                <T as Sub_<U>>::Output: Mul_<V>,
        }
    );
}

#[test]
fn nested_operation_at_right() {
    assert_same_type!(
        ops_type!(T - (U * V)),
        <T as Sub_<<U as Mul_<V>>::Output>>::Output,
        {
            <T, U, V>
            where
                U: Mul_<V>,
                T: Sub_<<U as Mul_<V>>::Output>,
        }
    );
}

#[test]
fn regular_type() {
    assert_same_type!(
        ops_type!(<T as Mul_<U>>::Output),
        <T as Mul_<U>>::Output,
        {
            <T, U>
            where
                T: Mul_<U>,
        }
    );
}

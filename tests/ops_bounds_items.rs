use ops_traits_macros::{ops_bounds, ops_type};

struct A;

struct B;
#[derive(Debug, Clone, Copy)]

struct C;

#[expect(unused)]
struct S;

impl std::ops::Add<B> for A {
    type Output = C;

    fn add(self, _: B) -> Self::Output {
        C
    }
}

#[test]
fn fn_() {
    #[ops_bounds(T + U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T + U) {
        x + y
    }

    assert_eq!(f(3, 4), 7);
}

#[test]
fn enum_() {
    #[ops_bounds(T + U)]
    enum E<T, U> {
        Variant(ops_type!(T + U)),
    }

    let _ = E::Variant::<A, B>(C);
}

#[test]
fn impl_() {
    struct S<T, U>(T, U);

    #[ops_bounds(T + U)]
    impl<T, U> S<T, U> {
        fn add(x: T, y: U) -> ops_type!(T + U) {
            x + y
        }
    }

    assert_eq!(S::add(3, 4), 7);
}

#[test]
fn struct_() {
    #[ops_bounds(T + U)]
    struct S<T, U> {
        #[expect(unused)]
        value: ops_type!(T + U),
    }

    let _ = S::<A, B> { value: C };
}

#[test]
fn trait_() {
    #[ops_bounds(T + U)]
    #[expect(unused)]
    trait Trait<T, U> {
        const VALUE: ops_type!(T + U);
    }

    impl Trait<A, B> for S {
        const VALUE: ops_type!(A + B) = C;
    }
}

#[test]
fn type_() {
    #[ops_bounds(T + U)]
    type Z<T, U> = ops_type!(T + U);

    let _: Z<A, B> = C;
}

#[test]
fn union_() {
    #[ops_bounds(T + U)]
    union S<T, U>
    where
        ops_type!(T + U): Copy,
    {
        #[expect(unused)]
        value: ops_type!(T + U),
    }

    let _ = S::<A, B> { value: C };
}

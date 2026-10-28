use ops_traits_macros::{ops_bounds, ops_type};

#[test]
fn deref() {
    #[ops_bounds(*T)]
    fn f<T>(x: &T) -> &ops_type!(*T) {
        x
    }

    #[ops_bounds(*T = O)]
    fn g<T, O: ?Sized>(x: &T) -> &O {
        x
    }

    assert_eq!(f(&"abc".to_string()), "abc");
    assert_eq!(g(&"abc".to_string()), "abc");
}

#[test]
fn neg() {
    #[ops_bounds(-T)]
    fn f<T>(x: T) -> ops_type!(-T) {
        -x
    }

    #[ops_bounds(-T = O)]
    fn g<T, O>(x: T) -> O {
        -x
    }

    assert_eq!(f(7), -7);
    assert_eq!(g(7), -7);
}

#[test]
fn not() {
    #[ops_bounds(!T)]
    fn f<T>(x: T) -> ops_type!(!T) {
        !x
    }

    #[ops_bounds(!T = O)]
    fn g<T, O>(x: T) -> O {
        !x
    }

    assert!(f(false));
    assert!(g(false));
}

#[test]
fn add() {
    #[ops_bounds(T + U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T + U) {
        x + y
    }

    #[ops_bounds(T + U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x + y
    }

    assert_eq!(f(3, 4), 7);
    assert_eq!(g(3, 4), 7);
}

#[test]
fn sub() {
    #[ops_bounds(T - U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T - U) {
        x - y
    }

    #[ops_bounds(T - U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x - y
    }

    assert_eq!(f(7, 4), 3);
    assert_eq!(g(7, 4), 3);
}

#[test]
fn mul() {
    #[ops_bounds(T * U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T * U) {
        x * y
    }

    #[ops_bounds(T * U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x * y
    }

    assert_eq!(f(2, 3), 6);
    assert_eq!(g(2, 3), 6);
}

#[test]
fn div() {
    #[ops_bounds(T / U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T / U) {
        x / y
    }

    #[ops_bounds(T / U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x / y
    }

    assert_eq!(f(6, 3), 2);
    assert_eq!(g(6, 3), 2);
}

#[test]
fn rem() {
    #[ops_bounds(T % U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T % U) {
        x % y
    }

    #[ops_bounds(T % U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x % y
    }

    assert_eq!(f(8, 3), 2);
    assert_eq!(g(8, 3), 2);
}

#[test]
fn shl() {
    // Note: parentheses is required around the left operand.

    #[ops_bounds((T) << U)]
    fn f<T, U>(x: T, y: U) -> ops_type!((T) << U) {
        x << y
    }

    #[ops_bounds((T) << U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x << y
    }

    assert_eq!(f(0b000_101, 2), 0b010_100);
    assert_eq!(g(0b000_101, 2), 0b010_100);
}

#[test]
fn shr() {
    #[ops_bounds(T >> U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T >> U) {
        x >> y
    }

    #[ops_bounds(T >> U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x >> y
    }

    assert_eq!(f(0b010_100, 2), 0b000_101);
    assert_eq!(g(0b010_100, 2), 0b000_101);
}

#[test]
fn bitand() {
    #[ops_bounds(T & U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T & U) {
        x & y
    }

    #[ops_bounds(T & U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x & y
    }

    assert_eq!(f(0b0111, 0b1110), 0b0110);
    assert_eq!(g(0b0111, 0b1110), 0b0110);
}

#[test]
fn bitxor() {
    #[ops_bounds(T ^ U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T ^ U) {
        x ^ y
    }

    #[ops_bounds(T ^ U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x ^ y
    }

    assert_eq!(f(0b0111, 0b1110), 0b1001);
    assert_eq!(g(0b0111, 0b1110), 0b1001);
}

#[test]
fn bitor() {
    #[ops_bounds(T | U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T | U) {
        x | y
    }

    #[ops_bounds(T | U = O)]
    fn g<T, U, O>(x: T, y: U) -> O {
        x | y
    }

    assert_eq!(f(0b1001, 0b0011), 0b1011);
    assert_eq!(g(0b1001, 0b0011), 0b1011);
}

#[test]
fn add_assign() {
    #[ops_bounds(T += U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x += y;
    }

    let mut n = 3;
    f(&mut n, 4);
    assert_eq!(n, 7);
}

#[test]
fn sub_assign() {
    #[ops_bounds(T -= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x -= y;
    }

    let mut n = 7;
    f(&mut n, 4);
    assert_eq!(n, 3);
}

#[test]
fn mul_assign() {
    #[ops_bounds(T *= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x *= y;
    }

    let mut n = 3;
    f(&mut n, 2);
    assert_eq!(n, 6);
}

#[test]
fn div_assign() {
    #[ops_bounds(T /= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x /= y;
    }

    let mut n = 6;
    f(&mut n, 2);
    assert_eq!(n, 3);
}

#[test]
fn rem_assign() {
    #[ops_bounds(T %= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x %= y;
    }

    let mut n = 8;
    f(&mut n, 3);
    assert_eq!(n, 2);
}

#[test]
fn shl_assign() {
    #[ops_bounds(T <<= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x <<= y;
    }

    let mut n = 0b000_101;
    f(&mut n, 2);
    assert_eq!(n, 0b010_100);
}

#[test]
fn shr_assign() {
    #[ops_bounds(T >>= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x >>= y;
    }

    let mut n = 0b010_100;
    f(&mut n, 2);
    assert_eq!(n, 0b000_101);
}

#[test]
fn bitand_assign() {
    #[ops_bounds(T &= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x &= y;
    }

    let mut n = 0b0111;
    f(&mut n, 0b1110);
    assert_eq!(n, 0b0110);
}

#[test]
fn bitxor_assign() {
    #[ops_bounds(T ^= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x ^= y;
    }

    let mut n = 0b0111;
    f(&mut n, 0b1110);
    assert_eq!(n, 0b1001);
}

#[test]
fn bitor_assign() {
    #[ops_bounds(T |= U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x |= y;
    }

    let mut n = 0b1001;
    f(&mut n, 0b0011);
    assert_eq!(n, 0b1011);
}

#[test]
fn partial_eq() {
    #[ops_bounds(T == U)]
    fn eq<T, U>(x: &T, y: &U) -> [bool; 2] {
        [x == y, x != y]
    }

    #[ops_bounds(T != U)]
    fn ne<T, U>(x: &T, y: &U) -> [bool; 2] {
        [x == y, x != y]
    }

    assert_eq!(eq(&3, &4), [false, true]);
    assert_eq!(ne(&3, &4), [false, true]);
}

#[test]
fn partial_ord() {
    #[ops_bounds(T <=> U)]
    fn cmp<T, U>(x: &T, y: &U) -> [bool; 6] {
        [x < y, x > y, x <= y, x >= y, x == y, x != y]
    }

    // Note: parentheses is required around the left operand.
    #[ops_bounds((T) < U)]
    fn lt<T, U>(x: &T, y: &U) -> [bool; 6] {
        [x < y, x > y, x <= y, x >= y, x == y, x != y]
    }

    #[ops_bounds(T > U)]
    fn gt<T, U>(x: &T, y: &U) -> [bool; 6] {
        [x < y, x > y, x <= y, x >= y, x == y, x != y]
    }

    #[ops_bounds(T <= U)]
    fn le<T, U>(x: &T, y: &U) -> [bool; 6] {
        [x < y, x > y, x <= y, x >= y, x == y, x != y]
    }

    #[ops_bounds(T >= U)]
    fn ge<T, U>(x: &T, y: &U) -> [bool; 6] {
        [x < y, x > y, x <= y, x >= y, x == y, x != y]
    }

    assert_eq!(cmp(&3, &4), [true, false, true, false, false, true]);
    assert_eq!(lt(&3, &4), [true, false, true, false, false, true]);
    assert_eq!(gt(&3, &4), [true, false, true, false, false, true]);
    assert_eq!(le(&3, &4), [true, false, true, false, false, true]);
    assert_eq!(ge(&3, &4), [true, false, true, false, false, true]);
}

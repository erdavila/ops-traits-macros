#![expect(clippy::precedence)]

use ops_traits_macros::{ops_bounds, ops_type};

/*
 *  Precedence levels:
 *      1:  |=  ^=  &=  <<=  >>=  +=  -=  *=  /=  %=
 *      2:  <=>  <  >  <=  >=  ==  !=
 *      3:  |
 *      4:  ^
 *      5:  &
 *      6:  <<  >>
 *      7:  +  -
 *      8:  *  /  %
 *      9:  *  -  !  (all unary)
 *
 *  Notes:
 *    - The macros handle the levels 1 and 2 operators the same way. So only level 1 operator
 *      will be tested, as the result applies to the level 2 ones also.
 *    - The macros accept only ONE operator of levels 1 or 2, so their precedence will not be
 *      tested against themselves.
 *    - Only one operator of each precedence level will be tested, as the results apply to the
 *      other operators of the same level.
 */

#[test]
fn _1_3() {
    #[ops_bounds(T |= U | V)]
    fn f<T, U, V>(x: &mut T, y: U, z: V) {
        *x |= y | z;
    }

    #[ops_bounds(T | U |= V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = x | y;
        result |= z;
        result
    }

    let mut a = 0b0100;
    f(&mut a, 0b0010, 0b0001);
    assert_eq!(a, 0b0111);

    assert_eq!(g(0b0001, 0b0010, 0b0100), 0b0111);
}

#[test]
fn _1_4() {
    #[ops_bounds(T |= U ^ V)]
    fn f<T, U, V>(x: &mut T, y: U, z: V) {
        *x |= y ^ z;
    }

    #[ops_bounds(T ^ U |= V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = x ^ y;
        result |= z;
        result
    }

    let mut a = 0b0100;
    f(&mut a, 0b1010, 0b1001);
    assert_eq!(a, 0b0111);

    assert_eq!(g(0b1001, 0b1010, 0b0100), 0b0111);
}

#[test]
fn _1_5() {
    #[ops_bounds(T |= U & V)]
    fn f<T, U, V>(x: &mut T, y: U, z: V) {
        *x |= y & z;
    }

    #[ops_bounds(T & U |= V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = x & y;
        result |= z;
        result
    }

    let mut a = 0b0100;
    f(&mut a, 0b1010, 0b1001);
    assert_eq!(a, 0b1100);

    assert_eq!(g(0b1001, 0b1010, 0b0100), 0b1100);
}

#[test]
fn _1_6() {
    #[ops_bounds(T |= U >> V)]
    fn f<T, U, V>(x: &mut T, y: U, z: V) {
        *x |= y >> z;
    }

    #[ops_bounds(T >> U |= V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = x >> y;
        result |= z;
        result
    }

    let mut a = 0b0110;
    f(&mut a, 0b1100, 2);
    assert_eq!(a, 0b0111);

    assert_eq!(g(0b1100, 2, 0b0110), 0b0111);
}

#[test]
fn _1_7() {
    #[ops_bounds(T |= U + V)]
    fn f<T, U, V>(x: &mut T, y: U, z: V) {
        *x |= y + z;
    }

    #[ops_bounds(T + U |= V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = x + y;
        result |= z;
        result
    }

    let mut a = 0b0110;
    f(&mut a, 10, 2);
    assert_eq!(a, 0b1110);

    assert_eq!(g(10, 2, 0b0110), 0b1110);
}

#[test]
fn _1_8() {
    #[ops_bounds(T |= U * V)]
    fn f<T, U, V>(x: &mut T, y: U, z: V) {
        *x |= y * z;
    }

    #[ops_bounds(T * U |= V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = x * y;
        result |= z;
        result
    }

    let mut a = 0b0110;
    f(&mut a, 3, 4);
    assert_eq!(a, 0b1110);

    assert_eq!(g(3, 4, 0b0110), 0b1110);
}

#[test]
fn _1_9() {
    #[ops_bounds(T |= !U)]
    fn f<T, U>(x: &mut T, y: U) {
        *x |= !y;
    }

    #[ops_bounds(!T |= U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T) {
        // It's not possible to write a single expression with the given bounds.
        let mut result = !x;
        result |= y;
        result
    }

    let mut a = 0b0110;
    f(&mut a, !0b0011);
    assert_eq!(a, 0b0111);

    assert_eq!(g(!0b0011, 0b0110), 0b0111);
}

#[test]
fn _3_3() {
    #[ops_bounds(T | U | V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U | V) {
        x | y | z
    }

    assert_eq!(f(0b0001, 0b0010, 0b0100), 0b0111);
}

#[test]
fn _3_4() {
    #[ops_bounds(T | U ^ V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U ^ V) {
        x | y ^ z
    }

    #[ops_bounds(T ^ U | V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U | V) {
        x ^ y | z
    }

    assert_eq!(f(0b0001, 0b0010, 0b0100), 0b0111);
    assert_eq!(g(0b0100, 0b0001, 0b0010), 0b0111);
}

#[test]
fn _3_5() {
    #[ops_bounds(T | U & V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U & V) {
        x | y & z
    }

    #[ops_bounds(T & U | V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U | V) {
        x & y | z
    }

    assert_eq!(f(0b1000, 0b1011, 0b0111), 0b1011);
    assert_eq!(g(0b1011, 0b0111, 0b1000), 0b1011);
}

#[test]
fn _3_6() {
    #[ops_bounds(T | U >> V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U >> V) {
        x | y >> z
    }

    #[ops_bounds(T >> U | V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U | V) {
        x >> y | z
    }

    assert_eq!(f(0b1000, 0b1100, 2), 0b1011);
    assert_eq!(g(0b1100, 2, 0b1000), 0b1011);
}

#[test]
fn _3_7() {
    #[ops_bounds(T | U + V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U + V) {
        x | y + z
    }

    #[ops_bounds(T + U | V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U | V) {
        x + y | z
    }

    assert_eq!(f(0b0001, 10, 2), 0b1101);
    assert_eq!(g(10, 2, 0b0001), 0b1101);
}

#[test]
fn _3_8() {
    #[ops_bounds(T | U * V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T | U * V) {
        x | y * z
    }

    #[ops_bounds(T * U | V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U | V) {
        x * y | z
    }

    assert_eq!(f(0b0001, 3, 2), 0b0111);
    assert_eq!(g(3, 2, 0b0001), 0b0111);
}

#[test]
fn _3_9() {
    #[ops_bounds(T | !U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T | !U) {
        x | !y
    }

    #[ops_bounds(!T | U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T | U) {
        !x | y
    }

    assert_eq!(f(0b1000, !0b0011), 0b1011);
    assert_eq!(g(!0b0011, 0b1000), 0b1011);
}

#[test]
fn _4_4() {
    #[ops_bounds(T ^ U ^ V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U ^ V) {
        x ^ y ^ z
    }

    assert_eq!(f(0b0011, 0b0101, 0b1100), 0b1010);
}

#[test]
fn _4_5() {
    #[ops_bounds(T ^ U & V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U & V) {
        x ^ y & z
    }

    #[ops_bounds(T & U ^ V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U ^ V) {
        x & y ^ z
    }

    assert_eq!(f(0b0001, 0b1110, 0b1100), 0b1101);
    assert_eq!(g(0b1110, 0b1100, 0b0001), 0b1101);
}

#[test]
fn _4_6() {
    #[ops_bounds(T ^ U >> V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U >> V) {
        x ^ y >> z
    }

    #[ops_bounds(T >> U ^ V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U ^ V) {
        x >> y ^ z
    }

    assert_eq!(f(0b0101, 0b1100, 2), 0b0110);
    assert_eq!(g(0b1100, 2, 0b0101), 0b0110);
}

#[test]
fn _4_7() {
    #[ops_bounds(T ^ U + V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U + V) {
        x ^ y + z
    }

    #[ops_bounds(T + U ^ V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U ^ V) {
        x + y ^ z
    }

    assert_eq!(f(0b1010, 10, 2), 0b0110);
    assert_eq!(g(10, 2, 0b1010), 0b0110);
}

#[test]
fn _4_8() {
    #[ops_bounds(T ^ U * V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T ^ U * V) {
        x ^ y * z
    }

    #[ops_bounds(T * U ^ V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U ^ V) {
        x * y ^ z
    }

    assert_eq!(f(0b1010, 3, 4), 0b0110);
    assert_eq!(g(3, 4, 0b1010), 0b0110);
}

#[test]
fn _4_9() {
    #[ops_bounds(T ^ !U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T ^ !U) {
        x ^ !y
    }

    #[ops_bounds(!T ^ U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T ^ U) {
        !x ^ y
    }

    assert_eq!(f(0b0110, !0b0011), 0b0101);
    assert_eq!(g(!0b0011, 0b0110), 0b0101);
}

#[test]
fn _5_5() {
    #[ops_bounds(T & U & V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U & V) {
        x & y & z
    }

    assert_eq!(f(0b0111, 0b1011, 0b0010), 0b0010);
}

#[test]
fn _5_6() {
    #[ops_bounds(T & U >> V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U >> V) {
        x & y >> z
    }

    #[ops_bounds(T >> U & V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U & V) {
        x >> y & z
    }

    assert_eq!(f(0b1001, 0b1100, 2), 0b0001);
    assert_eq!(g(0b1100, 2, 0b1001), 0b0001);
}

#[test]
fn _5_7() {
    #[ops_bounds(T & U + V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U + V) {
        x & y + z
    }

    #[ops_bounds(T + U & V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U & V) {
        x + y & z
    }

    assert_eq!(f(0b1001, 10, 2), 0b1000);
    assert_eq!(g(10, 2, 0b1001), 0b1000);
}

#[test]
fn _5_8() {
    #[ops_bounds(T & U * V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T & U * V) {
        x & y * z
    }

    #[ops_bounds(T * U & V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U & V) {
        x * y & z
    }

    assert_eq!(f(0b1001, 3, 4), 0b1000);
    assert_eq!(g(3, 4, 0b1001), 0b1000);
}

#[test]
fn _5_9() {
    #[ops_bounds(T & !U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T & !U) {
        x & !y
    }

    #[ops_bounds(!T & U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T & U) {
        !x & y
    }

    assert_eq!(f(0b0110, !0b0011), 0b0010);
    assert_eq!(g(!0b0011, 0b0110), 0b0010);
}

#[test]
fn _6_6() {
    #[ops_bounds(T >> U >> V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U >> V) {
        x >> y >> z
    }

    assert_eq!(f(0b1100, 1, 1), 0b0011);
}

#[test]
fn _6_7() {
    #[ops_bounds(T >> U + V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U + V) {
        x >> y + z
    }

    #[ops_bounds(T + U >> V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U >> V) {
        x + y >> z
    }

    assert_eq!(f(0b1000, 2, 1), 0b0001);
    assert_eq!(g(10, 2, 2), 0b0011);
}

#[test]
fn _6_8() {
    #[ops_bounds(T >> U * V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T >> U * V) {
        x >> y * z
    }

    #[ops_bounds(T * U >> V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U >> V) {
        x * y >> z
    }

    assert_eq!(f(0b1100, 2, 1), 0b0011);
    assert_eq!(g(3, 4, 2), 0b0011);
}

#[test]
fn _6_9() {
    #[ops_bounds(T >> !U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T >> !U) {
        x >> !y
    }

    #[ops_bounds(!T >> U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T >> U) {
        !x >> y
    }

    assert_eq!(f(0b1000, !0b0011), 0b0001);
    assert_eq!(g(!0b1100, 2), 0b0011);
}

fn _7_7() {
    #[ops_bounds(T + U + V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U + V) {
        x + y + z
    }

    assert_eq!(f(10, 2, 1), 13);
}

#[test]
fn _7_8() {
    #[ops_bounds(T + U * V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T + U * V) {
        x + y * z
    }

    #[ops_bounds(T * U + V)]
    fn g<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U + V) {
        x * y + z
    }

    assert_eq!(f(1, 3, 4), 13);
    assert_eq!(g(3, 4, 1), 13);
}

#[test]
fn _7_9() {
    #[ops_bounds(T + !U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T + !U) {
        x + !y
    }

    #[ops_bounds(!T + U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T + U) {
        !x + y
    }

    assert_eq!(f(9, !0b0011), 12);
    assert_eq!(g(!0b0011, 9), 12);
}

fn _8_8() {
    #[ops_bounds(T * U * V)]
    fn f<T, U, V>(x: T, y: U, z: V) -> ops_type!(T * U * V) {
        x * y * z
    }

    assert_eq!(f(1, 7, 2), 14);
}

#[test]
fn _8_9() {
    #[ops_bounds(T * !U)]
    fn f<T, U>(x: T, y: U) -> ops_type!(T * !U) {
        x * !y
    }

    #[ops_bounds(!T * U)]
    fn g<T, U>(x: T, y: U) -> ops_type!(!T * U) {
        !x * y
    }

    assert_eq!(f(4, !0b0011), 12);
    assert_eq!(g(!0b0011, 4), 12);
}

fn _9_9() {
    #[ops_bounds(!!T)]
    fn f<T>(x: T) -> ops_type!(!!T) {
        !!x
    }

    assert_eq!(f(0b0011), 0b0011);
}

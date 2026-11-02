Macros to simplify expressing `std::ops` trait bounds and types.

`std::ops` trait bounds and types can be expressed in a form similar to mathematical
expressions. Both supplied macros support the same expressions, with `ops_bounds` supporting additional
features.

For instance, the code:

```rust
use ops_traits_macros::{ops_bounds, ops_type};

#[ops_bounds(A * B + C / D)]
fn calculate<A, B, C, D>(a: A, b: B, c: C, d: D) -> ops_type!(A * B + C / D) {
    a * b + c / d
}
```

is equivalent to this:

```rust
use std::ops::{Add, Div, Mul};

fn calculate<A, B, C, D>(
    a: A,
    b: B,
    c: C,
    d: D,
) -> <<A as Mul<B>>::Output as Add<<C as Div<D>>::Output>>::Output
where
    A: Mul<B>,
    C: Div<D>,
    <A as Mul<B>>::Output: Add<<C as Div<D>>::Output>,
{
    a * b + c / d
}
```

The parsing implemented by the macros respects the same [precedence rules] as the Rust compiler,
and parentheses can be used to group operations.

Read the documentation for details.

[precedence rules]: https://doc.rust-lang.org/reference/expressions.html#expression-precedence

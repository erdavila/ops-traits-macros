//! Macros to simplify expressing [`std::ops`] trait bounds and types.
//!
//! [`std::ops`] trait bounds and types can be expressed in a form similar to mathematical
//! expressions. Both macros support the same expressions, with [`ops_bounds`] supporting additional
//! features.
//!
//! For instance, the code:
//!
//! ```
//! use ops_traits_macros::{ops_bounds, ops_type};
//!
//! #[ops_bounds(A * B + C / D)]
//! fn calculate<A, B, C, D>(a: A, b: B, c: C, d: D) -> ops_type!(A * B + C / D) {
//!     a * b + c / d
//! }
//! ```
//!
//! is equivalent to this:
//!
//! ```
//! use std::ops::{Add, Div, Mul};
//!
//! fn calculate<A, B, C, D>(
//!     a: A,
//!     b: B,
//!     c: C,
//!     d: D,
//! ) -> <<A as Mul<B>>::Output as Add<<C as Div<D>>::Output>>::Output
//! where
//!     A: Mul<B>,
//!     C: Div<D>,
//!     <A as Mul<B>>::Output: Add<<C as Div<D>>::Output>,
//! {
//!     a * b + c / d
//! }
//! ```
//!
//! The parsing implemented by the macros respects the same [precedence rules] as the Rust compiler,
//! and parentheses can be used to group operations.
//!
//! [precedence rules]: https://doc.rust-lang.org/reference/expressions.html#expression-precedence
//!
//! The information below applies to both [`ops_bounds`] and [`ops_type!`] macros. Consult the
//! [`ops_bounds`] macro attribute for additional features.
//!
//! # Supported operators
//!
//! | Operator             | Trait                | Precedence  |
//! |----------------------|----------------------|-------------|
//! | `\|`                 | [`std::ops::BitOr`]  | 1 (lowest)  |
//! | `^`                  | [`std::ops::BitXor`] | 2           |
//! | `&`                  | [`std::ops::BitAnd`] | 3           |
//! | `<<`(see note below) | [`std::ops::Shl`]    | 4           |
//! | `>>`                 | [`std::ops::Shr`]    | 4           |
//! | `+`                  | [`std::ops::Add`]    | 5           |
//! | `-`                  | [`std::ops::Sub`]    | 5           |
//! | `*`                  | [`std::ops::Mul`]    | 6           |
//! | `/`                  | [`std::ops::Div`]    | 6           |
//! | `%`                  | [`std::ops::Rem`]    | 6           |
//! | `!` (unary)          | [`std::ops::Not`]    | 7 (highest) |
//! | `-` (unary)          | [`std::ops::Neg`]    | 7 (highest) |
//! | `*` (unary)          | [`std::ops::Deref`]  | 7 (highest) |
//!
//! **Note:** Due to type parsing restrictions (where `<` may be parsed as opening generic
//! arguments), when the shift left operator is used, the left operand *must* be surrounded by
//! parentheses:
//!
//! ```
//! use ops_traits_macros::{ops_bounds, ops_type};
//!
//! #[ops_bounds((T) << U)]
//! fn shift_left<T, U>(x: T, y: U) -> ops_type!((T) << U) {
//!     x << y
//! }
//! ```
//!
//! # Accepted types
//!
//! The macros accept not only generic type arguments, but _any_ Rust type.
//!
//! ```
//! use std::ops::Index;
//!
//! use ops_traits_macros::{ops_bounds, ops_type};
//!
//! #[ops_bounds(<T as Index<usize>>::Output * U)]
//! fn multiply_indexed_value<T, U>(
//!     x: T,
//!     index: usize,
//!     y: U,
//! ) -> ops_type!(<T as Index<usize>>::Output * U)
//! where
//!     T: Index<usize>,
//!     <T as Index<usize>>::Output: Copy,
//! {
//!     x[index] * y
//! }
//! ```
//!
//! # `Self` as right-hand-side operand
//!
//! To express that a binary operation can be applied to operands of the same type (using the
//! trait's default `Rhs = Self`), `_` can be used as the right operand.
//!
//! For instance:
//!
//! ```
//! use ops_traits_macros::{ops_bounds, ops_type};
//!
//! #[ops_bounds(T * _)]
//! fn square<T>(x: T) -> ops_type!(T * _)
//! where
//!     T: Copy,
//! {
//!     x * x
//! }
//! ```
//!
//! is equivalent to:
//!
//! ```
//! use std::ops::Mul;
//!
//! fn square<T>(x: T) -> <T as Mul>::Output
//! where
//!     T: Mul + Copy,
//! {
//!     x * x
//! }
//! ```
//!
//! # Example
//!
//! A very generic implementation of the division operation for complex numbers:
//!
//! ```
//! use std::ops::Div;
//!
//! use ops_traits_macros::{ops_bounds, ops_type};
//!
//! struct Complex<R, I> {
//!     real: R,
//!     imag: I,
//! }
//!
//! #[ops_bounds(
//!     C * _ + D * _ = Den,
//!     (A * C + B * D) / Den,
//!     (B * C - A * D) / Den,
//! )]
//! impl<A, B, C, D, Den> Div<Complex<C, D>> for Complex<A, B>
//! where
//!     A: Copy,
//!     B: Copy,
//!     C: Copy,
//!     D: Copy,
//!     Den: Copy,
//! {
//!     type Output = Complex<ops_type!((A * C + B * D) / Den), ops_type!((B * C - A * D) / Den)>;
//!
//!     fn div(self, rhs: Complex<C, D>) -> Self::Output {
//!         let Complex { real: a, imag: b } = self;
//!         let Complex { real: c, imag: d } = rhs;
//!         /*
//!            self   a+bi   (a+bi)(c-di)   ac - adi + bci + bd   ac + bd   bc - ad
//!            ---- = ---- = ------------ = ------------------- = ------- + ------- i
//!            rhs    c+di   (c+di)(c-di)         c² + d²         c² + d²   c² + d²
//!         */
//!         let denominator = c * c + d * d;
//!         let real = (a * c + b * d) / denominator;
//!         let imag = (b * c - a * d) / denominator;
//!
//!         Complex { real, imag }
//!     }
//! }
//! ```
//!
//! <details>
//!   <summary>The equivalent implementation without the macros.</summary>
//!
//! ```
//! use std::ops::{Add, Div, Mul, Sub};
//!
//! struct Complex<R, I> {
//!     real: R,
//!     imag: I,
//! }
//!
//! impl<A, B, C, D, Den> Div<Complex<C, D>> for Complex<A, B>
//! where
//!     A: Mul<C> + Mul<D> + Copy,
//!     B: Mul<C> + Mul<D> + Copy,
//!     C: Mul + Copy,
//!     D: Mul + Copy,
//!     Den: Copy,
//!     <C as Mul>::Output: Add<<D as Mul>::Output, Output = Den>,
//!     <A as Mul<C>>::Output: Add<<B as Mul<D>>::Output>,
//!     <B as Mul<C>>::Output: Sub<<A as Mul<D>>::Output>,
//!     <<A as Mul<C>>::Output as Add<<B as Mul<D>>::Output>>::Output: Div<Den>,
//!     <<B as Mul<C>>::Output as Sub<<A as Mul<D>>::Output>>::Output: Div<Den>,
//! {
//!     type Output = Complex<
//!         <<<A as Mul<C>>::Output as Add<<B as Mul<D>>::Output>>::Output as Div<Den>>::Output,
//!         <<<B as Mul<C>>::Output as Sub<<A as Mul<D>>::Output>>::Output as Div<Den>>::Output,
//!     >;
//!
//!     fn div(self, rhs: Complex<C, D>) -> Self::Output {
//!         let Complex { real: a, imag: b } = self;
//!         let Complex { real: c, imag: d } = rhs;
//!         /*
//!            self   a+bi   (a+bi)(c-di)   ac - adi + bci + bd   ac + bd   bc - ad
//!            ---- = ---- = ------------ = ------------------- = ------- + ------- i
//!            rhs    c+di   (c+di)(c-di)         c² + d²         c² + d²   c² + d²
//!         */
//!         let denominator = c * c + d * d;
//!         let real = (a * c + b * d) / denominator;
//!         let imag = (b * c - a * d) / denominator;
//!
//!         Complex { real, imag }
//!     }
//! }
//! ```
//!
//! </details>

mod grammar;
mod operator;
mod output;
mod parse;

use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use quote::ToTokens as _;
use syn::parse::Parser as _;
use syn::punctuated::Punctuated;
use syn::{Error, Item, Result, Token, WherePredicate, parse_macro_input};

use crate::grammar::{NoAssocTyOpOpsBound, OpsBound, OpsExpr};
use crate::operator::{CombineBinOp, NoAssocTyBinOp, Op, UnOp};
use crate::output::{qualified_type, where_predicate, where_predicate_with_assoc};
use crate::parse::TryParse;

/// A macro attribute that allows expressing [`std::ops`] trait bounds in a mathematical expression
/// format.
///
/// The macro can be used on any item that accepts a `where` clause: `enum`, `fn`, `impl`, `struct`,
/// `trait`, `type`, `union`.
///
/// See the [crate] docs for basic information and examples that apply also to the [`ops_type!`]
/// macro. See below for additional features supported by the [`ops_bounds`] attribute.
///
/// The input to the attribute is a comma-separated list of math-like expressions. For each
/// expression, the macro generates all the `where`-clause predicates required for the operations in
/// the expression.
///
/// For instance, the following bounds are generated from the expression `A * B + C / D = O`:
///
/// ```
/// # use ops_traits_macros::ops_type;
/// # use std::ops::{Add, Div, Mul};
/// #
/// # type T<A, B, C, D, O>
/// where
///     // A can be multiplied by B.
///     A: Mul<B>,
///     // C can be divided by D.
///     C: Div<D>,
///     // A * B can be added to C / D and the result is of type O.
///     <A as Mul<B>>::Output: Add<<C as Div<D>>::Output, Output = O>
/// # = (ops_type!(A * B + C / D), O);
/// ```
///
/// # Associated type predicate
///
/// By appending `=` and then a type or a second expression, the bound includes an associated type
/// predicate that constrains the output of the first expression to match the type of the second.
///
/// ```
/// use ops_traits_macros::{ops_bounds, ops_type};
///
/// // Requires that multiplying A by B results in the same type as multiplying B by A.
/// #[ops_bounds(A * B = B * A)]
/// fn multiply<A, B>(a: A, b: B) -> ops_type!(B * A) {
///     a * b
/// }
/// ```
///
/// produces:
///
/// ```
/// use std::ops::Mul;
///
/// // Requires that multiplying A by B results in the same type as multiplying B by A.
/// fn multiply<A, B>(a: A, b: B) -> <B as Mul<A>>::Output
/// where
///     A: Mul<B, Output = <B as Mul<A>>::Output>,
///     B: Mul<A>,
/// {
///     a * b
/// }
/// ```
///
/// # Comparison and assignment operators
///
/// In addition to the [arithmetic, bit and dereference operators](crate#supported-operators),
/// `ops_bounds` also supports assignment and comparison operators.
///
/// | Operators                                   | Trait                      |
/// |---------------------------------------------|----------------------------|
/// | `\|=`                                       | [`std::ops::BitOrAssign`]  |
/// | `^=`                                        | [`std::ops::BitXorAssign`] |
/// | `&=`                                        | [`std::ops::BitAndAssign`] |
/// | `<<=`                                       | [`std::ops::ShlAssign`]    |
/// | `>>=`                                       | [`std::ops::ShrAssign`]    |
/// | `+=`                                        | [`std::ops::AddAssign`]    |
/// | `-=`                                        | [`std::ops::SubAssign`]    |
/// | `*=`                                        | [`std::ops::MulAssign`]    |
/// | `/=`                                        | [`std::ops::DivAssign`]    |
/// | `%=`                                        | [`std::ops::RemAssign`]    |
/// | `==`, `!=`                                  | [`std::cmp::PartialEq`]    |
/// | `<`(see note below), `>`, `<=`, `>=`, `<=>` | [`std::cmp::PartialOrd`]   |
///
/// When either the `==` or `!=` operator is used, it applies to both operators because they share
/// the same trait bound ([`std::cmp::PartialEq`]). Therefore, it is not necessary to include bounds
/// for both operators with the same operand types.
///
/// The same applies to the remaining comparison operators: specifying any one of them is sufficient
/// to include a [`std::cmp::PartialOrd`] bound. A dedicated `<=>` operator is also available to
/// represent a general comparison bound.
///
/// All of the additional operators have lower precedence than [the other
/// operators](crate#supported-operators), and each expression can contain at most one of them.
///
/// ```
/// use ops_traits_macros::ops_bounds;
///
/// #[ops_bounds(T <=> _)]
/// fn max<T>(a: T, b: T, c: T) -> T {
///     if a > b {
///         if a > c { a } else { c }
///     } else {
///         if b > c { b } else { c }
///     }
/// }
/// ```
///
/// **Note:** Due to type parsing restrictions (where `<` may be parsed as opening generic
/// arguments), when the less-than operator is used, the left operand *must* be surrounded by
/// parentheses:
///
/// ```
/// use ops_traits_macros::ops_bounds;
///
/// #[ops_bounds((T) < U)]
/// fn shift_left<T, U>(x: T, y: U) -> bool {
///     x < y
/// }
/// ```
#[proc_macro_attribute]
pub fn ops_bounds(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ops_bounds = {
        let parser = Punctuated::<OpsBound, Token![,]>::parse_terminated;
        parser.parse(attr)
    };

    let mut item = parse_macro_input!(item as Item);
    let item_generics = match &mut item {
        Item::Enum(item_enum) => Ok(&mut item_enum.generics),
        Item::Fn(item_fn) => Ok(&mut item_fn.sig.generics),
        Item::Impl(item_impl) => Ok(&mut item_impl.generics),
        Item::Struct(item_struct) => Ok(&mut item_struct.generics),
        Item::Trait(item_trait) => Ok(&mut item_trait.generics),
        Item::Type(item_type) => Ok(&mut item_type.generics),
        Item::Union(item_union) => Ok(&mut item_union.generics),
        item => Err(Error::new_spanned(
            item,
            "the ops_bounds macro is only supported in the following items: enum, fn, impl, struct, trait, type, union",
        )),
    };

    zip_combine_results(item_generics, ops_bounds)
        .map(|(gens, bounds)| {
            let predicates = ops_bounds_to_where_predicates(bounds);
            gens.make_where_clause().predicates.extend(predicates);
        })
        .map_or_else(Error::into_compile_error, |()| item.into_token_stream())
        .into()
}

fn ops_bounds_to_where_predicates(
    bounds: impl IntoIterator<Item = OpsBound>,
) -> impl Iterator<Item = WherePredicate> {
    let mut unique_assoc_ty_op_bounds = HashMap::new();
    let mut unique_no_assoc_ty_op_bounds = HashSet::new();

    for bound in bounds {
        for sub_oper in bound.sub_opers() {
            unique_assoc_ty_op_bounds
                .entry(sub_oper)
                .or_insert(HashSet::new());
        }

        match bound {
            OpsBound::AssocTyOp(bound) => {
                let assocs = unique_assoc_ty_op_bounds
                    .entry(bound.oper)
                    .or_insert(HashSet::new());

                if let Some(assoc) = bound.assoc {
                    assocs.insert(assoc);
                }
            }
            OpsBound::NoAssocTyOp(bound) => {
                unique_no_assoc_ty_op_bounds.insert(bound);
            }
        }
    }

    let assoc_ty_op_predicates =
        unique_assoc_ty_op_bounds
            .into_iter()
            .flat_map(|(oper, assocs)| {
                // Ensure there is at least one bound.
                let no_assocs = assocs
                    .is_empty()
                    .then(|| oper.clone().into_where_predicate())
                    .into_iter();

                let with_assocs = assocs
                    .into_iter()
                    .map(move |assoc| oper.clone().into_where_predicate_with_assoc(assoc));

                no_assocs.chain(with_assocs)
            });

    let no_assoc_ty_op_predicates = unique_no_assoc_ty_op_bounds
        .into_iter()
        .map(NoAssocTyOpOpsBound::into_where_predicate);

    assoc_ty_op_predicates.chain(no_assoc_ty_op_predicates)
}

/// Allows expressing the output types of [`std::ops`] operations in a mathematical expression
/// format.
///
/// The macro can be used in _any_ position where a type is accepted.
///
/// See the [crate] docs for basic information and examples.
#[proc_macro]
pub fn ops_type(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as OpsExpr)
        .into_type()
        .into_token_stream()
        .into()
}

fn zip_combine_results<T, U>(r1: Result<T>, r2: Result<U>) -> Result<(T, U)> {
    match (r1, r2) {
        (Ok(x), Ok(y)) => Ok((x, y)),
        (Ok(_), Err(e2)) => Err(e2),
        (Err(e1), Ok(_)) => Err(e1),
        (Err(mut e1), Err(e2)) => {
            e1.combine(e2);
            Err(e1)
        }
    }
}

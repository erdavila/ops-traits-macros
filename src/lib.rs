use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{
    Error, GenericArgument, Ident, Item, Path, PathArguments, Result, Token, Type, TypeParamBound,
    WherePredicate, parenthesized, parse_macro_input, parse_quote, parse_quote_spanned,
};

/*
    OpsBound := Operation AssociatedType?
    OpsExpr = Operation | Type

    Operation := UnOp Term | Term BinOp RightOperand

    RightOperand := '_' | Term
    AssociatedType := '=' Term
    Term := Type | '(' OpsExpr ')'

    ------------------------------------------------------------

    CLAUSE -> OpsBound
    OPERATION -> OpsExpr

    OPERAND / ASSOC / LEFT / TYPE -> Term
    RAW-TYPE -> Type
*/

#[proc_macro_attribute]
pub fn ops_bounds(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ops_bounds = {
        let parser = Punctuated::<OpsBound, Token![,]>::parse_terminated;
        parser.parse(attr)
    };

    let mut item = parse_macro_input!(item as Item);
    let where_clause = {
        match &mut item {
            // Item::Enum(item_enum) => todo!(),
            Item::Fn(item_fn) => Ok(item_fn.sig.generics.make_where_clause()),
            // Item::Impl(item_impl) => todo!(),
            // Item::Struct(item_struct) => todo!(),
            // Item::Trait(item_trait) => todo!(),
            // Item::Type(item_type) => todo!(),
            // Item::Union(item_union) => todo!(),
            item => Err(Error::new_spanned(
                item,
                "the ops_bounds macro is only supported in functions",
            )),
        }
    };

    where_clause
        .zip_combine(ops_bounds)
        .map(|(wc, bounds)| {
            // TODO: Consider using HashMap<Operation, HashSet<AssociatedType>>.

            let unique_bounds: HashSet<OpsBound> =
                bounds.into_iter().flat_map(OpsBound::all_bounds).collect();

            let predicates = unique_bounds
                .into_iter()
                .map(OpsBound::into_where_predicate);

            wc.predicates.extend(predicates);
        })
        .map_or_else(Error::into_compile_error, |()| item.into_token_stream())
        .into()
}

#[proc_macro]
pub fn ops_type(item: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(item as OpsExpr);

    expr.into_type().into_token_stream().into()
}

// TODO: consider using Rc instead of Box.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OpsBound {
    operation: Operation,
    assoc: AssociatedType,
}
impl OpsBound {
    fn all_bounds(self) -> Vec<OpsBound> {
        let mut bounds = Vec::from([self.clone()]);
        bounds.extend(self.operation.sub_bounds());
        bounds.extend(self.assoc.all_bounds());
        bounds
    }

    fn into_where_predicate(self) -> WherePredicate {
        let (bounded_term, bound) = match self.operation {
            Operation::Unary { unop, operand } => (operand, unop.type_param_bound(self.assoc)),
            Operation::Binary { left, binop, right } => {
                (left, binop.type_param_bound(right, self.assoc))
            }
        };

        let bounded_ty = bounded_term.into_type();
        parse_quote! { #bounded_ty : #bound }
    }
}
impl Parse for OpsBound {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(OpsBound {
            operation: input.parse()?,
            assoc: input.parse()?,
        })
    }
}
impl From<Operation> for OpsBound {
    fn from(operation: Operation) -> Self {
        OpsBound {
            operation,
            assoc: AssociatedType::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OpsExpr {
    Operation(Operation),
    Type(Box<Type>),
}
impl OpsExpr {
    fn all_bounds(&self) -> Vec<OpsBound> {
        match self {
            OpsExpr::Operation(operation) => operation.all_bounds(),
            OpsExpr::Type(_) => Vec::new(),
        }
    }

    fn into_type(self) -> Type {
        match self {
            OpsExpr::Operation(operation) => operation.into_type(),
            OpsExpr::Type(_) => todo!("OpsExpr::into_type - Type"),
        }
    }
}
impl Parse for OpsExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Some(unop) = UnOp::try_parse(input) {
            Ok(OpsExpr::Operation(Operation::Unary {
                unop,
                operand: input.parse()?,
            }))
        } else if input.peek(Paren) {
            todo!("Parse for OpsExpr: paren")
        } else {
            let ty = input.parse()?;
            if let Some(binop) = BinOp::try_parse(input) {
                Ok(OpsExpr::Operation(Operation::Binary {
                    left: Box::new(Term::Type(ty)),
                    binop,
                    right: input.parse()?,
                }))
            } else {
                Ok(OpsExpr::Type(ty))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Operation {
    Unary {
        unop: UnOp,
        operand: Box<Term>,
    },
    Binary {
        left: Box<Term>,
        binop: BinOp,
        right: RightOperand,
    },
}
impl Operation {
    fn all_bounds(&self) -> Vec<OpsBound> {
        let mut bounds = self.sub_bounds();
        bounds.push(self.clone().into());
        bounds
    }

    fn sub_bounds(&self) -> Vec<OpsBound> {
        match self {
            Operation::Unary { unop: _, operand } => operand.all_bounds(),
            Operation::Binary {
                left,
                binop: _,
                right,
            } => {
                let mut bounds = left.all_bounds();
                bounds.extend(right.all_bounds());
                bounds
            }
        }
    }

    fn into_type(self) -> Type {
        match self {
            Operation::Unary { unop, operand } => unop.qualified_type(operand.into_type()),
            Operation::Binary { left, binop, right } => {
                binop.qualified_type(left.into_type(), right)
            }
        }
    }
}
impl Parse for Operation {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Some(unop) = UnOp::try_parse(input) {
            Ok(Operation::Unary {
                unop,
                operand: input.parse()?,
            })
        } else {
            Ok(Operation::Binary {
                left: input.parse()?,
                binop: input.parse()?,
                right: input.parse()?,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RightOperand {
    SelfRhs(Token![_]),
    Term(Box<Term>),
}
impl RightOperand {
    fn all_bounds(&self) -> Vec<OpsBound> {
        match self {
            RightOperand::SelfRhs(_) => Vec::new(),
            RightOperand::Term(term) => term.all_bounds(),
        }
    }

    fn into_generic_argument(self) -> Option<GenericArgument> {
        match self {
            RightOperand::SelfRhs(_) => None,
            RightOperand::Term(term) => {
                let arg = term.into_type();
                Some(parse_quote! { #arg })
            }
        }
    }
}
impl Parse for RightOperand {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![_]) {
            Ok(RightOperand::SelfRhs(input.parse()?))
        } else {
            Ok(RightOperand::Term(input.parse()?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AssociatedType {
    None,
    Some { eq_token: Token![=], term: Term },
}
impl AssociatedType {
    fn all_bounds(self) -> Vec<OpsBound> {
        match self {
            AssociatedType::None => Vec::new(),
            AssociatedType::Some { eq_token: _, term } => term.all_bounds(),
        }
    }

    fn into_generic_argument(self, op: &impl Op) -> Option<GenericArgument> {
        match self {
            AssociatedType::None => None,
            AssociatedType::Some { eq_token, term } => {
                let assoc_type_name = op.assoc_type_name();
                let assoc = term.into_type();
                Some(parse_quote! { #assoc_type_name #eq_token #assoc })
            }
        }
    }
}
impl Parse for AssociatedType {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![=]) {
            Ok(AssociatedType::Some {
                eq_token: input.parse()?,
                term: input.parse()?,
            })
        } else {
            Ok(AssociatedType::None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Term {
    Type(Box<Type>),
    Group { paren: Paren, expr: Box<OpsExpr> },
}
impl Term {
    fn all_bounds(&self) -> Vec<OpsBound> {
        match self {
            Term::Type(_) => Vec::new(),
            Term::Group { paren: _, expr } => expr.all_bounds(),
        }
    }

    fn into_type(self) -> Type {
        match self {
            Term::Type(ty) => *ty,
            Term::Group { paren: _, expr } => expr.into_type(),
        }
    }
}
impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Paren) {
            let content;
            Ok(Term::Group {
                paren: parenthesized!(content in input),
                expr: content.parse()?,
            })
        } else {
            Ok(Term::Type(input.parse()?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum UnOp {
    Neg(Token![-]),
}
impl UnOp {
    fn try_parse(input: ParseStream) -> Option<Self> {
        if input.peek(Token![-]) {
            Some(UnOp::Neg(input.parse().unwrap()))
        } else {
            None
        }
    }

    fn qualified_type(self, bound_ty: Type) -> Type {
        self.qualified_type_with_path_args(bound_ty, PathArguments::None)
    }

    fn type_param_bound(self, assoc: AssociatedType) -> TypeParamBound {
        self.type_param_bound_with_path_args(None, assoc)
    }
}
impl Op for UnOp {
    fn trait_path(&self) -> Path {
        match self {
            UnOp::Neg(minus) => parse_quote_spanned! { minus.span => ::core::ops::Neg },
        }
    }

    fn assoc_type_name(&self) -> Ident {
        match self {
            UnOp::Neg(minus) => parse_quote_spanned! { minus.span => Output },
        }
    }
}
impl Parse for UnOp {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(UnOp::Neg(input.parse()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BinOp {
    Mul(Token![*]),
}
impl BinOp {
    fn try_parse(input: ParseStream) -> Option<Self> {
        if input.peek(Token![*]) {
            Some(BinOp::Mul(input.parse().unwrap()))
        } else {
            None
        }
    }

    fn qualified_type(self, bound_ty: Type, rhs: RightOperand) -> Type {
        let path_args = path_arguments([rhs.into_generic_argument()]);
        self.qualified_type_with_path_args(bound_ty, path_args)
    }

    fn type_param_bound(self, rhs: RightOperand, assoc: AssociatedType) -> TypeParamBound {
        self.type_param_bound_with_path_args(rhs.into_generic_argument(), assoc)
    }
}
impl Op for BinOp {
    fn trait_path(&self) -> Path {
        match self {
            BinOp::Mul(star) => parse_quote_spanned! { star.span => ::core::ops::Mul },
        }
    }

    fn assoc_type_name(&self) -> Ident {
        match self {
            BinOp::Mul(star) => parse_quote_spanned! { star.span => Output },
        }
    }
}
impl Parse for BinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(BinOp::Mul(input.parse()?))
    }
}

fn path_arguments(args: impl IntoIterator<Item = Option<GenericArgument>>) -> PathArguments {
    let args: Punctuated<_, Token![,]> = args.into_iter().flatten().collect();
    if args.is_empty() {
        PathArguments::None
    } else {
        PathArguments::AngleBracketed(parse_quote! { < #args > })
    }
}

trait Op {
    fn trait_path(&self) -> Path;
    fn assoc_type_name(&self) -> Ident;

    fn qualified_type_with_path_args(self, bound_ty: Type, path_args: PathArguments) -> Type
    where
        Self: Sized,
    {
        let trait_path = self.trait_path();
        let assoc_type_name = self.assoc_type_name();
        parse_quote! { <#bound_ty as #trait_path #path_args>::#assoc_type_name }
    }

    fn type_param_bound_with_path_args(
        &self,
        rhs_arg: Option<GenericArgument>,
        assoc: AssociatedType,
    ) -> TypeParamBound
    where
        Self: Sized,
    {
        let trait_path = self.trait_path();
        let path_args = path_arguments([rhs_arg, assoc.into_generic_argument(self)]);
        parse_quote! { #trait_path #path_args }
    }
}

trait ResultExt<T> {
    fn zip_combine<U>(self, other: Result<U>) -> Result<(T, U)>;
}
impl<T> ResultExt<T> for Result<T> {
    fn zip_combine<U>(self, other: Result<U>) -> Result<(T, U)> {
        match (self, other) {
            (Ok(x), Ok(y)) => Ok((x, y)),
            (Ok(_), Err(e2)) => Err(e2),
            (Err(e1), Ok(_)) => Err(e1),
            (Err(mut e1), Err(e2)) => {
                e1.combine(e2);
                Err(e1)
            }
        }
    }
}

/*
#![no_std]
//! ...
//!
//! # Macro return
//!
//! The return from macros is handled by the [`__return`] macro.
//!
//! There are two supported methods of return. For the examples, we'll use a macro
//! that generates a bound that requires the passed type argument to implement the `Clone` trait:
//!
//! ```
//! macro_rules! with_clone_bound {
//!     ($type:ty, $($return:tt)+) => {
//!         ::ops_traits_macros::__return!(($type: Clone) { $($return)+ })
//!     };
//! }
//! ```
//!
//! ## Callback
//!
//! The caller includes a macro arm as argument to the called macro. The arm is
//! then "executed", receiving as the patterns the output of the called macro.
//!
//! ```
//! # macro_rules! with_clone_bound {
//! #     ($type:ty, $($return:tt)+) => {
//! #         ::ops_traits_macros::__return!(($type: Clone) { $($return)+ })
//! #     };
//! # }
//! with_clone_bound!(
//!     T,
//!     ($($bounds:tt)*) => {
//!         fn f<T>(x: &T) -> T
//!         where
//!             $($bounds)*
//!         {
//!             x.clone()
//!         }
//!     }
//! );
//! # #[derive(Clone)]
//! # struct S;
//! # f(&S);
//! ```
//!
//! This generates the following:
//!
//! ```ignore
//! macro_rules! __emit__ { ... }
//! fn f<T>(x: &T) -> T where T: Clone {
//!     x.clone()
//! };
//! ```
//!
//! Notice that it includes a one-shot macro `__emit__`.
//!
//! ## CPS (Continuation-Passing Style)
//!
//! The caller includes the name of a macro that will receive the return, and any additional
//! arguments.
//!
//! In the example below, the line with `define_function!(@return f)` defines that the macro
//! `define_function` is to be called with the arguments `@return f` followed by the bounds
//! generated by `with_clone_bound`.
//!
//! ```
//! # macro_rules! with_clone_bound {
//! #     ($type:ty, $($return:tt)+) => {
//! #         ::ops_traits_macros::__return!(($type: Clone) { $($return)+ })
//! #     };
//! # }
//! macro_rules! define_function {
//!     ($name:ident) => {
//!         with_clone_bound!(
//!             T,
//!             define_function!(@return $name)
//!         );
//!     };
//!
//!     (@return $name:ident ( $($bounds:tt)* )) => {
//!         fn $name<T>(x: &T) -> T
//!         where
//!             $($bounds)*
//!         {
//!             x.clone()
//!         }
//!     };
//! }
//!
//! define_function!(g);
//! # #[derive(Clone)]
//! # struct S;
//! # g(&S);
//! ```
//!
//! This generates the following:
//!
//! ```
//! fn g<T>(x: &T) -> T
//! where
//!     T: Clone,
//! {
//!     x.clone()
//! };
//! ```

/*
    Suffixes in macro patterns:
        `_p` - tokens surrounded by parentheses:     $something_p:tt == ($($something:tt)*)
        `_s` - tokens surrounded by square brackets: $something_s:tt == [$($something:tt)*]
        `_c` - tokens surrounded by curly braces:    $something_c:tt == {$($something:tt)*}
*/

/*
    CLAUSES := EMPTY | CLAUSE (',' CLAUSES?)?

    CLAUSE := UNOP OPERAND ('=' ASSOC)?
            | LEFT BINOP RIGHT ('=' ASSOC)?
            | LEFT ':' RAW-BOUND
    OPERATION := UNOP OPERAND
               | LEFT (BINOP RIGHT)?

    OPERAND := TYPE
    ASSOC := TYPE
    LEFT := TYPE
    RIGHT := '_' | TYPE
    TYPE := '(' OPERATION ')' | RAW-TYPE

    RAW-TYPE := everything except BINOP or ':' or '=' or ','
    RAW-BOUND := everything except ','
*/

#[macro_export]
macro_rules! with_ops_bounds {
    (( $($bounds:tt)* ), $($return:tt)+) => {
        $crate::__parse_clauses!([$($bounds)*] {$($return)+})
    };

    ([ $($bounds:tt)* ], $($return:tt)+) => {
        $crate::__parse_clauses!([$($bounds)*] {$($return)+})
    };

    ({ $($bounds:tt)* }, $($return:tt)+) => {
        $crate::__parse_clauses!([$($bounds)*] {$($return)+})
    };
}

#[macro_export]
macro_rules! ops_type {
    ($($operation:tt)+) => {
        $crate::__parse_operation!([$($operation)+])
    };
}

/*
 * | Input    | Returns
 * |----------|----------
 * | [TOKENS] | CLAUSES
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_clauses {
    //             [TOKENS] {RETURN}
    // -> @loop [] [TOKENS] {RETURN}
    ([$($tokens:tt)*]  $return_c:tt) => {
        $crate::__parse_clauses!(@loop  []  [$($tokens)*]  $return_c)
    };

    /*
     * @loop [DONE] [TOKENS] {RETURN}
     */

    // No tokens.
    // @loop   [DONE] [] {RETURN}
    // Return: DONE
    (@loop  [$($done:tt)*]  [/*TOKENS*/]  $return_c:tt) => {
        $crate::__return!(( $($done)* )  $return_c)
    };

    //    @loop          [DONE] [TOKENS] {RETURN}
    // -> @clause_return [DONE]          {RETURN} ([CLAUSE] [DELIMITER] [REMAINING])
    (@loop  $done_s:tt  $tokens_s:tt  $return_c:tt) => {
        $crate::__parse_clause!(
            $tokens_s
            { $crate::__parse_clauses!(@clause_return  $done_s  $return_c) }
        )
    };

    /*
     * @clause_return [DONE] {RETURN} ([CLAUSE] [DELIMITER] [REMAINING])
     */

    //    @clause_return [DONE] {RETURN} ([CLAUSE] [,?] [REMAINING])
    // -> @loop          [DONE             CLAUSE,]     [REMAINING]
    (@clause_return  [$($done:tt)*]  $return_c:tt  ([$($clause:tt)+]  [$(,)?]  $remaining_s:tt)) => {
        $crate::__parse_clauses!(@loop  [$($done)* $($clause)+, ]  $remaining_s  $return_c)
    };
}

/*
 * | Input    | Returns
 * |----------|----------
 * | [TOKENS] | [CLAUSE] [DELIMITER] [REMAINING]
 *
 * [DELIMITER] should only be [] or [,].
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_clause {
    //    [TOKENS] {RETURN}
    // -> @expr_return {RETURN} (...)
    ([$($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_expr!(
            [$($tokens)+]
            { $crate::__parse_clause!(@expr_return  $return_c) }
        )
    };

    /*
     * @expr_return {RETURN} (...)
     */

    // Unary operation. WITH associated type predicate.
    //         @expr_return {RETURN} (unop [OPERAND] [TRAIT] [ASSOC-TYPE-NAME] [=] [TOKENS])
    // -> @unop_assoc_type_return          [OPERAND] [TRAIT]  ASSOC-TYPE-NAME {RETURN} ([ASSOC] [DELIMITER] [REMAINING])
    (@expr_return  $return_c:tt  (unop  $operand_s:tt  $trait_s:tt  [$assoc_type_name:ident]  [=]  $tokens_s:tt)) => {
        $crate::__parse_type!(
            $tokens_s
            { $crate::__parse_clause!(@unop_assoc_type_return  $operand_s  $trait_s  $assoc_type_name  $return_c) }
        )
    };

    // Unary operation. WITHOUT associated type predicate.
    //         @expr_return {RETURN} (unop [OPERAND] [TRAIT] [ASSOC-TYPE-NAME?] [DELIMITER] [REMAINING])
    // Return: [OPERAND : TRAIT] [DELIMITER] [REMAINING]
    (@expr_return  $return_c:tt  (unop  [$($operand:tt)+]  [$($trait:tt)+]  $assoc_type_name_s:tt  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($operand)+ : $($trait)+]  $delimiter_s  $remaining_s)  $return_c)
    };

    // Binary operation. WITH associated type predicate.
    //    @expr_return {RETURN} (binop [LEFT] [TRAIT] [RIGHT] [ASSOC-TYPE-NAME] [=] [TOKENS])
    // -> @binop_assoc_type_return [LEFT] [TRAIT] [RIGHT] ASSOC-TYPE-NAME {RETURN} ([ASSOC] [DELIMITER] [REMAINING])
    (@expr_return  $return_c:tt  (binop  $left_s:tt  $trait_s:tt  $right_s:tt  [$assoc_type_name:ident]  [=]  $tokens_s:tt)) => {
        $crate::__parse_type!(
            $tokens_s
            { $crate::__parse_clause!(@binop_assoc_type_return  $left_s  $trait_s  $right_s  $assoc_type_name  $return_c) }
        )
    };

    // Binary operation. WITHOUT associated type predicate. Right operand IS `Self`.
    //         @expr_return {RETURN} (binop [LEFT] [TRAIT] [] [ASSOC-TYPE-NAME?] [DELIMITER] [REMAINING])
    // Return: [LEFT : TRAIT] [DELIMITER] [REMAINING]
    (@expr_return  $return_c:tt  (binop  [$($left:tt)+]  [$($trait:tt)+]  [/*RIGHT*/]  $assoc_type_name_s:tt  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($left)+ : $($trait)+]  $delimiter_s  $remaining_s)  $return_c)
    };

    // Binary operation. WITHOUT associated type predicate. Right operand is NOT `Self`.
    //         @expr_return {RETURN} (binop [LEFT] [TRAIT] [RIGHT] [ASSOC-TYPE-NAME?] [DELIMITER] [REMAINING])
    // Return: [LEFT : TRAIT < RIGHT >] [DELIMITER] [REMAINING]
    (@expr_return  $return_c:tt  (binop  [$($left:tt)+]  [$($trait:tt)+]  [$($right:tt)+]  $assoc_type_name_s:tt  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($left)+ : $($trait)+ < $($right)+ >]  $delimiter_s  $remaining_s)  $return_c)
    };

    //    @expr_return      {RETURN} (type [TYPE] [:] [TOKENS])
    // -> @raw_bound_return {RETURN}       [TYPE] ([BOUND] [DELIMITER] [REMAINING])
    (@expr_return  $return_c:tt  (type  $type_s:tt  [:]  $tokens_s:tt)) => {
        $crate::__parse_raw_bound!(
            $tokens_s
            { $crate::__parse_clause!(@raw_bound_return  $return_c  $type_s) }
        )
    };

    /*
     * @unop_assoc_type_return [OPERAND] [TRAIT] ASSOC-TYPE-NAME {RETURN} ([ASSOC] [DELIMITER] [REMAINING])
     */

    // Unary operation. WITH associated type predicate.
    //         @unop_assoc_type_return [OPERAND] [TRAIT]  ASSOC-TYPE-NAME {RETURN} ([ASSOC]   [DELIMITER] [REMAINING])
    // Return:                         [OPERAND : TRAIT < ASSOC-TYPE-NAME =          ASSOC >] [DELIMITER] [REMAINING]
    (@unop_assoc_type_return  [$($operand:tt)+]  [$($trait:tt)+]  $assoc_type_name:ident  $return_c:tt  ([$($assoc:tt)+]  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($operand)+ : $($trait)+ < $assoc_type_name = $($assoc)+ >]  $delimiter_s  $remaining_s)  $return_c)
    };

    /*
     * @binop_assoc_type_return [LEFT] [TRAIT] [RIGHT] ASSOC-TYPE-NAME {RETURN} ([ASSOC] [DELIMITER] [REMAINING])
     */

    // Binary operation. WITH associated type predicate. Right operand IS `Self`.
    //         @binop_assoc_type_return [LEFT] [TRAIT] [] ASSOC-TYPE-NAME {RETURN} ([ASSOC]   [DELIMITER] [REMAINING])
    // Return:                          [LEFT : TRAIT   < ASSOC-TYPE-NAME =          ASSOC >] [DELIMITER] [REMAINING]
    (@binop_assoc_type_return  [$($left:tt)+]  [$($trait:tt)+]  [/*RIGHT*/]  $assoc_type_name:ident  $return_c:tt  ([$($assoc:tt)+]  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($left)* : $($trait)* < $assoc_type_name = $($assoc)+ > ]  $delimiter_s  $remaining_s)  $return_c)
    };

    // Binary operation. WITH associated type predicate. Right operand is NOT `Self`.
    //         @binop_assoc_type_return [LEFT] [TRAIT] [RIGHT] ASSOC-TYPE-NAME {RETURN} ([ASSOC]   [DELIMITER] [REMAINING])
    // Return:                          [LEFT : TRAIT < RIGHT, ASSOC-TYPE-NAME =          ASSOC >] [DELIMITER] [REMAINING]
    (@binop_assoc_type_return  [$($left:tt)+]  [$($trait:tt)+]  [$($right:tt)+]  $assoc_type_name:ident  $return_c:tt  ([$($assoc:tt)+]  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($left)* : $($trait)* < $($right)+, $assoc_type_name = $($assoc)+ > ]  $delimiter_s  $remaining_s)  $return_c)
    };


    /*
     * @raw_bound_return {RETURN} [TYPE] ([BOUND] [DELIMITER] [REMAINING])
     */

    //         @raw_bound_return {RETURN} [TYPE] ([BOUND] [DELIMITER] [REMAINING])
    // Return:                            [TYPE :  BOUND] [DELIMITER] [REMAINING]
    (@raw_bound_return  $return_c:tt  [$($type:tt)+]  ([$($bound:tt)+]  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(([$($type)+ : $($bound)+]  $delimiter_s  $remaining_s)  $return_c)
    };
}

/*
 * | Input              | Generates
 * |--------------------|--------------------
 * | [UNOP OPERAND]     | '<' type(OPERAND) 'as' trait(UNOP) '>::' assoc-type-name(UNOP)
 * | [LEFT BINOP '_']   | '<' type(LEFT) 'as' trait(BINOP)                  '>::' assoc-type-name(BINOP)
 * | [LEFT BINOP RIGHT] | '<' type(LEFT) 'as' trait(BINOP) '<' type(RIGHT) '>>::' assoc-type-name(BINOP)
 * | [LEFT]             | type(LEFT)
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_operation {
    //    [TOKENS]
    // -> @expr_return (...)
    ([$($tokens:tt)+]) => {
        $crate::__parse_expr!(
            [$($tokens)+]
            { $crate::__parse_operation!(@expr_return) }
        )
    };

    /*
     * @expr_return (...)
     */

    // Unary operation.
    //           @expr_return (unop [OPERAND] [TRAIT] [ASSOC-TYPE-NAME] [] [])
    // Generate: < OPERAND as TRAIT > :: ASSOC-TYPE-NAME
    (@expr_return (unop  [$($operand:tt)+]  [$($trait:tt)+]  [$assoc_type_name:ident]  [/*DELIMITER*/]  [/*REMAINING*/])) => {
        < $($operand)+ as $($trait)+ > :: $assoc_type_name
    };

    // Binary operation. Right operand IS `Self`.
    //           @expr_return (binop [LEFT] [TRAIT] [] [ASSOC-TYPE-NAME] [] [])
    // Generate: < LEFT as TRAIT > :: ASSOC-TYPE-NAME
    (@expr_return (binop  [$($left:tt)+]  [$($trait:tt)+]  [/*RIGHT*/]  [$assoc_type_name:ident]  [/*DELIMITER*/]  [/*REMAINING*/])) => {
        < $($left)+ as $($trait)+ > :: $assoc_type_name
    };

    // Binary operation. Right operand is NOT `Self`.
    //           @expr_return (binop [LEFT] [TRAIT] [RIGHT] [ASSOC-TYPE-NAME] [] [])
    // Generate: < LEFT as TRAIT < RIGHT > > :: ASSOC-TYPE-NAME
    (@expr_return (binop  [$($left:tt)+]  [$($trait:tt)+]  [$($right:tt)+]  [$assoc_type_name:ident]  [/*DELIMITER*/]  [/*REMAINING*/])) => {
        < $($left)+ as $($trait)+ < $($right)+ > > :: $assoc_type_name
    };

    // Just a type.
    //           @expr_return (type [TYPE] [] [])
    // Generate:                     TYPE
    (@expr_return (type  [$($type:tt)+]  [/*DELIMITER*/]  [/*REMAINING*/])) => {
        $($type)+
    };

    // ?
    (@expr_return ($($tt:tt)*)) => {
        compile_error!("TODO")
    };
}

/*
 * | Input                   | Returns
 * |-------------------------|-------------------------
 * | [UNOP OPERAND REST]     | unop [type(OPERAND)] [TRAIT] [ASSOC-TYPE-NAME?] [OP-DELIMITER] [REMAINING]
 * | [LEFT BINOP RIGHT REST] | binop [type(LEFT)] [TRAIT] [type(RIGHT)] [ASSOC-TYPE-NAME?] [OP-DELIMITER] [REMAINING]
 * | [TYPE REST]             | type [type(TYPE)] [TYPE-DELIMITER] [REMAINING]
 *
 * [OP-DELIMITER] can be:
 *     []
 *     [=]
 *     [,]
 *
 * [TYPE-DELIMITER] is:
 *     []
 *     [:]
 *     [=]
 *     [,]
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_expr {
    //                [TOKEN TOKENS] {RETURN}
    // -> @unop_info_return [TOKENS] {RETURN} (...)
    ([$tok:tt  $($tokens:tt)*]  $return_c:tt) => {
        $crate::__unop_info!(
            $tok
            { $crate::__parse_expr!(@unop_info_return  $tok  [$($tokens)*]  $return_c) }
        )
    };

    /*
     * @unop_info_return TOKEN [TOKENS] {RETURN} (...)
     */

    // The token was NOT an UNOP.
    //    @unop_info_return TOKEN [TOKENS] {RETURN} ()
    // -> @left_type_return                {RETURN} ([TYPE] [DELIMITER] [REMAINING])
    (@unop_info_return  $tok:tt  [$($tokens:tt)+]  $return_c:tt  ())  => {
        $crate::__parse_type!(
            [$tok  $($tokens)*]
            { $crate::__parse_expr!(@left_type_return  $return_c) }
        )
    };

    // The token WAS an UNOP.
    //    @unop_info_return TOKEN [TOKENS] {RETURN} ([TRAIT]  ASSOC-TYPE-NAME?)
    // -> @operand_type_return                       [TRAIT] [ASSOC-TYPE-NAME?] {RETURN} ([TYPE] [DELIMITER] [REMAINING])
    (@unop_info_return  $tok:tt  $tokens_s:tt  $return_c:tt  ($trait_s:tt  $($assoc_type_name:ident)?))  => {
        $crate::__parse_type!(
            $tokens_s
            { $crate::__parse_expr!(@operand_type_return  $trait_s  [$($assoc_type_name)?]  $return_c) }
        )
    };

    /*
     * @operand_type_return [TRAIT] [ASSOC-TYPE-NAME?] {RETURN} ([TYPE] [DELIMITER] [REMAINING])
     */

    //         @operand_type_return [TRAIT] [ASSOC-TYPE-NAME?] {RETURN} ([TYPE] [DELIMITER] [REMAINING])
    // Return: unop     [TYPE]      [TRAIT] [ASSOC-TYPE-NAME?]                  [DELIMITER] [REMAINING]
    (@operand_type_return  $trait_s:tt  $assoc_type_name_s:tt  $return_c:tt  ($type_s:tt  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!((unop  $type_s  $trait_s  $assoc_type_name_s  $delimiter_s  $remaining_s)  $return_c)
    };

    /*
     * @left_type_return {RETURN} ([TYPE] [DELIMITER] [TOKENS])
     */

    //         @left_type_return {RETURN} ([TYPE] [] [])
    // Return: type                        [TYPE] [] []
    (@left_type_return  $return_c:tt  ($type_s:tt  [/*DELIMITER*/]  [/*TOKENS*/])) => {
        $crate::__return!((type  $type_s  [/*DELIMITER*/]  [/*REMAININGS*/])  $return_c)
    };

    //         @left_type_return {RETURN} ([TYPE] [:] [TOKENS])
    // Return: type                        [TYPE] [:] [TOKENS]
    (@left_type_return  $return_c:tt  ($type_s:tt  [:]  $tokens_s:tt)) => {
        $crate::__return!((type  $type_s  [:]  $tokens_s)  $return_c)
    };

    //    @left_type_return {RETURN} ([LEFT] [[TRAIT]  ASSOC-TYPE-NAME?] [TOKENS])
    // -> @right                      [LEFT]  [TRAIT] [ASSOC-TYPE-NAME?] [TOKENS] {RETURN}
    (@left_type_return  $return_c:tt  ($left_s:tt  [[$($trait:tt)+]  $($assoc_type_name:ident)?]  $tokens_s:tt)) => {
        $crate::__parse_expr!(@right  $left_s  [$($trait)+]  [$($assoc_type_name)?]  $tokens_s  $return_c)
    };

    // ?
    (@left_type_return  $return_c:tt  ($type_s:tt  $delimiter_s:tt  $tokens_s:tt)) => {
        compile_error!("TODO")
    };

    /*
     * @right [LEFT] [TRAIT] [ASSOC-TYPE-NAME?] [TOKENS] {RETURN}
     */

    // Right side IS '_'.
    //    @right            [LEFT] [TRAIT] [ASSOC-TYPE-NAME?] [_ TOKENS] {RETURN}
    // -> @delimiter_return [LEFT] [TRAIT] [ASSOC-TYPE-NAME?]            {RETURN} ([DELIMITER] [REMAINING])
    (@right  $left_s:tt  $trait_s:tt  $assoc_type_name_s:tt  [_ $($tokens:tt)*]  $return_c:tt) => {
        $crate::__parse_delimiter!(
            [$($tokens)*]
            { $crate::__parse_expr!(@delimiter_return  $left_s  $trait_s  $assoc_type_name_s  $return_c) }
        )
    };

    // Right side is NOT '_'.
    //    @right             [LEFT] [TRAIT] [ASSOC-TYPE-NAME?] [TOKENS] {RETURN}
    // -> @right_type_return [LEFT] [TRAIT] [ASSOC-TYPE-NAME?]          {RETURN} ([RIGHT] [DELIMITER] [REMAINING])
    (@right  $left_s:tt  $trait_s:tt  $assoc_type_name_s:tt  $tokens_s:tt  $return_c:tt) => {
        $crate::__parse_type!(
            $tokens_s
            { $crate::__parse_expr!(@right_type_return  $left_s  $trait_s  $assoc_type_name_s  $return_c) }
        )
    };

    /*
     * @delimiter_return [LEFT] [TRAIT] [ASSOC-TYPE-NAME?] {RETURN} ([DELIMITER] [REMAINING])
     */

    //         @delimiter_return [LEFT] [TRAIT]    [ASSOC-TYPE-NAME?] {RETURN} ([DELIMITER] [REMAINING])
    // Return: binop             [LEFT] [TRAIT] [] [ASSOC-TYPE-NAME?]           [DELIMITER] [REMAINING]
    (@delimiter_return  $left_s:tt  $trait_s:tt  $assoc_type_name_s:tt  $return_c:tt  ($delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!((binop  $left_s  $trait_s  [/*RIGHT*/]  $assoc_type_name_s  $delimiter_s  $remaining_s)  $return_c)
    };

    /*
     * @right_type_return [LEFT] [TRAIT] [ASSOC-TYPE-NAME?] {RETURN} ([RIGHT] [DELIMITER] [REMAINING])
     */

    //         @right_type_return [LEFT] [TRAIT]         [ASSOC-TYPE-NAME?] {RETURN} ([RIGHT] [DELIMITER] [REMAINING])
    // Return: binop              [LEFT] [TRAIT] [RIGHT] [ASSOC-TYPE-NAME?]                   [DELIMITER] [REMAINING]
    (@right_type_return $left_s:tt  $trait_s:tt  $assoc_type_name_s:tt  $return_c:tt  ($right_s:tt  $delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!((binop  $left_s  $trait_s  $right_s  $assoc_type_name_s  $delimiter_s  $remaining_s)  $return_c)
    };

    // ?
    (@right_type_return $left_s:tt  $trait_s:tt  $assoc_type_name_s:tt  $return_c:tt  ($right_s:tt  $delimiter_s:tt  $remaining_s:tt)) => {
        compile_error!("TODO")
    };
}

/*
 * | Input              | Returns
 * |--------------------|--------------------
 * | [(OPERATION) REST] | [type(OPERATION)] [DELIMITER] [REMAINING]
 * | [RAW-TYPE REST]    | [RAW-TYPE] [DELIMITER] [REMAINING]
 *
 * [DELIMITER] can be the same ones as returned by __parse_delimiter.
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_type {
    //                            [(TOKENS) REST] {RETURN}
    // -> @delimiter_return [parse([TOKENS])]     {RETURN} ([DELIMITER] [REMAINING])
    ([ ( $($tokens:tt)+ ) $($rest:tt)*]  $return_c:tt) => {
        $crate::__parse_delimiter!(
            [$($rest)*]
            { $crate::__parse_type!(@delimiter_return [$crate::__parse_operation!([$($tokens)+])]  $return_c) }
        )
    };

    //          [TOKENS] {RETURN}
    // -> @loop [TOKENS] {RETURN}
    ([$($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_type!(@loop [] [$($tokens)+]  $return_c)
    };

    /*
     * @delimiter_return [TYPE] {RETURN} ([DELIMITER] [REMAINING])
     */

    //         @delimiter_return [TYPE] {RETURN} ([DELIMITER] [REMAINING])
    // Return:                   [TYPE]           [DELIMITER] [REMAINING]
    (@delimiter_return  $type_s:tt  $return_c:tt  ($delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(($type_s  $delimiter_s  $remaining_s)  $return_c)
    };

    /*
     * @loop [DONE] [PENDING] {RETURN}
     */

    //    @loop [DONE] [< ... >    TOKENS] {RETURN}
    // -> @loop [DONE   < ... >] [ TOKENS] {RETURN}
    (@loop  $done_s:tt  [< $($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_angle_brackets_content!(
            [$($tokens)+]
            { $crate::__parse_type!(@angle_return  $done_s  $return_c) }
        )
    };

    //    @loop [DONE] [<< ... > INSIDE >   TOKENS] {RETURN}
    // -> @loop [DONE   << ... > INSIDE >] [TOKENS] {RETURN}
    (@loop  $done_s:tt  [<< $($tokens:tt)+]  $return_c:tt) => {
        compile_error!("TODO")
    };

    //    @loop        [DONE] [TOKENS] {RETURN}
    // -> @loop_return [DONE]          {RETURN} (DELIMITER [REMAINING])
    (@loop  $done_s:tt  $tokens_s:tt  $return_c:tt) => {
        $crate::__parse_delimiter!(
            $tokens_s
            { $crate::__parse_type!(@loop_return  $done_s  $return_c) }
        )
    };

    /*
     * @angle_return [DONE] {RETURN} ([CONTENT] [REMAINING])
     */

    //    @angle_return [DONE] {RETURN} ([CONTENT]   [REMAINING])
    // -> @loop         [DONE           < CONTENT >] [REMAINING] {RETURN}
    (@angle_return  [$($done:tt)*]  $return_c:tt  ([$($content:tt)*]  $remaining_s:tt)) => {
        $crate::__parse_type!(@loop [$($done)* < $($content)* >]  $remaining_s  $return_c)
    };

    /*
     * @loop_return [DONE] {RETURN} (DELIMITER [REMAINING])
     */

    //    @loop_return [DONE] {RETURN} (_not_ [TOKEN   REST])
    // -> @loop        [DONE                   TOKEN] [REST] {RETURN}
    (@loop_return  [$($done:tt)*]  $return_c:tt  (_not_  [$tok:tt  $($rest:tt)*])) => {
        $crate::__parse_type!(@loop  [$($done)*  $tok]  [$($rest)*]  $return_c)
    };

    //         @loop_return [DONE] {RETURN} ([DELIMITER] [REMAINING])
    // Return:              [DONE]           [DELIMITER] [REMAINING]
    (@loop_return  $done_s:tt  $return_c:tt  ($delimiter_s:tt  $remaining_s:tt)) => {
        $crate::__return!(($done_s  $delimiter_s  $remaining_s) $return_c)
    };
}

/*
 * This macro is intended to be used when a type is parsed without reaching the delimiter after
 * it, such as when:
 *     - '_' is parsed as RIGHT
 *     - '(' OPERATION ')' is parsed as TYPE
 *
 * | Input        | Returns
 * |--------------|--------------
 * | [TOKENS]     | [DELIMITER] [REMAINING]    /* general case */
 * | []           | [] []
 * | [: REST]     | [:] [REST]
 * | [= REST]     | [=] [REST]
 * | [, REST]     | [,] [REST]
 * | [BINOP REST] | [trait(BINOP) assoc-type-name(BINOP)?] [REST]
 * | [TOKEN REST] | _not_ [TOKEN REST]         /* the next token is not a delimiter */
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_delimiter {
    // No more tokens.
    //         [] {RETURN}
    // Return: [] []
    ([/*TOKENS*/]  $return_c:tt) => {
        $crate::__return!(([/*DELIMITER*/] [/*REMAINING*/]) $return_c)
    };

    //         [:   TOKENS] {RETURN}
    // Return: [:] [TOKENS]
    ([:  $($tokens:tt)*]  $return_c:tt) => {
        $crate::__return!(([:]  [$($tokens)*])  $return_c)
    };

    //         [=   TOKENS] {RETURN}
    // Return: [=] [TOKENS]
    ([=  $($tokens:tt)*]  $return_c:tt) => {
        $crate::__return!(([=]  [$($tokens)*])  $return_c)
    };

    //         [,   TOKENS] {RETURN}
    // Return: [,] [TOKENS]
    ([,  $($tokens:tt)*]  $return_c:tt) => {
        $crate::__return!(([,]  [$($tokens)*])  $return_c)
    };

    //                       [TOKEN  TOKENS] {RETURN}
    // -> @binop_info_return  TOKEN [TOKENS] {RETURN} (...)
    ([$tok:tt  $($tokens:tt)*]  $return_c:tt) => {
        $crate::__binop_info!(
            $tok
            { $crate::__parse_delimiter!(@binop_info_return  $tok  [$($tokens)*]  $return_c) }
        )
    };

    /*
     * @binop_info_return TOKEN [TOKENS] {RETURN} (...)
     */

    // The token is not recognized.
    //         @binop_info_return TOKEN [TOKENS] {RETURN} ()
    // Return: not               [TOKEN  TOKENS]
    (@binop_info_return  $tok:tt  [$($tokens:tt)*]  $return_c:tt  ()) => {
        $crate::__return!((_not_  [$tok  $($tokens)*])  $return_c)
    };

    // The token is BINOP.
    //         @binop_info_return BINOP [TOKENS] {RETURN} ([TRAIT] ASSOC-TYPE-NAME?)
    // Return:                                            [[TRAIT] ASSOC-TYPE-NAME?] [TOKENS]
    (@binop_info_return  $tok:tt  $tokens_s:tt  $return_c:tt  ($trait_s:tt  $($assoc_type_name:ident)?)) => {
        $crate::__return!(([$trait_s  $($assoc_type_name)?]  $tokens_s)  $return_c)
    };
}

/*
 * | Input | Returns
 * |-------|-------
 * | OP    | [TRAIT] ASSOC-TYPE-NAME?
 * | OP    | NOTHING
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __unop_info {
    (- $return_c:tt) => {
        $crate::__return!(([::core::ops::Neg] Output) $return_c)
    };

    // Unrecognized operator.
    ($op:tt $return_c:tt) => {
        $crate::__return!(() $return_c)
    };
}

/*
 * | Input | Returns
 * |-------|-------
 * | OP    | [TRAIT] ASSOC-TYPE-NAME?
 * | OP    | NOTHING
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __binop_info {
    (+ $return_c:tt) => {
        $crate::__return!(([::core::ops::Add] Output) $return_c)
    };

    (- $return_c:tt) => {
        $crate::__return!(([::core::ops::Sub] Output) $return_c)
    };

    (* $return_c:tt) => {
        $crate::__return!(([::core::ops::Mul] Output) $return_c)
    };

    // Unrecognized operator.
    ($op:tt $return_c:tt) => {
        $crate::__return!(() $return_c)
    };
}

/*
 * Parses anything until the end or until a comma that is not inside braces.
 * | Input    | Returns
 * |----------|----------
 * | [TOKENS] | [BOUND] [DELIMITER] [REMAINING]
 *
 * [DELIMITER] is [] or [,].
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_raw_bound {
    ([$($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_raw_bound!(@loop  []  [$($tokens)+]  $return_c)
    };

    /*
     * @loop [DONE] [TOKENS] {RETURN}
     */

    // ?
    (@loop  $done_s:tt  [/*TOKENS*/]  $return_c:tt) => {
        compile_error!("TODO")
    };

    //    @loop [DONE] [< ... >   TOKENS] {RETURN}
    // -> @loop [DONE   < ... >] [TOKENS] {RETURN}
    (@loop  $done_s:tt  [< $($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_angle_brackets_content!(
            [$($tokens)+]
            { $crate::__parse_raw_bound!(@angle_return  $done_s  $return_c) }
        )
    };

    // ?
    (@loop  $done_s:tt  [<< $($tokens:tt)+]  $return_c:tt) => {
        compile_error!("TODO")
    };

    //         @loop [DONE] [,   TOKENS] {RETURN}
    // Return:       [DONE] [,] [TOKENS]
    (@loop  $done_s:tt  [, $($tokens:tt)*]  $return_c:tt) => {
        $crate::__return!(($done_s  [,]  [$($tokens)*])  $return_c)
    };

    //    @loop [DONE] [TOKEN   TOKENS] {RETURN}
    // -> @loop [DONE   TOKEN] [TOKENS] {RETURN}
    (@loop  [$($done:tt)*]  [$tok:tt $($tokens:tt)*]  $return_c:tt) => {
        $crate::__parse_raw_bound!(@loop  [$($done)*  $tok]  [$($tokens)*]  $return_c)
    };

    /*
     * @angle_return [DONE] {RETURN} ([CONTENT] [REMAINING])
     */

    //    @angle_return [DONE] {RETURN} ([CONTENT]   [REMAINING])
    // -> @loop         [DONE           < CONTENT >] [REMAINING] {RETURN}
    (@angle_return [$($done:tt)*]  $return_c:tt  ([$($content:tt)*]  $remaining_s:tt)) => {
        $crate::__parse_raw_bound!(@loop  [$($done)* < $($content)* >]  $remaining_s  $return_c)
    };
}

/*
 * The opening angle bracket ('<') must already have being consumed.
 *
 * | Input           | Returns
 * |-----------------|-----------------
 * | [TOKENS > REST] | [TOKENS] [REST]
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __parse_angle_brackets_content {
    ([$($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_angle_brackets_content!(@loop [] [$($tokens)+]  $return_c)
    };

    /*
     * @loop [DONE] [TOKENS] {RETURN}
     */

    //         @loop [DONE] [> TOKENS] {RETURN}
    // Return:       [DONE]   [TOKENS]
    (@loop  $done_s:tt  [> $($tokens:tt)*]  $return_c:tt) => {
        $crate::__return!(($done_s  [$($tokens)*])  $return_c)
    };

    //    @loop [DONE]  [>> TOKENS] {RETURN}
    // -> @loop [DONE] [> > TOKENS] {RETURN}
    (@loop  $done_s:tt  [>> $($tokens:tt)*]  $return_c:tt) => {
        $crate::__parse_angle_brackets_content!(@loop  $done_s  [> > $($tokens)*]  $return_c)
    };

    //    @loop [DONE] [< ... >   TOKENS] {RETURN}
    // -> @loop [DONE   < ... >] [TOKENS] {RETURN}
    (@loop  $done_s:tt  [< $($tokens:tt)+]  $return_c:tt) => {
        // "Recursive" call.
        $crate::__parse_angle_brackets_content!(
            [$($tokens)+]
            { $crate::__parse_angle_brackets_content!(@angle_return  $done_s  $return_c) }
        )
    };

    // ?
    (@loop  $done_s:tt  [<< $($tokens:tt)+]  $return_c:tt) => {
        compile_error!("TODO")
    };

    //    @loop [DONE] [TOKEN   TOKENS] {RETURN}
    // -> @loop [DONE   TOKEN] [TOKENS] {RETURN}
    (@loop  [$($done:tt)*]  [$tok:tt $($tokens:tt)+]  $return_c:tt) => {
        $crate::__parse_angle_brackets_content!(@loop [$($done)*  $tok]  [$($tokens)+]  $return_c)
    };

    /*
     * @angle_return [DONE] {RETURN} ([CONTENT] [REMAINING])
     */

    //    @angle_return [DONE] {RETURN} ([CONTENT]   [REMAINING])
    // -> @loop         [DONE           < CONTENT >] [REMAINING] {RETURN}
    (@angle_return  [$($done:tt)*]  $return_c:tt  ([$($content:tt)*]  $remaining_s:tt)) => {
        // Returning from "recursive" call.
        $crate::__parse_angle_brackets_content!(@loop [$($done)* < $($content)* >]  $remaining_s  $return_c)
    };
}

/*
 * Input: (RESULT) {RETURN}
 */
#[doc(hidden)]
#[macro_export]
macro_rules! __return {
    // Callback
    (( $($result:tt)* ) { ( $($macro_pattern:tt)* ) $($macro_rest:tt)* }) => {
        macro_rules! __emit__ { ( $($macro_pattern)* ) $($macro_rest)* }
        __emit__!($($result)*)
    };

    // CPS
    ($result_p:tt  { $macro_name:ident $( ::$path:ident )* ! ($( $macro_args:tt )*) }) => {
        // MACRO!(ARGS (RESULT))
        $macro_name $(::$path)* ! ($($macro_args)*  $result_p)
    };
}
*/

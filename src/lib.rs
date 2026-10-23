use std::collections::{HashMap, HashSet};
use std::iter::once;

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
    OpsBound := OpsExpr AssociatedType
    OpsExpr := UnOp Term | Term BinOp RightOperand

    AssociatedType := '=' Term | EMPTY
    RightOperand := '_' | Term
    Term := Type | '(' OpsExpr ')' | '(' Type ')'

    UnOp := '-'
    BinOp := '+' | '-' | '*'
*/

#[proc_macro_attribute]
pub fn ops_bounds(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ops_bounds = {
        let parser = Punctuated::<OpsBound, Token![,]>::parse_terminated;
        parser.parse(attr)
    };

    let mut item = parse_macro_input!(item as Item);
    let where_clause = match &mut item {
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
    };

    zip_combine_results(where_clause, ops_bounds)
        .map(|(where_clause, bounds)| {
            let predicates = ops_bounds_to_where_predicates(bounds);
            where_clause.predicates.extend(predicates);
        })
        .map_or_else(Error::into_compile_error, |()| item.into_token_stream())
        .into()
}

fn ops_bounds_to_where_predicates(
    bounds: impl IntoIterator<Item = OpsBound>,
) -> impl Iterator<Item = WherePredicate> {
    let mut unique_bounds = HashMap::new();

    for bound in bounds {
        for expr in bound.sub_exprs() {
            unique_bounds.entry(expr).or_insert(HashSet::new());
        }

        unique_bounds
            .entry(bound.expr)
            .or_insert(HashSet::new())
            .insert(bound.assoc);
    }

    unique_bounds
        .into_iter()
        .flat_map(|(expr, assocs)| {
            // Ensure there is at least one OpsBound.
            let no_assocs = assocs.is_empty().then(|| OpsBound::from(expr.clone()));

            let with_assocs = assocs.into_iter().map(move |assoc| OpsBound {
                expr: expr.clone(),
                assoc,
            });

            no_assocs.into_iter().chain(with_assocs)
        })
        .map(OpsBound::into_where_predicate)
}

#[proc_macro]
pub fn ops_type(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as OpsExpr)
        .into_type()
        .into_token_stream()
        .into()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OpsBound {
    expr: OpsExpr,
    assoc: AssociatedType,
}
impl OpsBound {
    fn sub_exprs(&self) -> impl Iterator<Item = OpsExpr> + use<> {
        self.expr.sub_exprs().chain(self.assoc.all_exprs())
    }

    fn into_where_predicate(self) -> WherePredicate {
        let (bounded_term, bound) = match self.expr {
            OpsExpr::Unary { unop, operand } => (operand, unop.type_param_bound(self.assoc)),
            OpsExpr::Binary { left, binop, right } => {
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
            expr: input.parse()?,
            assoc: input.parse()?,
        })
    }
}
impl From<OpsExpr> for OpsBound {
    fn from(expr: OpsExpr) -> Self {
        OpsBound {
            expr,
            assoc: AssociatedType::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OpsExpr {
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
impl OpsExpr {
    fn all_exprs(&self) -> Box<dyn Iterator<Item = OpsExpr>> {
        Box::new(once(self.clone()).chain(self.sub_exprs()))
    }

    fn sub_exprs(&self) -> impl Iterator<Item = OpsExpr> + use<> {
        let (unary, binary) = match self {
            OpsExpr::Unary { unop: _, operand } => (Some(operand.all_exprs()), None),
            OpsExpr::Binary {
                left,
                binop: _,
                right,
            } => (None, Some(left.all_exprs().chain(right.all_exprs()))),
        };

        unary
            .into_iter()
            .flatten()
            .chain(binary.into_iter().flatten())
    }

    fn into_type(self) -> Type {
        match self {
            OpsExpr::Unary { unop, operand } => unop.qualified_type(operand.into_type()),
            OpsExpr::Binary { left, binop, right } => binop.qualified_type(left.into_type(), right),
        }
    }
}
impl Parse for OpsExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Some(unop) = UnOp::try_parse(input)? {
            // UnOp Term
            Ok(OpsExpr::Unary {
                unop,
                operand: input.parse()?,
            })
        } else {
            // Term BinOp RightOperand
            Ok(OpsExpr::Binary {
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
    fn all_exprs(&self) -> impl Iterator<Item = OpsExpr> + use<> {
        let opers = match self {
            RightOperand::SelfRhs(_) => None,
            RightOperand::Term(term) => Some(term.all_exprs()),
        };

        opers.into_iter().flatten()
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
    fn all_exprs(&self) -> impl Iterator<Item = OpsExpr> + use<> {
        let opers = match self {
            AssociatedType::None => None,
            AssociatedType::Some { eq_token: _, term } => Some(term.all_exprs()),
        };

        opers.into_iter().flatten()
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
    Type { paren: Option<Paren>, ty: Box<Type> },
    Group { paren: Paren, expr: Box<OpsExpr> },
}
impl Term {
    fn all_exprs(&self) -> impl Iterator<Item = OpsExpr> + use<> {
        let opers = match self {
            Term::Type { .. } => None,
            Term::Group { paren: _, expr } => Some(expr.all_exprs()),
        };

        opers.into_iter().flatten()
    }

    fn into_type(self) -> Type {
        match self {
            Term::Type { paren: _, ty } => *ty,
            Term::Group { paren: _, expr } => expr.into_type(),
        }
    }
}
impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Paren) {
            // '(' OpsExprOrType ')'
            let content;
            let paren = parenthesized!(content in input);

            match content.parse()? {
                OpsExprOrType::OpsExpr(expr) => Ok(Term::Group {
                    paren,
                    expr: Box::new(expr),
                }),
                OpsExprOrType::Type(ty) => Ok(Term::Type {
                    paren: Some(paren),
                    ty,
                }),
            }
        } else {
            // Type
            Ok(Term::Type {
                paren: None,
                ty: Box::new(input.call(Type::without_plus)?),
            })
        }
    }
}
impl<T: Into<Box<Type>>> From<T> for Term {
    fn from(ty: T) -> Self {
        Term::Type {
            paren: None,
            ty: ty.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum UnOp {
    Neg(Token![-]),
}
impl UnOp {
    fn try_parse(input: ParseStream) -> Result<Option<Self>> {
        if input.peek(Token![-]) {
            Ok(Some(UnOp::Neg(input.parse()?)))
        } else {
            Ok(None)
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
    Add(Token![+]),
    Sub(Token![-]),
    Mul(Token![*]),
}
impl BinOp {
    fn try_parse(input: ParseStream) -> Result<Option<Self>> {
        if input.peek(Token![+]) {
            Ok(Some(BinOp::Add(input.parse()?)))
        } else if input.peek(Token![-]) {
            Ok(Some(BinOp::Sub(input.parse()?)))
        } else if input.peek(Token![*]) {
            Ok(Some(BinOp::Mul(input.parse()?)))
        } else {
            Ok(None)
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
            BinOp::Add(plus) => parse_quote_spanned! { plus.span => ::core::ops::Add },
            BinOp::Sub(minus) => parse_quote_spanned! { minus.span => ::core::ops::Sub },
            BinOp::Mul(star) => parse_quote_spanned! { star.span => ::core::ops::Mul },
        }
    }

    fn assoc_type_name(&self) -> Ident {
        match self {
            BinOp::Add(plus) => parse_quote_spanned! { plus.span => Output },
            BinOp::Sub(minus) => parse_quote_spanned! { minus.span => Output },
            BinOp::Mul(star) => parse_quote_spanned! { star.span => Output },
        }
    }
}
impl Parse for BinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let la = input.lookahead1();

        if la.peek(Token![+]) {
            Ok(BinOp::Add(input.parse()?))
        } else if la.peek(Token![-]) {
            Ok(BinOp::Sub(input.parse()?))
        } else if la.peek(Token![*]) {
            Ok(BinOp::Mul(input.parse()?))
        } else {
            Err(la.error())
        }
    }
}

/*
   Helper type to help parsing Term.

   The rule
       Term := Type | '(' OpsExpr ')' | '(' Type ')'
   becomes
       Term := Type | '(' OpsExprOrType ')'

   Then:
       OpsExprOrType := Type | OpsExpr
           ->
       OpsExprOrType := Type
                      | UnOp Term
                      | Term BinOp RightOperand
           ->
       OpsExprOrType := Type
                      | UnOp Term
                      | Type BinOp RightOperand
                      | '(' OpsExprOrType ')' BinOp RightOperand
           ->
       OpsExprOrType := UnOp Term
                      | '(' OpsExprOrType ')' BinOp RightOperand
                      | Type (BinOp RightOperand)?

*/
enum OpsExprOrType {
    OpsExpr(OpsExpr),
    Type(Box<Type>),
}
impl Parse for OpsExprOrType {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Some(unop) = UnOp::try_parse(input)? {
            // UnOp Term
            Ok(OpsExprOrType::OpsExpr(OpsExpr::Unary {
                unop,
                operand: input.parse()?,
            }))
        } else if input.peek(Paren) {
            // '(' OpsExprOrType ')' BinOp RightOperand
            Ok(OpsExprOrType::OpsExpr(input.parse()?))
        } else {
            let ty: Type = input.parse()?;
            if let Some(binop) = BinOp::try_parse(input)? {
                // Type BinOp RightOperand
                Ok(OpsExprOrType::OpsExpr(OpsExpr::Binary {
                    left: Box::new(Term::from(ty)),
                    binop,
                    right: input.parse()?,
                }))
            } else {
                // Type
                Ok(OpsExprOrType::Type(Box::new(ty)))
            }
        }
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

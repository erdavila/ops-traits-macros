use syn::punctuated::Punctuated;
use syn::{GenericArgument, PathArguments, Token, Type, WherePredicate, parse_quote};

use crate::operator::AssocTyOp;
use crate::{AssocTyPredicate, Expression, Op, RightOperand};

pub(crate) fn where_predicate(
    left: Expression,
    op: &impl Op,
    right: Option<RightOperand>,
) -> WherePredicate {
    let path_args = path_arguments(right, None);
    where_predicate_with_path_args(left, op, &path_args)
}

pub(crate) fn where_predicate_with_assoc(
    left: Expression,
    op: &impl AssocTyOp,
    right: Option<RightOperand>,
    assoc: AssocTyPredicate,
) -> WherePredicate {
    let path_args = path_arguments(right, Some((op, assoc)));
    where_predicate_with_path_args(left, op, &path_args)
}

/*
 *  | WherePredicate------------------------------------------------------------------ |
 *  | Primary | ':' | TypeParamBound-------------------------------------------------- |
 *  |         |     | Path | PathArguments-------------------------------------------- |
 *  |         |     |      | '<' | GenericArgument | ',' | GenericArgument------ | '>' |
 *  |         |     |      |     | Primary-------- |     | Ident | '=' | Primary |     |
 */
fn where_predicate_with_path_args(
    left: Expression,
    op: &impl Op,
    path_args: &PathArguments,
) -> WherePredicate {
    let bounded_ty = left.into_type();
    let trait_path = op.trait_path();

    parse_quote! { #bounded_ty : #trait_path #path_args }
}

/*
 *  | Type--------------------------------------------------------- |
 *  | '<' | Type | 'as' | Path | PathArguments | '>' | '::' | Ident |
 */
pub(crate) fn qualified_type(
    left: Expression,
    op: &impl AssocTyOp,
    right: Option<RightOperand>,
) -> Type {
    let bounded_ty = left.into_type();
    let trait_path = op.trait_path();
    let path_args = path_arguments(right, None);
    let assoc_ty_name = op.assoc_type_name();

    parse_quote! { < #bounded_ty as #trait_path #path_args > :: #assoc_ty_name }
}

fn path_arguments(
    right: Option<RightOperand>,
    assoc: Option<(&dyn AssocTyOp, AssocTyPredicate)>,
) -> PathArguments {
    let rhs_arg: Option<GenericArgument> = right.and_then(|right| match right {
        RightOperand::SelfRhs(_) => None,
        RightOperand::Expression(expr) => {
            let rhs = expr.into_type();
            Some(parse_quote! { #rhs })
        }
    });

    let assoc_arg: Option<GenericArgument> = assoc.map(|(op, assoc)| {
        let assoc_name = op.assoc_type_name();
        let AssocTyPredicate { eq_token, expr } = assoc;
        let assoc_pred = expr.into_type();
        parse_quote! { #assoc_name #eq_token #assoc_pred }
    });

    let args: Punctuated<_, Token![,]> = rhs_arg.into_iter().chain(assoc_arg).collect();

    if args.is_empty() {
        PathArguments::None
    } else {
        PathArguments::AngleBracketed(parse_quote! { < #args > })
    }
}

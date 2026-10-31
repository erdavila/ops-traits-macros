use std::iter::empty;

use indexmap::{IndexMap, IndexSet};
use syn::punctuated::Punctuated;
use syn::{
    GenericArgument, Ident, Path, PathArguments, Token, Type, TypeParamBound, WherePredicate,
    parse_quote,
};

use crate::grammar::{
    AssocTyOpOpsBound, AssocTyPredicate, BinaryOperation, Expression, Operation, Primary,
    RightOperand, UnaryOperation,
};
use crate::operator::AssocTyOp;
use crate::{CombineBinOp, NoAssocTyOpOpsBound, Op, OpsBound, OpsExpr, UnOp};

impl AssocTyOpOpsBound {
    fn into_assoc_ty_op_predicates(
        self,
    ) -> AssocTyOpPredicates<impl Iterator<Item = OperationPredicate>> {
        let operation_preds = self.oper.into_predicates();

        let (assoc, assoc_iter) = if let Some(assoc) = self.assoc {
            let assoc_preds = assoc.into_predicates();
            let assoc_ty = assoc_preds.self_pred.clone().into_type();
            let assoc_preds_iter = assoc_preds.into_iter();
            (Some(assoc_ty), Some(assoc_preds_iter))
        } else {
            (None, None)
        };

        AssocTyOpPredicates {
            self_pred: AssocTyOpPredicate {
                bounded_ty: operation_preds.self_pred.bounded_ty,
                bound: *operation_preds.self_pred.bound,
                assoc,
            },
            sub_preds: operation_preds
                .sub_preds
                .chain(assoc_iter.into_iter().flatten()),
        }
    }
}

impl NoAssocTyOpOpsBound {
    fn into_no_assoc_ty_op_predicates(
        self,
    ) -> NoAssocTyOpPredicates<impl Iterator<Item = OperationPredicate>> {
        self.0.into_no_assoc_ty_op_predicates()
    }
}

impl Operation {
    fn into_predicates(self) -> OperationPredicates<impl Iterator<Item = OperationPredicate>> {
        match self {
            Operation::Unary(un_oper) => {
                let preds = un_oper.into_predicates();
                preds.map_sub_preds(AlternativeIterator::One)
            }
            Operation::Binary(bin_oper) => {
                let preds = bin_oper.into_predicates();
                preds.map_sub_preds(AlternativeIterator::Two)
            }
        }
    }

    fn into_predicate(self) -> OperationPredicate {
        match self {
            Operation::Unary(un_oper) => un_oper.into_predicate(),
            Operation::Binary(bin_oper) => bin_oper.into_predicate(),
        }
    }
}

impl UnaryOperation {
    fn into_predicates(self) -> OperationPredicates<impl Iterator<Item = OperationPredicate>> {
        let operand_preds = self.operand.into_predicates();

        OperationPredicates {
            self_pred: Self::make_predicate(
                operand_preds.self_pred.clone().into_type(),
                &self.unop,
            ),
            sub_preds: operand_preds.into_iter(),
        }
    }

    fn into_predicate(self) -> OperationPredicate {
        Self::make_predicate(self.operand.into_predicate().into_type(), &self.unop)
    }

    fn make_predicate(bounded_ty: Type, unop: &UnOp) -> OperationPredicate {
        OperationPredicate {
            bounded_ty,
            bound: Box::new(AssocTyOpBound {
                trait_path: unop.trait_path(),
                rhs: None,
                assoc_ty_name: unop.assoc_type_name(),
            }),
        }
    }
}

impl<O: Op> BinaryOperation<O> {
    fn into_no_assoc_ty_op_predicates(
        self,
    ) -> NoAssocTyOpPredicates<impl Iterator<Item = OperationPredicate>> {
        let left_preds = self.left.into_predicates();

        let (rhs, right_iter) = if let Some(right_preds) = self.right.into_predicates() {
            let rhs = right_preds.self_pred.clone().into_type();
            let right_iter = right_preds.into_iter();
            (Some(rhs), Some(right_iter))
        } else {
            (None, None)
        };

        NoAssocTyOpPredicates {
            self_pred: Self::make_no_assoc_ty_op_predicate(
                left_preds.self_pred.clone().into_type(),
                &self.binop,
                rhs,
            ),
            sub_preds: left_preds
                .into_iter()
                .chain(right_iter.into_iter().flatten()),
        }
    }

    fn make_no_assoc_ty_op_predicate(
        bounded_ty: Type,
        binop: &O,
        rhs: Option<Type>,
    ) -> NoAssocTyOpPredicate {
        NoAssocTyOpPredicate {
            bounded_ty,
            bound: NoAssocTyOpBound {
                trait_path: binop.trait_path(),
                rhs,
            },
        }
    }
}
impl BinaryOperation<CombineBinOp> {
    fn into_predicates(self) -> OperationPredicates<impl Iterator<Item = OperationPredicate>> {
        let assoc_ty_name = self.binop.assoc_type_name();
        let preds = self.into_no_assoc_ty_op_predicates();
        preds.map_self_pred(|self_pred| self_pred.with_assoc_ty_name(assoc_ty_name))
    }

    fn into_predicate(self) -> OperationPredicate {
        let assoc_ty_name = self.binop.assoc_type_name();
        let preds = Self::make_no_assoc_ty_op_predicate(
            self.left.into_type(),
            &self.binop,
            self.right.into_type(),
        );
        preds.with_assoc_ty_name(assoc_ty_name)
    }
}

impl Expression {
    fn into_predicates(self) -> ExpressionPredicates<Box<dyn Iterator<Item = OperationPredicate>>> {
        match self {
            Expression::Primary(primary) => {
                let preds = primary.into_predicates();
                preds.map_sub_preds(AlternativeIterator::One)
            }
            Expression::Operation(operation) => {
                let oper_preds = operation.into_predicates();

                let preds = ExpressionPredicates {
                    self_pred: ExpressionPredicate::Operation(oper_preds.self_pred),
                    sub_preds: oper_preds.sub_preds,
                };
                preds.map_sub_preds(AlternativeIterator::Two)
            }
        }
        .map_sub_preds(|sub_preds| Box::new(sub_preds) as _)
    }

    fn into_type(self) -> Type {
        self.into_predicate().into_type()
    }

    fn into_predicate(self) -> ExpressionPredicate {
        match self {
            Expression::Primary(primary) => primary.into_predicate(),
            Expression::Operation(operation) => {
                ExpressionPredicate::Operation(operation.into_predicate())
            }
        }
    }
}

impl AssocTyPredicate {
    fn into_predicates(self) -> ExpressionPredicates<impl Iterator<Item = OperationPredicate>> {
        self.expr.into_predicates()
    }
}

impl RightOperand {
    fn into_predicates(
        self,
    ) -> Option<ExpressionPredicates<impl Iterator<Item = OperationPredicate>>> {
        self.into_expression().map(Expression::into_predicates)
    }

    fn into_type(self) -> Option<Type> {
        self.into_expression().map(Expression::into_type)
    }

    fn into_expression(self) -> Option<Expression> {
        match self {
            RightOperand::SelfRhs(_) => None,
            RightOperand::Expression(expr) => Some(*expr),
        }
    }
}

impl Primary {
    fn into_predicates(self) -> ExpressionPredicates<impl Iterator<Item = OperationPredicate>> {
        match self {
            Primary::Type(ty) => {
                let preds = ExpressionPredicates {
                    self_pred: ExpressionPredicate::TypeOnly(*ty),
                    sub_preds: empty(),
                };
                preds.map_sub_preds(AlternativeIterator::One)
            }
            Primary::Expression { expr, .. } => {
                let preds = expr.into_predicates();
                preds.map_sub_preds(AlternativeIterator::Two)
            }
        }
    }

    fn into_predicate(self) -> ExpressionPredicate {
        match self {
            Primary::Type(ty) => ExpressionPredicate::TypeOnly(*ty),
            Primary::Expression { expr, .. } => expr.into_predicate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Predicates<T, I> {
    self_pred: T,
    sub_preds: I,
}
impl<T, I> Predicates<T, I> {
    fn map_self_pred<U>(self, f: impl FnOnce(T) -> U) -> Predicates<U, I> {
        Predicates {
            self_pred: f(self.self_pred),
            sub_preds: self.sub_preds,
        }
    }

    fn map_sub_preds<J>(self, f: impl FnOnce(I) -> J) -> Predicates<T, J> {
        Predicates {
            self_pred: self.self_pred,
            sub_preds: f(self.sub_preds),
        }
    }
}
impl<I> Predicates<ExpressionPredicate, I>
where
    I: Iterator<Item = OperationPredicate>,
{
    fn into_iter(self) -> impl Iterator<Item = OperationPredicate> {
        let self_pred = match self.self_pred {
            ExpressionPredicate::TypeOnly(_) => None,
            ExpressionPredicate::Operation(pred) => Some(pred),
        };

        self_pred.into_iter().chain(self.sub_preds)
    }
}

type AssocTyOpPredicates<I> = Predicates<AssocTyOpPredicate, I>;
type NoAssocTyOpPredicates<I> = Predicates<NoAssocTyOpPredicate, I>;
type OperationPredicates<I> = Predicates<OperationPredicate, I>;
type ExpressionPredicates<I> = Predicates<ExpressionPredicate, I>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssocTyOpPredicate {
    bounded_ty: Type,
    bound: AssocTyOpBound,
    assoc: Option<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NoAssocTyOpPredicate {
    bounded_ty: Type,
    bound: NoAssocTyOpBound,
}
impl NoAssocTyOpPredicate {
    fn with_assoc_ty_name(self, assoc_ty_name: Ident) -> OperationPredicate {
        OperationPredicate {
            bounded_ty: self.bounded_ty,
            bound: Box::new(self.bound.with_assoc_ty_name(assoc_ty_name)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OperationPredicate {
    bounded_ty: Type,
    bound: Box<AssocTyOpBound>,
}
impl OperationPredicate {
    fn into_type(self) -> Type {
        qualified_type(
            &self.bounded_ty,
            &self.bound.trait_path,
            self.bound.rhs,
            &self.bound.assoc_ty_name,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ExpressionPredicate {
    TypeOnly(Type),
    Operation(OperationPredicate),
}
impl ExpressionPredicate {
    fn into_type(self) -> Type {
        match self {
            ExpressionPredicate::TypeOnly(ty) => ty,
            ExpressionPredicate::Operation(pred) => pred.into_type(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssocTyOpBound {
    trait_path: Path,
    rhs: Option<Type>,
    assoc_ty_name: Ident,
}
impl AssocTyOpBound {
    fn into_type_param_bound(self) -> TypeParamBound {
        type_param_bound(&self.trait_path, self.rhs, None)
    }

    fn into_type_param_bound_with_assoc(self, assoc: Type) -> TypeParamBound {
        type_param_bound(
            &self.trait_path,
            self.rhs,
            Some((self.assoc_ty_name, assoc)),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NoAssocTyOpBound {
    trait_path: Path,
    rhs: Option<Type>,
}
impl NoAssocTyOpBound {
    fn into_type_param_bound(self) -> TypeParamBound {
        type_param_bound(&self.trait_path, self.rhs, None)
    }

    fn with_assoc_ty_name(self, assoc_ty_name: Ident) -> AssocTyOpBound {
        AssocTyOpBound {
            trait_path: self.trait_path,
            rhs: self.rhs,
            assoc_ty_name,
        }
    }
}

pub(crate) fn ops_bounds_to_where_predicates(
    ops_bounds: impl IntoIterator<Item = OpsBound>,
) -> impl Iterator<Item = WherePredicate> {
    #[derive(Debug, Default)]
    struct Bounds {
        with_assoc: IndexMap<AssocTyOpBound, IndexSet<Type>>,
        no_assoc: IndexSet<NoAssocTyOpBound>,
    }

    let mut unique_bounds: IndexMap<Type, Bounds> = IndexMap::new();

    for ops_bound in ops_bounds {
        let sub_preds = match ops_bound {
            OpsBound::AssocTyOp(ops_bound) => {
                let preds = ops_bound.into_assoc_ty_op_predicates();

                let AssocTyOpPredicate {
                    bounded_ty,
                    bound,
                    assoc,
                } = preds.self_pred;
                let bounds = unique_bounds.entry(bounded_ty).or_default();
                let assocs = bounds.with_assoc.entry(bound).or_default();
                if let Some(assoc) = assoc {
                    assocs.insert(assoc);
                }

                AlternativeIterator::One(preds.sub_preds)
            }
            OpsBound::NoAssocTyOp(ops_bound) => {
                let preds = ops_bound.into_no_assoc_ty_op_predicates();

                let NoAssocTyOpPredicate { bounded_ty, bound } = preds.self_pred;
                let bounds = unique_bounds.entry(bounded_ty).or_default();
                bounds.no_assoc.insert(bound);

                AlternativeIterator::Two(preds.sub_preds)
            }
        };

        for preds in sub_preds {
            let bounds = unique_bounds.entry(preds.bounded_ty).or_default();
            bounds.with_assoc.entry(*preds.bound).or_default();
        }
    }

    unique_bounds.into_iter().map(|(bounded_ty, bounds)| {
        let assoc_bounds = bounds.with_assoc.into_iter().flat_map(|(bound, assocs)| {
            // Ensure there is at least one bound.
            let no_assocs = assocs
                .is_empty()
                .then(|| bound.clone().into_type_param_bound())
                .into_iter();

            let assocs = assocs
                .into_iter()
                .map(move |assoc| bound.clone().into_type_param_bound_with_assoc(assoc));

            no_assocs.chain(assocs)
        });

        let no_assoc_bounds = bounds
            .no_assoc
            .into_iter()
            .map(NoAssocTyOpBound::into_type_param_bound);

        where_predicate(&bounded_ty, assoc_bounds.chain(no_assoc_bounds))
    })
}

pub(crate) fn ops_expr_to_type(ops_expr: OpsExpr) -> Type {
    ops_expr.0.into_predicate().into_type()
}

/*
 *  | WherePredicate------------------------------------------- |
 *  | Type------ | ':' | TypeParamBound | ('+' TypeParamBound)* |
 *  |============|=====|================|=======================|
 *  | bounded_ty | ':' | type_param_bounds                      |
 */
fn where_predicate(
    bounded_ty: &Type,
    type_param_bounds: impl IntoIterator<Item = TypeParamBound>,
) -> WherePredicate {
    let type_param_bounds: Punctuated<TypeParamBound, Token![+]> =
        type_param_bounds.into_iter().collect();

    parse_quote! { #bounded_ty : #type_param_bounds }
}

/*
 *  | QSelf------------------------ |
 *  | '<' | Type------ | 'as' | '>' |
 *  |=====|============|======|=====|
 *  |     | bounded_ty |      |     |
 *
 *
 *  | Path¹------------------------------------------------------------------ |
 *  | ('::' PathSegment)* '::' | PathSegment---------- | '::' | PathSegment-- |
 *  |                          | Ident | PathArguments |      | Ident-------- |
 *  |==================================|===============|======|===============|
 *  | trait_path                       | path_args     |      | assoc_ty_name |
 *
 *
 *  QSelf and Path¹ are "mixed" into the output Type:
 *
 *  | Type----------------------------------------------------------------------------- |
 *  | TypePath = QSelf + Path¹ -------------------------------------------------------- |
 *  | '<' | Type------ | 'as' | Path------ | PathArguments | '>' | '::' | Ident-------- |
 *  |=====|============|======|============|===============|=====|======|===============|
 *  | '<' | bounded_ty | 'as' | trait_path | path_args     | '>' | '::' | assoc_ty_name |
 */
fn qualified_type(
    bounded_ty: &Type,
    trait_path: &Path,
    rhs: Option<Type>,
    assoc_ty_name: &Ident,
) -> Type {
    let path_args = path_arguments(rhs, None);

    parse_quote! { < #bounded_ty as #trait_path #path_args > :: #assoc_ty_name }
}

/*
 *  | TypeParamBound---------------------------------- |
 *  | TraitBound-------------------------------------- |
 *  | Path-------------------------------------------- |
 *  | ('::' PathSegment)* '::' | PathSegment---------- |
 *  |                          | Ident | PathArguments |
 *  |==================================|===============|
 *  | trait_path                       | path_args     |
 */
fn type_param_bound(
    trait_path: &Path,
    rhs: Option<Type>,
    assoc: Option<(Ident, Type)>,
) -> TypeParamBound {
    let path_args = path_arguments(rhs, assoc);

    parse_quote! { #trait_path #path_args }
}

/*
 *  | PathArguments------------------------------------------------------- |
 *  | AngleBracketedGenericArguments-------------------------------------- |
 *  | '<' | GenericArgument | ',' | GenericArgument----------------- | '>' |
 *  |     | Type----------- |     | AssocConst---------------------- |     |
 *  |     |                 |     | Ident-------- | '=' | Expr------ |     |
 *  |=====|=================|=====|===============|=====|============|=====|
 *  | '<' | rhs             | ',' | assoc_ty_name | '=' | assoc_pred | '>' |
 */
fn path_arguments(rhs: Option<Type>, assoc: Option<(Ident, Type)>) -> PathArguments {
    let rhs_arg = rhs.map(GenericArgument::Type);
    let assoc_arg = assoc.map(|(assoc_ty_name, assoc_pred)| {
        parse_quote! { #assoc_ty_name = #assoc_pred }
    });

    let args: Punctuated<_, Token![,]> = rhs_arg.into_iter().chain(assoc_arg).collect();

    if args.is_empty() {
        PathArguments::None
    } else {
        PathArguments::AngleBracketed(parse_quote! { < #args > })
    }
}

enum AlternativeIterator<I1, I2> {
    One(I1),
    Two(I2),
}
impl<I1, I2, T> Iterator for AlternativeIterator<I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AlternativeIterator::One(iter) => iter.next(),
            AlternativeIterator::Two(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;
    use crate::OpsBounds;

    macro_rules! preds_dbg {
        ($preds:expr) => {{
            std::fmt::from_fn(|f| {
                let mut preds: Vec<_> = $preds
                    .iter()
                    .map(|pred| {
                        pred.into_token_stream()
                            .to_string()
                            .replace(":: core :: ops :: ", "")
                            .replace(":: core :: cmp :: ", "")
                    })
                    .collect();
                preds.sort();

                writeln!(f)?;
                writeln!(f, "{}:", stringify!($preds))?;
                for pred in preds {
                    writeln!(f, "    {pred}")?;
                }
                Ok(())
            })
        }};
    }

    #[test]
    fn bounds_uniqueness() {
        let bounds: OpsBounds = parse_quote! {
            A * B + C / D,
            A * (B) + D,
            A > B,
            (A) < B,
            C / D = A * B,
            C / D = B * A,
        };

        let preds: Vec<_> = ops_bounds_to_where_predicates(bounds).collect();
        let unique_preds: IndexSet<_> = preds.iter().cloned().collect();
        assert_eq!(preds.len(), unique_preds.len());

        let expected: IndexSet<WherePredicate> = IndexSet::from([
            parse_quote! {
                A: ::core::ops::Mul<B>
                    + ::core::cmp::PartialOrd<B>
            },
            parse_quote! {
                <A as ::core::ops::Mul<B>>::Output:
                    ::core::ops::Add<<C as ::core::ops::Div<D>>::Output>
                    + ::core::ops::Add<D>
            },
            parse_quote! { B: ::core::ops::Mul<A> },
            parse_quote! {
                C: ::core::ops::Div<D, Output = <A as ::core::ops::Mul<B>>::Output>
                    + ::core::ops::Div<D, Output = <B as ::core::ops::Mul<A>>::Output>
            },
        ]);
        assert_eq!(
            unique_preds,
            expected,
            "{}{}",
            preds_dbg!(unique_preds),
            preds_dbg!(expected)
        );
    }
}

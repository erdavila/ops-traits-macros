mod operator;
mod output;

use std::collections::{HashMap, HashSet};
use std::iter::once;

use proc_macro::TokenStream;
use quote::ToTokens as _;
use syn::parse::{Lookahead1, Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Error, Item, Result, Token, Type, WherePredicate, parenthesized, parse_macro_input};

use crate::operator::{BinOp, CombineBinOp, NoAssocTyBinOp, Op, UnOp};
use crate::output::{qualified_type, where_predicate, where_predicate_with_assoc};

/*
    OpsBound := AssocTyOpOpsBound | NoAssocTyOpOperation
    OpsExpr := AssocTyOpOperation

    AssocTyOpOpsBound := AssocTyOpOperation AssocTyPredicate?
    AssocTyOpOperation := UnaryOperation | BinaryOperation<CombineBinOp>
    NoAssocTyOpOperation := BinaryOperation<NoAssocTyBinOp>

    UnaryOperation := UnOp Term
    BinaryOperation<O> := Term O RightOperand

    RightOperand := '_' | Term
    AssocTyPredicate := '=' Term
    Term := Type | '(' Type ')' | '(' AssocTyOpOperation ')'

    UnOp := '*' | '-' | '!'
    CombineBinOp := '+' | '-' | '*' | '/' | '%' | '<<' | '>>' | '&' | '^' | '|'
    NoAssocTyBinOp := '+=' | '-=' | '*=' | '/=' | '%=' | '<<=' | '>>=' | '&=' | '^=' | '|='

    BinOp := CombineBinOp | NoAssocTyBinOp
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
                let oper = bound.into_oper();

                unique_no_assoc_ty_op_bounds.insert(oper);
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
        .map(NoAssocTyOpOperation::into_where_predicate);

    assoc_ty_op_predicates.chain(no_assoc_ty_op_predicates)
}

#[proc_macro]
pub fn ops_type(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as OpsExpr)
        .into_type()
        .into_token_stream()
        .into()
}

/*
 *  OpsBound -> AssocTyOpOpsBound
 *           -> UnaryOperation AssocTyPredicate?
 *           -> unop:UnOp operand:Term
 *  => operand ':' trait(unop)
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> UnaryOperation AssocTyPredicate?
 *           -> unop:UnOp operand:Term '=' assoc
 *  => operand ':' trait(unop) '<' assoc_type_name(unop) '=' assoc '>'
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> BinaryOperation<CombineBinOp> AssocTyPredicate?
 *           -> left:Term binop:CombineBinOp '_'
 *  => left ':' trait(binop)
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> BinaryOperation<CombineBinOp> AssocTyPredicate?
 *           -> left:Term binop:CombineBinOp '_' '=' assoc
 *  => left ':' trait(binop) '<' assoc_type_name(binop) '=' assoc '>'
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> BinaryOperation<CombineBinOp> AssocTyPredicate?
 *           -> left:Term binop:CombineBinOp right:Term
 *  => left ':' trait(binop) '<' right '>'
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> BinaryOperation<CombineBinOp> AssocTyPredicate?
 *           -> left:Term binop:CombineBinOp right:Term '=' assoc
 *  => left ':' trait(binop) '<' right ',' assoc_type_name(binop) '=' assoc '>'
 *
 *  OpsBound -> BinaryOperation<NoAssocTyBinOp>
 *           -> left:Term binop:NoAssocTyBinOp '_'
 *  => left ':' trait(binop)
 *
 *  OpsBound -> BinaryOperation<NoAssocTyBinOp>
 *           -> left:Term binop:NoAssocTyBinOp right:Term
 *  => left ':' trait(binop) '<' right '>'
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OpsBound {
    AssocTyOp(AssocTyOpOpsBound),
    NoAssocTyOp(BinaryOperation<NoAssocTyBinOp>),
}
impl OpsBound {
    fn sub_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        let (assoc_ty_op_opers, no_assoc_ty_op_opers) = match self {
            OpsBound::AssocTyOp(bound) => (Some(bound.sub_opers()), None),
            OpsBound::NoAssocTyOp(bound) => (None, Some(bound.sub_opers())),
        };

        let assoc_ty_op_opers = assoc_ty_op_opers.into_iter().flatten();
        let no_assoc_ty_op_opers = no_assoc_ty_op_opers.into_iter().flatten();

        assoc_ty_op_opers.chain(no_assoc_ty_op_opers)
    }
}
impl Parse for OpsBound {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Some(un_operation) = UnaryOperation::try_parse(input, &input.lookahead1())? {
            Ok(OpsBound::AssocTyOp(AssocTyOpOpsBound {
                oper: AssocTyOpOperation::Unary(un_operation),
                assoc: AssocTyPredicate::try_parse(input, &input.lookahead1())?,
            }))
        } else {
            let bin_operation: BinaryOperation<BinOp> = input.parse()?;

            match bin_operation.binop {
                BinOp::Combine(binop) => Ok(OpsBound::AssocTyOp(AssocTyOpOpsBound {
                    oper: AssocTyOpOperation::Binary(BinaryOperation {
                        left: bin_operation.left,
                        binop,
                        right: bin_operation.right,
                    }),
                    assoc: AssocTyPredicate::try_parse(input, &input.lookahead1())?,
                })),
                BinOp::NoAssocTy(binop) => Ok(OpsBound::NoAssocTyOp(BinaryOperation {
                    left: bin_operation.left,
                    binop,
                    right: bin_operation.right,
                })),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssocTyOpOpsBound {
    oper: AssocTyOpOperation,
    assoc: Option<AssocTyPredicate>,
}
impl AssocTyOpOpsBound {
    fn sub_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        let expr_opers = self.oper.sub_opers();
        let assoc_opers = self
            .assoc
            .as_ref()
            .map(AssocTyPredicate::all_opers)
            .into_iter()
            .flatten();
        expr_opers.chain(assoc_opers)
    }
}

/*
 *  OpsExpr -> AssocTyOpOperation
 *          -> UnaryOperation
 *          -> unop:UnOp operand:Term
 *  => '<' operand 'as' trait(unop) '>' '::' assoc_type_name(unop)
 *
 *  OpsExpr -> AssocTyOpOperation
 *          -> BinaryOperation<CombineBinOp>
 *          -> left:Term binop:CombineBinOp '_'
 *  => '<' left 'as' trait(binop) '>' '::' assoc_type_name(binop)
 *
 *  OpsExpr -> AssocTyOpOperation
 *          -> BinaryOperation<CombineBinOp>
 *          -> left:Term binop:CombineBinOp right:Term
 *  => '<' left 'as' trait(binop) '<' right '>' '>' '::' assoc_type_name(binop)
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OpsExpr(AssocTyOpOperation);
impl OpsExpr {
    fn into_type(self) -> Type {
        self.0.into_type()
    }
}
impl Parse for OpsExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        AssocTyOpOperation::parse(input).map(OpsExpr)
    }
}
impl<T> From<T> for OpsExpr
where
    T: Into<AssocTyOpOperation>,
{
    fn from(value: T) -> Self {
        OpsExpr(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RightOperand {
    SelfRhs(Token![_]),
    Term(Box<Term>),
}
impl RightOperand {
    fn all_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        let opers = match self {
            RightOperand::SelfRhs(_) => None,
            RightOperand::Term(term) => Some(term.all_opers()),
        };

        opers.into_iter().flatten()
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
struct AssocTyPredicate {
    eq_token: Token![=],
    term: Term,
}
impl AssocTyPredicate {
    fn try_parse(input: ParseStream, lookahead: &Lookahead1) -> Result<Option<Self>> {
        if lookahead.peek(Token![=]) {
            Some(Self::parse(input)).transpose()
        } else {
            Ok(None)
        }
    }

    fn all_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        self.term.all_opers()
    }
}
impl Parse for AssocTyPredicate {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(AssocTyPredicate {
            eq_token: input.parse()?,
            term: input.parse()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Term {
    // Type | '(' Type ')'
    Type {
        paren: Option<Paren>,
        ty: Box<Type>,
    },
    // '(' AssocTyOpOperation ')'
    Group {
        paren: Paren,
        oper: Box<AssocTyOpOperation>,
    },
}
impl Term {
    fn all_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        let opers = match self {
            Term::Type { .. } => None,
            Term::Group { oper, .. } => Some(oper.all_opers()),
        };

        opers.into_iter().flatten()
    }

    fn into_type(self) -> Type {
        match self {
            Term::Type { ty, .. } => *ty,
            Term::Group { oper: expr, .. } => expr.into_type(),
        }
    }
}
impl Parse for Term {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Paren) {
            // '(' Type ')' | '(' AssocTyOpOperation ')'
            let content;
            let paren = parenthesized!(content in input);

            match content.parse()? {
                TypeOrAssocTyOpOperation::AssocTyOpOperation(oper) => {
                    // '(' AssocTyOpOperation ')'
                    Ok(Term::Group {
                        paren,
                        oper: Box::new(oper),
                    })
                }
                TypeOrAssocTyOpOperation::Type(ty) => {
                    // '(' Type ')'
                    Ok(Term::Type {
                        paren: Some(paren),
                        ty,
                    })
                }
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

/*
 *  Helper type to help parsing Term.
 *
 *  The rule
 *      Term := Type | '(' Type ')' | '(' AssocTyOpOperation ')'
 *  becomes
 *      Term := Type | '(' TypeOrAssocTyOpOperation ')'
 *
 *  Then:
 *      TypeOrAssocTyOpOperation := Type
 *                                | AssocTyOpOperation
 *         ->
 *      TypeOrAssocTyOpOperation := Type
 *                                | UnaryOperation
 *                                | BinaryOperation<CombineBinOp>
 *         ->
 *      TypeOrAssocTyOpOperation := Type
 *                                | UnOp Term
 *                                | Term CombineBinOp RightOperand
 *         ->
 *      TypeOrAssocTyOpOperation := UnOp Term
 *                                | '(' TypeOrAssocTyOpOperation ')' CombineBinOp RightOperand
 *                                | Type (CombineBinOp RightOperand)?
 */
enum TypeOrAssocTyOpOperation {
    AssocTyOpOperation(AssocTyOpOperation),
    Type(Box<Type>),
}
impl Parse for TypeOrAssocTyOpOperation {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if let Some(un_operation) = UnaryOperation::try_parse(input, &lookahead)? {
            // UnOp Term
            Ok(TypeOrAssocTyOpOperation::AssocTyOpOperation(
                un_operation.into(),
            ))
        } else if input.peek(Paren) {
            // '(' TypeOrAssocTyOpOperation ')' CombineBinOp RightOperand
            Ok(TypeOrAssocTyOpOperation::AssocTyOpOperation(input.parse()?))
        } else {
            let ty: Type = input.parse()?;
            if let Some(binop) = CombineBinOp::try_parse(input, &input.lookahead1())? {
                // Type CombineBinOp RightOperand
                let bin_operation = BinaryOperation {
                    left: Box::new(Term::from(ty)),
                    binop,
                    right: input.parse()?,
                };
                Ok(TypeOrAssocTyOpOperation::AssocTyOpOperation(
                    bin_operation.into(),
                ))
            } else {
                // Type
                Ok(TypeOrAssocTyOpOperation::Type(Box::new(ty)))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AssocTyOpOperation {
    Unary(UnaryOperation),
    Binary(BinaryOperation<CombineBinOp>),
}
impl AssocTyOpOperation {
    fn all_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        once(self.clone()).chain(self.sub_opers())
    }

    fn sub_opers(&self) -> Box<dyn Iterator<Item = AssocTyOpOperation>> {
        let (un_opers, bin_opers) = match self {
            AssocTyOpOperation::Unary(un_oper) => (Some(un_oper.sub_opers()), None),
            AssocTyOpOperation::Binary(bin_oper) => (None, Some(bin_oper.sub_opers())),
        };

        let un_opers = un_opers.into_iter().flatten();
        let bin_opers = bin_opers.into_iter().flatten();

        Box::new(un_opers.chain(bin_opers))
    }

    fn into_type(self) -> Type {
        match self {
            AssocTyOpOperation::Unary(un_oper) => un_oper.into_type(),
            AssocTyOpOperation::Binary(bin_oper) => bin_oper.into_type(),
        }
    }

    fn into_where_predicate(self) -> WherePredicate {
        match self {
            AssocTyOpOperation::Unary(un_oper) => un_oper.into_where_predicate(),
            AssocTyOpOperation::Binary(bin_oper) => bin_oper.into_where_predicate(),
        }
    }

    fn into_where_predicate_with_assoc(self, assoc: AssocTyPredicate) -> WherePredicate {
        match self {
            AssocTyOpOperation::Unary(un_oper) => un_oper.into_where_predicate_with_assoc(assoc),
            AssocTyOpOperation::Binary(bin_oper) => bin_oper.into_where_predicate_with_assoc(assoc),
        }
    }
}
impl Parse for AssocTyOpOperation {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        if let Some(un_oper) = UnaryOperation::try_parse(input, &lookahead)? {
            // UnOp Term
            Ok(AssocTyOpOperation::Unary(un_oper))
        } else {
            // Term CombineBinOp RightOperand
            Ok(AssocTyOpOperation::Binary(input.parse()?))
        }
    }
}
impl From<UnaryOperation> for AssocTyOpOperation {
    fn from(value: UnaryOperation) -> Self {
        AssocTyOpOperation::Unary(value)
    }
}
impl From<BinaryOperation<CombineBinOp>> for AssocTyOpOperation {
    fn from(value: BinaryOperation<CombineBinOp>) -> Self {
        AssocTyOpOperation::Binary(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NoAssocTyOpOperation(BinaryOperation<NoAssocTyBinOp>);
impl NoAssocTyOpOperation {
    fn into_where_predicate(self) -> WherePredicate {
        self.0.into_where_predicate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UnaryOperation {
    unop: UnOp,
    operand: Box<Term>,
}
impl UnaryOperation {
    fn try_parse(input: ParseStream, lookahead: &Lookahead1) -> Result<Option<Self>> {
        if let Some(unop) = UnOp::try_parse(input, lookahead)? {
            Ok(Some(UnaryOperation {
                unop,
                operand: input.parse()?,
            }))
        } else {
            Ok(None)
        }
    }

    fn sub_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<> {
        self.operand.all_opers()
    }

    fn into_where_predicate(self) -> WherePredicate {
        where_predicate(*self.operand, &self.unop, None)
    }

    fn into_where_predicate_with_assoc(self, assoc: AssocTyPredicate) -> WherePredicate {
        where_predicate_with_assoc(*self.operand, &self.unop, None, assoc)
    }

    fn into_type(self) -> Type {
        qualified_type(*self.operand, &self.unop, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BinaryOperation<O> {
    left: Box<Term>,
    binop: O,
    right: RightOperand,
}
impl<O: Op> BinaryOperation<O> {
    fn sub_opers(&self) -> impl Iterator<Item = AssocTyOpOperation> + use<O> {
        let left_opers = self.left.all_opers();
        let right_opers = self.right.all_opers();
        left_opers.chain(right_opers)
    }

    fn into_where_predicate(self) -> WherePredicate {
        where_predicate(*self.left, &self.binop, Some(self.right))
    }
}
impl BinaryOperation<CombineBinOp> {
    fn into_where_predicate_with_assoc(self, assoc: AssocTyPredicate) -> WherePredicate {
        where_predicate_with_assoc(*self.left, &self.binop, Some(self.right), assoc)
    }

    fn into_type(self) -> Type {
        qualified_type(*self.left, &self.binop, Some(self.right))
    }
}
impl BinaryOperation<NoAssocTyBinOp> {
    fn into_oper(self) -> NoAssocTyOpOperation {
        NoAssocTyOpOperation(self)
    }
}
impl<O: Parse> Parse for BinaryOperation<O> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(BinaryOperation {
            left: input.parse()?,
            binop: input.parse()?,
            right: input.parse()?,
        })
    }
}
impl From<BinaryOperation<CombineBinOp>> for BinaryOperation<BinOp> {
    fn from(BinaryOperation { left, binop, right }: BinaryOperation<CombineBinOp>) -> Self {
        BinaryOperation {
            left,
            binop: BinOp::Combine(binop),
            right,
        }
    }
}
impl From<BinaryOperation<NoAssocTyBinOp>> for BinaryOperation<BinOp> {
    fn from(BinaryOperation { left, binop, right }: BinaryOperation<NoAssocTyBinOp>) -> Self {
        BinaryOperation {
            left,
            binop: BinOp::NoAssocTy(binop),
            right,
        }
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

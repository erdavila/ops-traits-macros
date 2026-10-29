mod operator;
mod output;

use std::collections::{HashMap, HashSet};
use std::iter::once;

use proc_macro::TokenStream;
use quote::ToTokens as _;
use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Error, Item, Result, Token, Type, WherePredicate, parse_macro_input};

use crate::operator::{CombineBinOp, NoAssocTyBinOp, Op, UnOp};
use crate::output::{qualified_type, where_predicate, where_predicate_with_assoc};

/*
 *  # Input grammar
 *
 *  OpsBound := Operation ('=' Expression³)?
 *            | Expression³ NoAssocTyBinOp RightOperand³
 *  OpsExpr := Operation
 *
 *  Operation := Expression³   # Must not reduce to Type.
 *
 *  Expressionⁿ := Expressionⁿ⁺ⁱ                               # for n < 9
 *               | Expressionⁿ CombineBinOpⁿ RightOperandⁿ⁺ⁱ   # for n < 9
 *  Expression⁹ := UnOp Expression⁹ | Primary
 *
 *  RightOperandⁿ := '_' | Expressionⁿ
 *  Primary := Type | '(' Expression³ ')'
 *
 *  UnOp := UnOp⁹
 *  NoAssocTyBinOp := NoAssocTyBinOp¹ | NoAssocTyBinOp²
 *
 *  # Operators in order of precedence (https://doc.rust-lang.org/reference/expressions.html#expression-precedence):
 *  NoAssocTyBinOp¹ := '|=' | '^=' | '&=' | '<<=' | '>>=' | '+=' | '-=' | '*=' | '/=' | '%='
 *  NoAssocTyBinOp² := '<' | '>' | '<=' | '>=' | '==' | '!=' | '<=>'
 *  CombineBinOp³ := '|'
 *  CombineBinOp⁴ := '^'
 *  CombineBinOp⁵ := '&'
 *  CombineBinOp⁶ := '<<' | '>>'
 *  CombineBinOp⁷ := '+' | '-'
 *  CombineBinOp⁸ := '*' | '/' | '%'
 *  UnOp⁹ := '*' | '-' | '!'
 *
 *  -----------------------------------------------------------------------------
 *
 *  # Grammar with types used during parsing
 *
 *  OpsBound := AssocTyOpOpsBound | NoAssocTyOpOpsBound
 *  OpsExpr := Operation
 *
 *  AssocTyOpOpsBound := Operation AssocTyPredicate?
 *  NoAssocTyOpOpsBound := BinaryOperation<NoAssocTyBinOp>³˒³
 *
 *  Operation := Expression³   # Must reduce to UnaryOperation or BinaryOperation<CombineBinOp>.
 *  UnaryOperation := UnOp Expression⁹
 *  BinaryOperation<O>ᵐ˒ⁿ := Expressionᵐ O RightOperandⁿ
 *
 *  Expressionⁿ := Expressionⁿ⁺ⁱ | BinaryOperation<CombineBinOpⁿ>ⁿ˒ⁿ⁺ⁱ   # for n < 9
 *  Expression⁹ := Primary | UnaryOperation
 *  # General rule:
 *  Expressionˣ := Primary | UnaryOperation | BinaryOperation<CombineBinOpˣ>ˣ˒ˣ⁺ⁱ
 *
 *  AssocTyPredicate := '=' Expression³
 *  RightOperandⁿ := '_' | Expressionⁿ
 *  Primary := Type | '(' Expression³ ')'
 */

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

#[proc_macro]
pub fn ops_type(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as OpsExpr)
        .into_type()
        .into_token_stream()
        .into()
}

/*
 *  OpsBounds -> AssocTyOpOpsBound
 *            -> Operation AssocTyPredicate?
 *            -> UnaryOperation ('=' Expression³)?
 *            -> unop:UnOp operand:Expression⁹ ('=' assoc:Expression³)?
 *  => operand ':' trait(unop) ('<' assoc_type_name(unop) '=' assoc '>')?
 *
 *  OpsBounds -> AssocTyOpOpsBound
 *            -> Operation AssocTyPredicate?
 *            -> BinaryOperation<CombineBinOp> ('=' Expression³)?
 *            -> Expression CombineBinOp RightOperand ('=' Expression³)?
 *            -> left:Expression binop:CombineBinOp '_' ('=' assoc:Expression³)?
 *  => left ':' trait(binop) ('<' assoc_type_name(unop) '=' assoc '>')?
 *
 *  OpsBounds -> AssocTyOpOpsBound
 *            -> Operation AssocTyPredicate?
 *            -> BinaryOperation<CombineBinOp> ('=' Expression³)?
 *            -> Expression CombineBinOp RightOperand ('=' Expression³)?
 *            -> left:Expression binop:CombineBinOp right:Expression ('=' assoc:Expression³)?
 *  => left ':' trait(binop) '<' right (',' assoc_type_name(binop) '=' assoc '>')?
 *
 *  OpsBounds -> NoAssocTyOpOpsBound
 *            -> BinaryOperation<NoAssocTyBinOp>³˒³
 *            -> Expression³ NoAssocTyBinOp RightOperand³
 *            -> left:Expression³ binop:NoAssocTyBinOp '_'
 *  => left ':' trait(binop)
 *
 *  OpsBounds -> NoAssocTyOpOpsBound
 *            -> BinaryOperation<NoAssocTyBinOp>³˒³
 *            -> Expression³ NoAssocTyBinOp RightOperand³
 *            -> left:Expression³ binop:NoAssocTyBinOp right:Expression³
 *  => left ':' trait(binop) '<' right '>')?
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OpsBound {
    // OpsBound := Operation ('=' Expression³)?
    AssocTyOp(AssocTyOpOpsBound),
    // OpsBound := Expression³ NoAssocTyBinOp RightOperand³
    NoAssocTyOp(NoAssocTyOpOpsBound),
}
impl OpsBound {
    fn sub_opers(&self) -> impl Iterator<Item = Operation> + use<> {
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
        let left = parse::expression(input)?;

        if let Some(binop) = NoAssocTyBinOp::try_parse(input)? {
            // OpsBound := Expression³ NoAssocTyBinOp RightOperand³
            Ok(OpsBound::NoAssocTyOp(BinaryOperation {
                left: Box::new(left),
                binop,
                right: RightOperand::parse_with(input, parse::expression)?,
            }))
        } else {
            // OpsBound := Operation ('=' Expression³)?
            Ok(OpsBound::AssocTyOp(AssocTyOpOpsBound {
                oper: left.into_oper(input)?,
                assoc: AssocTyPredicate::try_parse(input)?,
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// AssocTyOpOpsBound := Operation AssocTyPredicate?
struct AssocTyOpOpsBound {
    oper: Operation,
    assoc: Option<AssocTyPredicate>,
}
impl AssocTyOpOpsBound {
    fn sub_opers(&self) -> impl Iterator<Item = Operation> + use<> {
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

// NoAssocTyOpOpsBound := BinaryOperation<NoAssocTyBinOp>³˒³
type NoAssocTyOpOpsBound = BinaryOperation<NoAssocTyBinOp>;

/*
 * OpsExpr -> Operation
 *         -> UnaryOperation
 *         -> unop:UnOp operand:Expression⁹
 *  => '<' operand 'as' trait(unop) '>' '::' assoc_type_name(unop)
 *
 * OpsExpr -> Operation
 *         -> BinaryOperation<CombineBinOp>
 *         -> Expression CombineBinOp RightOperand
 *         -> left:Expression binop:CombineBinOp '_'
 *  => '<' left 'as' trait(binop) '>' '::' assoc_type_name(binop)
 *
 * OpsExpr -> Operation
 *         -> BinaryOperation<CombineBinOp>
 *         -> left:Expression binop:CombineBinOp right:Expression
 *  => '<' left 'as' trait(binop) '<' right '>' '>' '::' assoc_type_name(binop)
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OpsExpr(Operation);
impl OpsExpr {
    fn into_type(self) -> Type {
        self.0.into_type()
    }
}
impl Parse for OpsExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = parse::expression(input)?;
        Ok(OpsExpr(expr.into_oper(input)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RightOperand {
    // RightOperandⁿ := '_'
    SelfRhs(Token![_]),
    // RightOperandⁿ := Expressionⁿ
    Expression(Box<Expression>),
}
impl RightOperand {
    fn parse_with(input: ParseStream, f: fn(ParseStream) -> Result<Expression>) -> Result<Self> {
        if input.peek(Token![_]) {
            // RightOperandⁿ := '_'
            Ok(RightOperand::SelfRhs(input.parse()?))
        } else {
            // RightOperandⁿ := Expressionⁿ
            Ok(RightOperand::Expression(Box::new(f(input)?)))
        }
    }

    fn all_opers(&self) -> impl Iterator<Item = Operation> + use<> {
        let opers = match self {
            RightOperand::SelfRhs(_) => None,
            RightOperand::Expression(expr) => Some(expr.all_opers()),
        };

        opers.into_iter().flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// AssocTyPredicate := '=' Expression³
struct AssocTyPredicate {
    eq_token: Token![=],
    expr: Expression,
}
impl AssocTyPredicate {
    fn try_parse(input: ParseStream) -> Result<Option<Self>> {
        if input.peek(Token![=]) {
            Ok(Some(AssocTyPredicate {
                eq_token: input.parse()?,
                expr: parse::expression(input)?,
            }))
        } else {
            Ok(None)
        }
    }

    fn all_opers(&self) -> impl Iterator<Item = Operation> + use<> {
        self.expr.all_opers()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Primary {
    // Primary := Type
    Type(Box<Type>),
    // Primary := '(' Expression³ ')'
    Expression {
        paren: Option<Paren>,
        expr: Box<Expression>,
    },
}
impl Primary {
    fn all_opers(&self) -> impl Iterator<Item = Operation> + use<> {
        let opers = match self {
            Primary::Type(_) => None,
            Primary::Expression { expr, .. } => Some(expr.all_opers()),
        };

        opers.into_iter().flatten()
    }

    fn into_type(self) -> Type {
        match self {
            Primary::Type(ty) => *ty,
            Primary::Expression { expr, .. } => expr.into_type(),
        }
    }
}
impl From<Type> for Primary {
    fn from(value: Type) -> Self {
        Primary::Type(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Expression {
    // Expressionˣ := Primary
    Primary(Primary),
    // Expressionˣ := UnaryOperation | BinaryOperation<CombineBinOpˣ>ˣ˒ˣ⁺ⁱ
    Operation(Operation),
}
impl Expression {
    fn into_oper(self, input: ParseStream) -> Result<Operation> {
        match self {
            Expression::Primary(_) => Err(input.lookahead1().error()),
            Expression::Operation(oper) => Ok(oper),
        }
    }

    fn all_opers(&self) -> Box<dyn Iterator<Item = Operation>> {
        let (prim, oper) = match self {
            Expression::Primary(primary) => (Some(primary.all_opers()), None),
            Expression::Operation(oper) => (None, Some(oper.all_opers())),
        };

        let prim_opers = prim.into_iter().flatten();
        let oper_opers = oper.into_iter().flatten();

        Box::new(prim_opers.chain(oper_opers))
    }

    fn into_type(self) -> Type {
        match self {
            Expression::Primary(primary) => primary.into_type(),
            Expression::Operation(oper) => oper.into_type(),
        }
    }
}
impl<T: Into<Operation>> From<T> for Expression {
    fn from(value: T) -> Self {
        Expression::Operation(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Operation {
    // Expressionˣ := UnaryOperation
    Unary(UnaryOperation),
    // Expressionˣ := BinaryOperation<CombineBinOpˣ>ˣ˒ˣ⁺ⁱ
    Binary(BinaryOperation<CombineBinOp>),
}
impl Operation {
    fn all_opers(&self) -> impl Iterator<Item = Operation> + use<> {
        once(self.clone()).chain(self.sub_opers())
    }

    fn sub_opers(&self) -> impl Iterator<Item = Operation> + use<> {
        let (un_opers, bin_opers) = match self {
            Operation::Unary(un_oper) => (Some(un_oper.sub_opers()), None),
            Operation::Binary(bin_oper) => (None, Some(bin_oper.sub_opers())),
        };

        let un_opers = un_opers.into_iter().flatten();
        let bin_opers = bin_opers.into_iter().flatten();

        un_opers.chain(bin_opers)
    }

    fn into_type(self) -> Type {
        match self {
            Operation::Unary(un_oper) => un_oper.into_type(),
            Operation::Binary(bin_oper) => bin_oper.into_type(),
        }
    }

    fn into_where_predicate(self) -> WherePredicate {
        match self {
            Operation::Unary(un_oper) => un_oper.into_where_predicate(),
            Operation::Binary(bin_oper) => bin_oper.into_where_predicate(),
        }
    }

    fn into_where_predicate_with_assoc(self, assoc: AssocTyPredicate) -> WherePredicate {
        match self {
            Operation::Unary(un_oper) => un_oper.into_where_predicate_with_assoc(assoc),
            Operation::Binary(bin_oper) => bin_oper.into_where_predicate_with_assoc(assoc),
        }
    }
}
impl From<UnaryOperation> for Operation {
    fn from(value: UnaryOperation) -> Self {
        Operation::Unary(value)
    }
}
impl From<BinaryOperation<CombineBinOp>> for Operation {
    fn from(value: BinaryOperation<CombineBinOp>) -> Self {
        Operation::Binary(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// UnaryOperation := UnOp Expression⁹
struct UnaryOperation {
    unop: UnOp,
    operand: Box<Expression>,
}
impl UnaryOperation {
    fn sub_opers(&self) -> impl Iterator<Item = Operation> + use<> {
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
// BinaryOperation<O>ᵐ˒ⁿ := Expressionᵐ O RightOperandⁿ
struct BinaryOperation<O> {
    left: Box<Expression>,
    binop: O,
    right: RightOperand,
}
impl<O: Op> BinaryOperation<O> {
    fn sub_opers(&self) -> impl Iterator<Item = Operation> + use<O> {
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

mod parse {
    use syn::parse::ParseStream;
    use syn::token::Paren;
    use syn::{Result, Type, parenthesized};

    use crate::{
        BinaryOperation, CombineBinOp, Expression, Primary, RightOperand, UnOp, UnaryOperation,
    };

    pub(crate) fn expression(input: ParseStream) -> Result<Expression> {
        bitor_oper(input)
    }

    fn bitor_oper(input: ParseStream) -> Result<Expression> {
        binop_oper(input, CombineBinOp::try_parse_bitor, bitxor_oper)
    }

    fn bitxor_oper(input: ParseStream) -> Result<Expression> {
        binop_oper(input, CombineBinOp::try_parse_bitxor, bitand_oper)
    }

    fn bitand_oper(input: ParseStream) -> Result<Expression> {
        binop_oper(input, CombineBinOp::try_parse_bitand, shl_shr_oper)
    }

    fn shl_shr_oper(input: ParseStream) -> Result<Expression> {
        binop_oper(input, CombineBinOp::try_parse_shl_shr, add_sub_oper)
    }

    fn add_sub_oper(input: ParseStream) -> Result<Expression> {
        binop_oper(input, CombineBinOp::try_parse_add_sub, mul_div_rem_oper)
    }

    fn mul_div_rem_oper(input: ParseStream) -> Result<Expression> {
        binop_oper(input, CombineBinOp::try_parse_mul_div_rem, unop_oper)
    }

    fn binop_oper(
        input: ParseStream,
        try_parse_op: fn(ParseStream) -> Result<Option<CombineBinOp>>,
        next_level: fn(ParseStream) -> Result<Expression>,
    ) -> Result<Expression> {
        // Expressionⁿ := Expressionⁿ⁺ⁱ
        let mut left = next_level(input)?;

        while let Some(binop) = try_parse_op(input)? {
            // Expressionⁿ := Expressionⁿ CombineBinOpⁿ RightOperandⁿ⁺ⁱ
            left = Expression::from(BinaryOperation {
                left: Box::new(left),
                binop,
                right: RightOperand::parse_with(input, next_level)?,
            });
        }

        Ok(left)
    }

    fn unop_oper(input: ParseStream) -> Result<Expression> {
        if let Some(unop) = UnOp::try_parse(input)? {
            // Expression⁹ := UnOp Expression⁹
            Ok(Expression::from(UnaryOperation {
                unop,
                operand: Box::new(unop_oper(input)?),
            }))
        } else {
            // Expression⁹ := Primary
            Ok(Expression::Primary(primary(input)?))
        }
    }

    fn primary(input: ParseStream) -> Result<Primary> {
        if input.peek(Paren) {
            // Primary := '(' Expression³ ')'
            let content;
            let paren = parenthesized!(content in input);
            Ok(Primary::Expression {
                paren: Some(paren),
                expr: Box::new(expression(&content)?),
            })
        } else {
            // Primary := Type
            let ty = input.call(Type::without_plus)?;
            Ok(ty.into())
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

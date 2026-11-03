/*
 *  # Input grammar
 *
 *  OpsBound := Operation ('=' Expression³)?
 *            | Expression³ NoAssocTyBinOp RightOperand³
 *            | Expression³ ':' TypeParamBound ('+' TypeParamBound)*
 *  OpsExpr := Expression³
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
 *  OpsBound := AssocTyOpOpsBound | NoAssocTyOpOpsBound | ExprBound
 *  OpsExpr := Expression³
 *
 *  AssocTyOpOpsBound := Operation AssocTyPredicate?
 *  NoAssocTyOpOpsBound := BinaryOperation<NoAssocTyBinOp>³˒³
 *  ExprBound := Expression³ ':' TypeParamBound ('+' TypeParamBound)*
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

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{Result, Token, Type, TypeParamBound};

use crate::{CombineBinOp, NoAssocTyBinOp, TryParse, UnOp, parse};

/*
 *  OpsBound -> AssocTyOpOpsBound
 *           -> Operation AssocTyPredicate?
 *           -> UnaryOperation ('=' Expression³)?
 *           -> unop:UnOp operand:Expression⁹ ('=' assoc:Expression³)?
 *  => operand ':' trait(unop) ('<' assoc_type_name(unop) '=' assoc '>')?
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> Operation AssocTyPredicate?
 *           -> BinaryOperation<CombineBinOp> ('=' Expression³)?
 *           -> Expression CombineBinOp RightOperand ('=' Expression³)?
 *           -> left:Expression binop:CombineBinOp '_' ('=' assoc:Expression³)?
 *  => left ':' trait(binop) ('<' assoc_type_name(unop) '=' assoc '>')?
 *
 *  OpsBound -> AssocTyOpOpsBound
 *           -> Operation AssocTyPredicate?
 *           -> BinaryOperation<CombineBinOp> ('=' Expression³)?
 *           -> Expression CombineBinOp RightOperand ('=' Expression³)?
 *           -> left:Expression binop:CombineBinOp right:Expression ('=' assoc:Expression³)?
 *  => left ':' trait(binop) '<' right (',' assoc_type_name(binop) '=' assoc '>')?
 *
 *  OpsBound -> NoAssocTyOpOpsBound
 *           -> BinaryOperation<NoAssocTyBinOp>³˒³
 *           -> Expression³ NoAssocTyBinOp RightOperand³
 *           -> left:Expression³ binop:NoAssocTyBinOp '_'
 *  => left ':' trait(binop)
 *
 *  OpsBound -> NoAssocTyOpOpsBound
 *           -> BinaryOperation<NoAssocTyBinOp>³˒³
 *           -> Expression³ NoAssocTyBinOp RightOperand³
 *           -> left:Expression³ binop:NoAssocTyBinOp right:Expression³
 *  => left ':' trait(binop) '<' right '>'
 *
 *  OpsBound -> NoAssocTyOpOpsBound
 *           -> ExprBound
 *           -> bounded:Expression³ ':' bounds:(TypeParamBound ('+' TypeParamBound)*))
 *  => bounded ':' bounds
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum OpsBound {
    // OpsBound := Operation ('=' Expression³)?
    AssocTyOp(AssocTyOpOpsBound),
    // OpsBound := Expression³ NoAssocTyBinOp RightOperand³
    NoAssocTyOp(NoAssocTyOpOpsBound),
    // OpsBound := Expression³ ':' TypeParamBound ('+' TypeParamBound)*
    Expr(ExprBound),
}
impl Parse for OpsBound {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = parse::expression(input)?;

        if input.peek(Token![:]) {
            // OpsBound := Expression³ ':' TypeParamBound ('+' TypeParamBound)*
            Ok(OpsBound::Expr(ExprBound {
                bounded: expr,
                colon: input.parse()?,
                std_bounds: Punctuated::parse_separated_nonempty(input)?,
            }))
        } else if let Some(binop) = NoAssocTyBinOp::try_parse(input)? {
            // OpsBound := Expression³ NoAssocTyBinOp RightOperand³
            Ok(OpsBound::NoAssocTyOp(NoAssocTyOpOpsBound(
                BinaryOperation {
                    left: Box::new(expr),
                    binop,
                    right: RightOperand::parse_with(input, parse::expression)?,
                },
            )))
        } else {
            // OpsBound := Operation ('=' Expression³)?
            Ok(OpsBound::AssocTyOp(AssocTyOpOpsBound {
                oper: expr.into_oper(input)?,
                assoc: AssocTyPredicate::try_parse(input)?,
            }))
        }
    }
}

// An operation that MAY have an associated type predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// AssocTyOpOpsBound := Operation AssocTyPredicate?
pub(crate) struct AssocTyOpOpsBound {
    pub(crate) oper: Operation,
    pub(crate) assoc: Option<AssocTyPredicate>,
}

// An operation that does NOT have an associated type predicate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// NoAssocTyOpOpsBound := BinaryOperation<NoAssocTyBinOp>³˒³
pub(crate) struct NoAssocTyOpOpsBound(pub(crate) BinaryOperation<NoAssocTyBinOp>);

// A standard Rust bound applied to the type of an expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// Expression³ ':' TypeParamBound ('+' TypeParamBound)*
pub(crate) struct ExprBound {
    pub(crate) bounded: Expression,
    colon: Token![:],
    pub(crate) std_bounds: Punctuated<TypeParamBound, Token![+]>,
}

/*
 * OpsExpr -> Expression³
 *         -> UnaryOperation
 *         -> unop:UnOp operand:Expression⁹
 *  => '<' operand 'as' trait(unop) '>' '::' assoc_type_name(unop)
 *
 * OpsExpr -> Expression³
 *         -> BinaryOperation<CombineBinOp>
 *         -> Expression CombineBinOp RightOperand
 *         -> left:Expression binop:CombineBinOp '_'
 *  => '<' left 'as' trait(binop) '>' '::' assoc_type_name(binop)
 *
 * OpsExpr -> Expression³
 *         -> BinaryOperation<CombineBinOp>
 *         -> left:Expression binop:CombineBinOp right:Expression
 *  => '<' left 'as' trait(binop) '<' right '>' '>' '::' assoc_type_name(binop)
 *
 * OpsExpr -> Expression³
 *         -> Primary
 *         -> type:Type
 *  => type
 *
 * OpsExpr -> Expression³
 *         -> Primary
 *         -> '(' Expression³ ')'
 *         -> ...
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct OpsExpr(pub(crate) Expression);
impl Parse for OpsExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr = parse::expression(input)?;
        Ok(OpsExpr(expr))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum RightOperand {
    // RightOperandⁿ := '_'
    SelfRhs(Token![_]),
    // RightOperandⁿ := Expressionⁿ
    Expression(Box<Expression>),
}
impl RightOperand {
    pub(crate) fn parse_with(
        input: ParseStream,
        f: fn(ParseStream) -> Result<Expression>,
    ) -> Result<Self> {
        if input.peek(Token![_]) {
            // RightOperandⁿ := '_'
            Ok(RightOperand::SelfRhs(input.parse()?))
        } else {
            // RightOperandⁿ := Expressionⁿ
            Ok(RightOperand::Expression(Box::new(f(input)?)))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// AssocTyPredicate := '=' Expression³
pub(crate) struct AssocTyPredicate {
    pub(crate) eq_token: Token![=],
    pub(crate) expr: Expression,
}
impl TryParse for AssocTyPredicate {
    fn try_parse(input: ParseStream) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        if input.peek(Token![=]) {
            Ok(Some(AssocTyPredicate {
                eq_token: input.parse()?,
                expr: parse::expression(input)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Primary {
    // Primary := Type
    Type(Box<Type>),
    // Primary := '(' Expression³ ')'
    Expression {
        paren: Option<Paren>,
        expr: Box<Expression>,
    },
}
impl From<Type> for Primary {
    fn from(value: Type) -> Self {
        Primary::Type(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Expression {
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
}
impl<T: Into<Operation>> From<T> for Expression {
    fn from(value: T) -> Self {
        Expression::Operation(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Operation {
    // Expressionˣ := UnaryOperation
    Unary(UnaryOperation),
    // Expressionˣ := BinaryOperation<CombineBinOpˣ>ˣ˒ˣ⁺ⁱ
    Binary(BinaryOperation<CombineBinOp>),
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
pub(crate) struct UnaryOperation {
    pub(crate) unop: UnOp,
    pub(crate) operand: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
// BinaryOperation<O>ᵐ˒ⁿ := Expressionᵐ O RightOperandⁿ
pub(crate) struct BinaryOperation<O> {
    pub(crate) left: Box<Expression>,
    pub(crate) binop: O,
    pub(crate) right: RightOperand,
}

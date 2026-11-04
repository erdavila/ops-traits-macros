use syn::parse::ParseStream;
use syn::token::Paren;
use syn::{Result, Type, parenthesized};

use crate::grammar::{
    BinaryOperation, Expression, Primary, RightOperand, SqrOperation, UnaryOperation,
};
use crate::operator::SqrOp;
use crate::{CombineBinOp, UnOp};

pub(crate) trait TryParse {
    fn try_parse(input: ParseStream) -> Result<Option<Self>>
    where
        Self: Sized;
}

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
    let mut expr = next_level(input)?;

    while let Some(binop) = try_parse_op(input)? {
        // Expressionⁿ := Expressionⁿ CombineBinOpⁿ RightOperandⁿ⁺ⁱ
        expr = Expression::from(BinaryOperation {
            left: Box::new(expr),
            binop,
            right: RightOperand::parse_with(input, next_level)?,
        });
    }

    Ok(expr)
}

fn unop_oper(input: ParseStream) -> Result<Expression> {
    if let Some(unop) = UnOp::try_parse(input)? {
        // Expression⁹ := UnOp Expression⁹
        Ok(Expression::from(UnaryOperation {
            unop,
            operand: Box::new(unop_oper(input)?),
        }))
    } else {
        // Expression⁹ := Expression¹⁰
        Ok(sqrop_oper(input)?)
    }
}

fn sqrop_oper(input: ParseStream) -> Result<Expression> {
    // Expression¹⁰ := Primary
    let mut expr = Expression::Primary(primary(input)?);

    while let Some(sqrop) = SqrOp::try_parse(input)? {
        // Expression¹⁰ := Expression¹⁰ SqrOp
        expr = Expression::from(SqrOperation {
            operand: Box::new(expr),
            sqrop,
        });
    }

    Ok(expr)
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

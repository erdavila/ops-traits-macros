use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Ident, Path, Result, Token, parse_quote_spanned};

pub(crate) trait Op {
    fn trait_path(&self) -> Path;
}

pub(crate) trait AssocTyOp: Op {
    fn assoc_type_name(&self) -> Ident;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum UnOp {
    Deref(Token![*]),
    Neg(Token![-]),
    Not(Token![!]),
}
impl UnOp {
    pub(crate) fn try_parse(input: ParseStream, lookahead: &Lookahead1) -> Result<Option<Self>> {
        if lookahead.peek(Token![*]) {
            Ok(Some(UnOp::Deref(input.parse()?)))
        } else if lookahead.peek(Token![-]) {
            Ok(Some(UnOp::Neg(input.parse()?)))
        } else if lookahead.peek(Token![!]) {
            Ok(Some(UnOp::Not(input.parse()?)))
        } else {
            Ok(None)
        }
    }
}
impl Op for UnOp {
    fn trait_path(&self) -> Path {
        match self {
            UnOp::Deref(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Deref },
            UnOp::Neg(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Neg },
            UnOp::Not(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Not },
        }
    }
}
impl AssocTyOp for UnOp {
    fn assoc_type_name(&self) -> Ident {
        match self {
            UnOp::Deref(tok) => parse_quote_spanned! { tok.span() => Target },
            UnOp::Neg(tok) => parse_quote_spanned! { tok.span() => Output },
            UnOp::Not(tok) => parse_quote_spanned! { tok.span() => Output },
        }
    }
}
impl Parse for UnOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        Self::try_parse(input, &lookahead)?.ok_or_else(|| lookahead.error())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum CombineBinOp {
    Add(Token![+]),
    Sub(Token![-]),
    Mul(Token![*]),
    Div(Token![/]),
    Rem(Token![%]),
    Shl(Token![<<]),
    Shr(Token![>>]),
    BitAnd(Token![&]),
    BitXor(Token![^]),
    BitOr(Token![|]),
}
impl CombineBinOp {
    pub(crate) fn try_parse(input: ParseStream, lookahead: &Lookahead1) -> Result<Option<Self>> {
        if lookahead.peek(Token![+]) {
            Ok(Some(CombineBinOp::Add(input.parse()?)))
        } else if lookahead.peek(Token![-]) {
            Ok(Some(CombineBinOp::Sub(input.parse()?)))
        } else if lookahead.peek(Token![*]) {
            Ok(Some(CombineBinOp::Mul(input.parse()?)))
        } else if lookahead.peek(Token![/]) {
            Ok(Some(CombineBinOp::Div(input.parse()?)))
        } else if lookahead.peek(Token![%]) {
            Ok(Some(CombineBinOp::Rem(input.parse()?)))
        } else if lookahead.peek(Token![<<]) {
            Ok(Some(CombineBinOp::Shl(input.parse()?)))
        } else if lookahead.peek(Token![>>]) {
            Ok(Some(CombineBinOp::Shr(input.parse()?)))
        } else if lookahead.peek(Token![&]) {
            Ok(Some(CombineBinOp::BitAnd(input.parse()?)))
        } else if lookahead.peek(Token![^]) {
            Ok(Some(CombineBinOp::BitXor(input.parse()?)))
        } else if lookahead.peek(Token![|]) {
            Ok(Some(CombineBinOp::BitOr(input.parse()?)))
        } else {
            Ok(None)
        }
    }
}
impl Op for CombineBinOp {
    fn trait_path(&self) -> Path {
        match self {
            CombineBinOp::Add(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Add },
            CombineBinOp::Sub(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Sub },
            CombineBinOp::Mul(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Mul },
            CombineBinOp::Div(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Div },
            CombineBinOp::Rem(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Rem },
            CombineBinOp::Shl(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Shl },
            CombineBinOp::Shr(tok) => parse_quote_spanned! { tok.span() => ::core::ops::Shr },
            CombineBinOp::BitAnd(tok) => parse_quote_spanned! { tok.span() => ::core::ops::BitAnd },
            CombineBinOp::BitXor(tok) => parse_quote_spanned! { tok.span() => ::core::ops::BitXor },
            CombineBinOp::BitOr(tok) => parse_quote_spanned! { tok.span() => ::core::ops::BitOr },
        }
    }
}
impl AssocTyOp for CombineBinOp {
    fn assoc_type_name(&self) -> Ident {
        match self {
            CombineBinOp::Add(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::Sub(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::Mul(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::Div(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::Rem(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::Shl(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::Shr(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::BitAnd(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::BitXor(tok) => parse_quote_spanned! { tok.span() => Output },
            CombineBinOp::BitOr(tok) => parse_quote_spanned! { tok.span() => Output },
        }
    }
}
impl Parse for CombineBinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        Self::try_parse(input, &lookahead)?.ok_or_else(|| lookahead.error())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum NoAssocTyBinOp {
    Add(Token![+=]),
    Sub(Token![-=]),
    Mul(Token![*=]),
    Div(Token![/=]),
    Rem(Token![%=]),
    Shl(Token![<<=]),
    Shr(Token![>>=]),
    BitAnd(Token![&=]),
    BitXor(Token![^=]),
    BitOr(Token![|=]),
}
impl NoAssocTyBinOp {
    fn try_parse(input: ParseStream, lookahead: &Lookahead1) -> Result<Option<Self>> {
        if lookahead.peek(Token![+=]) {
            Ok(Some(NoAssocTyBinOp::Add(input.parse()?)))
        } else if lookahead.peek(Token![-=]) {
            Ok(Some(NoAssocTyBinOp::Sub(input.parse()?)))
        } else if lookahead.peek(Token![*=]) {
            Ok(Some(NoAssocTyBinOp::Mul(input.parse()?)))
        } else if lookahead.peek(Token![/=]) {
            Ok(Some(NoAssocTyBinOp::Div(input.parse()?)))
        } else if lookahead.peek(Token![%=]) {
            Ok(Some(NoAssocTyBinOp::Rem(input.parse()?)))
        } else if lookahead.peek(Token![<<=]) {
            Ok(Some(NoAssocTyBinOp::Shl(input.parse()?)))
        } else if lookahead.peek(Token![>>=]) {
            Ok(Some(NoAssocTyBinOp::Shr(input.parse()?)))
        } else if lookahead.peek(Token![&=]) {
            Ok(Some(NoAssocTyBinOp::BitAnd(input.parse()?)))
        } else if lookahead.peek(Token![^=]) {
            Ok(Some(NoAssocTyBinOp::BitXor(input.parse()?)))
        } else if lookahead.peek(Token![|=]) {
            Ok(Some(NoAssocTyBinOp::BitOr(input.parse()?)))
        } else {
            Ok(None)
        }
    }
}
impl Op for NoAssocTyBinOp {
    fn trait_path(&self) -> Path {
        match self {
            NoAssocTyBinOp::Add(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::AddAssign }
            }
            NoAssocTyBinOp::Sub(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::SubAssign }
            }
            NoAssocTyBinOp::Mul(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::MulAssign }
            }
            NoAssocTyBinOp::Div(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::DivAssign }
            }
            NoAssocTyBinOp::Rem(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::RemAssign }
            }
            NoAssocTyBinOp::Shl(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::ShlAssign }
            }
            NoAssocTyBinOp::Shr(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::ShrAssign }
            }
            NoAssocTyBinOp::BitAnd(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::BitAndAssign }
            }
            NoAssocTyBinOp::BitXor(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::BitXorAssign }
            }
            NoAssocTyBinOp::BitOr(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::BitOrAssign }
            }
        }
    }
}
impl Parse for NoAssocTyBinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        Self::try_parse(input, &lookahead)?.ok_or_else(|| lookahead.error())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BinOp {
    Combine(CombineBinOp),
    NoAssocTy(NoAssocTyBinOp),
}
impl Op for BinOp {
    fn trait_path(&self) -> Path {
        match self {
            BinOp::Combine(binop) => binop.trait_path(),
            BinOp::NoAssocTy(binop) => binop.trait_path(),
        }
    }
}
impl Parse for BinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if let Some(binop) = NoAssocTyBinOp::try_parse(input, &lookahead)? {
            Ok(BinOp::NoAssocTy(binop))
        } else if let Some(binop) = CombineBinOp::try_parse(input, &lookahead)? {
            Ok(BinOp::Combine(binop))
        } else {
            Err(lookahead.error())
        }
    }
}

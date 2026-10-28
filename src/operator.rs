use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::spanned::Spanned as _;
use syn::{Ident, Path, Result, Token, custom_punctuation, parse_quote_spanned};

custom_punctuation!(Cmp, <=>);

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
    AddAssign(Token![+=]),
    SubAssign(Token![-=]),
    MulAssign(Token![*=]),
    DivAssign(Token![/=]),
    RemAssign(Token![%=]),
    ShlAssign(Token![<<=]),
    ShrAssign(Token![>>=]),
    BitAndAssign(Token![&=]),
    BitXorAssign(Token![^=]),
    BitOrAssign(Token![|=]),
    PartialEqEq(Token![==]),
    PartialEqNe(Token![!=]),
    PartialOrdCmp(Cmp),
    PartialOrdLt(Token![<]),
    PartialOrdGt(Token![>]),
    PartialOrdLe(Token![<=]),
    PartialOrdGe(Token![>=]),
}
impl NoAssocTyBinOp {
    fn try_parse(input: ParseStream, lookahead: &Lookahead1) -> Result<Option<Self>> {
        if lookahead.peek(Token![+=]) {
            Ok(Some(NoAssocTyBinOp::AddAssign(input.parse()?)))
        } else if lookahead.peek(Token![-=]) {
            Ok(Some(NoAssocTyBinOp::SubAssign(input.parse()?)))
        } else if lookahead.peek(Token![*=]) {
            Ok(Some(NoAssocTyBinOp::MulAssign(input.parse()?)))
        } else if lookahead.peek(Token![/=]) {
            Ok(Some(NoAssocTyBinOp::DivAssign(input.parse()?)))
        } else if lookahead.peek(Token![%=]) {
            Ok(Some(NoAssocTyBinOp::RemAssign(input.parse()?)))
        } else if lookahead.peek(Token![<<=]) {
            Ok(Some(NoAssocTyBinOp::ShlAssign(input.parse()?)))
        } else if lookahead.peek(Token![>>=]) {
            Ok(Some(NoAssocTyBinOp::ShrAssign(input.parse()?)))
        } else if lookahead.peek(Token![&=]) {
            Ok(Some(NoAssocTyBinOp::BitAndAssign(input.parse()?)))
        } else if lookahead.peek(Token![^=]) {
            Ok(Some(NoAssocTyBinOp::BitXorAssign(input.parse()?)))
        } else if lookahead.peek(Token![|=]) {
            Ok(Some(NoAssocTyBinOp::BitOrAssign(input.parse()?)))
        } else if lookahead.peek(Token![==]) {
            Ok(Some(NoAssocTyBinOp::PartialEqEq(input.parse()?)))
        } else if lookahead.peek(Token![!=]) {
            Ok(Some(NoAssocTyBinOp::PartialEqNe(input.parse()?)))
        } else if lookahead.peek(Cmp) {
            Ok(Some(NoAssocTyBinOp::PartialOrdCmp(input.parse()?)))
        } else if lookahead.peek(Token![<=]) {
            Ok(Some(NoAssocTyBinOp::PartialOrdLe(input.parse()?)))
        } else if lookahead.peek(Token![>=]) {
            Ok(Some(NoAssocTyBinOp::PartialOrdGe(input.parse()?)))
        } else if lookahead.peek(Token![<]) && !lookahead.peek(Token![<<]) {
            Ok(Some(NoAssocTyBinOp::PartialOrdLt(input.parse()?)))
        } else if lookahead.peek(Token![>]) && !lookahead.peek(Token![>>]) {
            Ok(Some(NoAssocTyBinOp::PartialOrdGt(input.parse()?)))
        } else {
            Ok(None)
        }
    }
}
impl Op for NoAssocTyBinOp {
    fn trait_path(&self) -> Path {
        match self {
            NoAssocTyBinOp::AddAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::AddAssign }
            }
            NoAssocTyBinOp::SubAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::SubAssign }
            }
            NoAssocTyBinOp::MulAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::MulAssign }
            }
            NoAssocTyBinOp::DivAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::DivAssign }
            }
            NoAssocTyBinOp::RemAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::RemAssign }
            }
            NoAssocTyBinOp::ShlAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::ShlAssign }
            }
            NoAssocTyBinOp::ShrAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::ShrAssign }
            }
            NoAssocTyBinOp::BitAndAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::BitAndAssign }
            }
            NoAssocTyBinOp::BitXorAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::BitXorAssign }
            }
            NoAssocTyBinOp::BitOrAssign(tok) => {
                parse_quote_spanned! { tok.span() => ::core::ops::BitOrAssign }
            }
            NoAssocTyBinOp::PartialEqEq(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialEq }
            }
            NoAssocTyBinOp::PartialEqNe(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialEq }
            }
            NoAssocTyBinOp::PartialOrdCmp(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialOrd }
            }
            NoAssocTyBinOp::PartialOrdLt(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialOrd }
            }
            NoAssocTyBinOp::PartialOrdGt(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialOrd }
            }
            NoAssocTyBinOp::PartialOrdLe(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialOrd }
            }
            NoAssocTyBinOp::PartialOrdGe(tok) => {
                parse_quote_spanned! { tok.span() => ::core::cmp::PartialOrd }
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

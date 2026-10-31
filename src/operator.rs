use proc_macro2::Span;
use syn::parse::ParseStream;
use syn::spanned::Spanned as _;
use syn::{Error, Ident, Path, Result, Token, custom_punctuation, parse_quote_spanned};

use crate::TryParse;

macro_rules! try_parse_token {
    (
        $input:expr,
        $enum:ty
        {
            $(
                $token:tt $(except $($not:tt),+)? => $variant:ident
            ),+
            $(,)?
        }
    ) => {{
        let input = $input;
        $(
            if input.peek(try_parse_token!(@ $token))
                $(
                    $( && !input.peek(try_parse_token!(@ $not)) )+
                )?
            {
                Ok(Some(<$enum>::$variant(input.parse()?)))
            } else
        )+
        {
            Ok::<_, Error>(None)
        }
    }};

    (@ [$tok:tt]) => { Token![$tok] };

    (@ $tok:tt) => { $tok };
}

pub(crate) trait Op {
    fn trait_path(&self) -> Path;
}

macro_rules! impl_op {
    ($enum:ty { $($variant:ident => $trait_path:path),+ $(,)? }) => {
        impl Op for $enum {
            fn trait_path(&self) -> Path {
                match self {
                    $(
                        Self::$variant(tok) => parse_quote_spanned! { tok.span() => $trait_path },
                    )+
                }
            }
        }
    };
}

pub(crate) trait AssocTyOp: Op {
    fn assoc_type_name(&self) -> Ident;
}

macro_rules! impl_assoc_ty_op {
    ($enum:ty { $($variant:ident => $assoc_type_name:ident),+ $(,)? }) => {
        impl AssocTyOp for $enum {
            fn assoc_type_name(&self) -> Ident {
                match self {
                    $(
                        Self::$variant(tok) => parse_quote_spanned! { tok.span() => $assoc_type_name },
                    )+
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum UnOp {
    Deref(Token![*]),
    Neg(Token![-]),
    Not(Token![!]),
}
impl TryParse for UnOp {
    fn try_parse(input: ParseStream) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        try_parse_token!(
            input,
            UnOp {
                [*] => Deref,
                [-] => Neg,
                [!] => Not,
            }
        )
    }
}
impl_op! {
    UnOp {
        Deref => ::core::ops::Deref,
        Neg => ::core::ops::Neg,
        Not => ::core::ops::Not,
    }
}
impl_assoc_ty_op! {
    UnOp {
        Deref => Target,
        Neg => Output,
        Not => Output,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum CombineBinOp {
    BitOr(Token![|]),
    BitXor(Token![^]),
    BitAnd(Token![&]),
    Shl(Token![<<]),
    Shr(Token![>>]),
    Add(Token![+]),
    Sub(Token![-]),
    Mul(Token![*]),
    Div(Token![/]),
    Rem(Token![%]),
}
impl CombineBinOp {
    pub(crate) fn try_parse_bitor(input: ParseStream) -> Result<Option<Self>> {
        try_parse_token!(input, CombineBinOp { [|] except [|=] => BitOr })
    }

    pub(crate) fn try_parse_bitxor(input: ParseStream) -> Result<Option<Self>> {
        try_parse_token!(input, CombineBinOp { [^] except [^=] => BitXor })
    }

    pub(crate) fn try_parse_bitand(input: ParseStream) -> Result<Option<Self>> {
        try_parse_token!(input, CombineBinOp { [&] except [&=] => BitAnd })
    }

    pub(crate) fn try_parse_shl_shr(input: ParseStream) -> Result<Option<Self>> {
        try_parse_token!(input, CombineBinOp {
            [<<] except [<<=] => Shl,
            [>>] except [>>=] => Shr,
        })
    }

    pub(crate) fn try_parse_add_sub(input: ParseStream) -> Result<Option<Self>> {
        try_parse_token!(input, CombineBinOp {
            [+] except [+=] => Add,
            [-] except [-=] => Sub,
        })
    }

    pub(crate) fn try_parse_mul_div_rem(input: ParseStream) -> Result<Option<Self>> {
        try_parse_token!(input, CombineBinOp {
            [*] except [*=] => Mul,
            [/] except [/=] => Div,
            [%] except [%=] => Rem,
        })
    }
}
impl_op! {
    CombineBinOp {
        BitOr => ::core::ops::BitOr,
        BitXor => ::core::ops::BitXor,
        BitAnd => ::core::ops::BitAnd,
        Shl => ::core::ops::Shl,
        Shr => ::core::ops::Shr,
        Add => ::core::ops::Add,
        Sub => ::core::ops::Sub,
        Mul => ::core::ops::Mul,
        Div => ::core::ops::Div,
        Rem => ::core::ops::Rem,
    }
}
impl_assoc_ty_op! {
    CombineBinOp {
        BitOr => Output,
        BitXor => Output,
        BitAnd => Output,
        Shl => Output,
        Shr => Output,
        Add => Output,
        Sub => Output,
        Mul => Output,
        Div => Output,
        Rem => Output,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum NoAssocTyBinOp {
    BitOrAssign(Token![|=]),
    BitXorAssign(Token![^=]),
    BitAndAssign(Token![&=]),
    ShlAssign(Token![<<=]),
    ShrAssign(Token![>>=]),
    AddAssign(Token![+=]),
    SubAssign(Token![-=]),
    MulAssign(Token![*=]),
    DivAssign(Token![/=]),
    RemAssign(Token![%=]),
    PartialEq(PartialEqOp),
    PartialOrd(PartialOrdOp),
}
impl TryParse for NoAssocTyBinOp {
    fn try_parse(input: ParseStream) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        macro_rules! try_parse_wrap_return {
            ($variant:ident) => {
                let op = TryParse::try_parse(input)?;
                if op.is_some() {
                    return Ok(op.map(NoAssocTyBinOp::$variant));
                }
            };
        }

        try_parse_wrap_return!(PartialEq);
        try_parse_wrap_return!(PartialOrd);
        try_parse_token!(
            input,
            NoAssocTyBinOp {
                [|=] => BitOrAssign,
                [^=] => BitXorAssign,
                [&=] => BitAndAssign,
                [<<=] => ShlAssign,
                [>>=] => ShrAssign,
                [+=] => AddAssign,
                [-=] => SubAssign,
                [*=] => MulAssign,
                [/=] => DivAssign,
                [%=] => RemAssign,
            }
        )
    }
}
impl_op! {
    NoAssocTyBinOp {
        BitOrAssign => ::core::ops::BitOrAssign,
        BitXorAssign => ::core::ops::BitXorAssign,
        BitAndAssign => ::core::ops::BitAndAssign,
        ShlAssign => ::core::ops::ShlAssign,
        ShrAssign => ::core::ops::ShrAssign,
        AddAssign => ::core::ops::AddAssign,
        SubAssign => ::core::ops::SubAssign,
        MulAssign => ::core::ops::MulAssign,
        DivAssign => ::core::ops::DivAssign,
        RemAssign => ::core::ops::RemAssign,
        PartialEq => ::core::cmp::PartialEq,
        PartialOrd => ::core::cmp::PartialOrd,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PartialEqOp {
    Eq(Token![==]),
    Ne(Token![!=]),
}
impl PartialEqOp {
    fn span(&self) -> Span {
        match self {
            PartialEqOp::Eq(tok) => tok.span(),
            PartialEqOp::Ne(tok) => tok.span(),
        }
    }
}
impl TryParse for PartialEqOp {
    fn try_parse(input: ParseStream) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        try_parse_token!(
            input,
            PartialEqOp {
                [==] => Eq,
                [!=] => Ne,
            }
        )
    }
}

custom_punctuation!(Cmp, <=>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PartialOrdOp {
    Lt(Token![<]),
    Gt(Token![>]),
    Le(Token![<=]),
    Ge(Token![>=]),
    Cmp(Cmp),
}
impl PartialOrdOp {
    fn span(&self) -> Span {
        match self {
            PartialOrdOp::Lt(tok) => tok.span(),
            PartialOrdOp::Gt(tok) => tok.span(),
            PartialOrdOp::Le(tok) => tok.span(),
            PartialOrdOp::Ge(tok) => tok.span(),
            PartialOrdOp::Cmp(tok) => tok.span(),
        }
    }
}
impl TryParse for PartialOrdOp {
    fn try_parse(input: ParseStream) -> Result<Option<Self>>
    where
        Self: Sized,
    {
        try_parse_token!(
            input,
            PartialOrdOp {
                [<] except [<<], [<=] => Lt,
                [>] except [>>], [>=] => Gt,
                [<=] except Cmp => Le,
                [>=] => Ge,
                Cmp => Cmp,
            }
        )
    }
}

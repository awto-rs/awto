use bae::FromAttributes;
use syn::spanned::Spanned;

#[derive(Default, FromAttributes)]
#[bae("awto")]
pub struct RootAttrs {}

#[derive(Default, FromAttributes)]
#[bae("awto")]
pub struct ItemAttrs {
    pub db_type: Option<syn::LitStr>,
    pub default: Option<syn::Lit>,
    pub default_raw: Option<syn::LitStr>,
    pub max_len: Option<syn::LitInt>,
    pub proto_type: Option<syn::LitStr>,
    pub references: Option<KeyVal<syn::LitStr>>,
    pub unique: Option<()>,
}

#[derive(Debug)]
pub struct KeyVal<V: syn::parse::Parse>(pub syn::LitStr, pub V);

impl syn::parse::Parse for KeyVal<syn::LitStr> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if !lookahead.peek(syn::token::Paren) {
            return Err(lookahead.error());
        }

        let group: syn::ExprTuple = input.parse()?;
        let group_span = group.span();
        let mut elems = group.elems.into_iter();

        let first_expr = elems
            .next()
            .ok_or_else(|| syn::Error::new(group_span, "expected group (..., ...)"))?;
        let first = if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = first_expr
        {
            lit_str
        } else {
            return Err(syn::Error::new(
                first_expr.span(),
                "expected first item to be string `(\"first\", ...)`",
            ));
        };

        let second_expr = elems
            .next()
            .ok_or_else(|| syn::Error::new(group_span, "expected group (..., ...)"))?;
        let second = if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = second_expr
        {
            lit_str
        } else {
            return Err(syn::Error::new(
                second_expr.span(),
                "expected second item to be string `(..., \"second\")`",
            ));
        };

        Ok(KeyVal(first, second))
    }
}

use bae::FromAttributes;
use quote::ToTokens;
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
    pub references: Option<KeyVal<syn::Ident, syn::LitStr>>,
    pub unique: Option<()>,
}

#[derive(Debug)]
pub struct KeyVal<K, V>(pub K, pub V);

impl<K, V> syn::parse::Parse for KeyVal<K, V>
where
    K: syn::parse::Parse,
    V: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if !lookahead.peek(syn::token::Paren) {
            return Err(lookahead.error());
        }

        let group: syn::ExprTuple = input.parse()?;
        let group_span = group.span();
        let mut elems = group.elems.into_iter();

        let first: K = syn::parse2(
            elems
                .next()
                .ok_or_else(|| syn::Error::new(group_span, "expected group (..., ...)"))?
                .into_token_stream(),
        )?;

        let second: V = syn::parse2(
            elems
                .next()
                .ok_or_else(|| syn::Error::new(group_span, "expected group (..., ...)"))?
                .into_token_stream(),
        )?;

        if elems.next().is_some() {
            return Err(syn::Error::new(
                group_span,
                "expected group of only 2 items (..., ...)",
            ));
        }

        Ok(KeyVal(first, second))
    }
}

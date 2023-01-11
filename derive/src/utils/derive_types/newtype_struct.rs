use super::{BaseStruct, TupleField};
use darling::{FromDeriveInput, FromField, FromGenerics};

pub struct NewtypeStruct<F: FromField = TupleField, G: FromGenerics = ()> {
    pub ident: syn::Ident,
    pub generics: G,
    pub data: F,
}

impl<F: FromField, G: FromGenerics> FromDeriveInput for NewtypeStruct<F, G> {
    fn from_derive_input(input: &syn::DeriveInput) -> darling::Result<Self> {
        let base: BaseStruct<F, G> = FromDeriveInput::from_derive_input(input)?;
        if base.data.fields.len() > 1 {
            return Err(darling::Error::unsupported_shape("tuple struct").with_span(&base.ident));
        }
        let field = base.data.fields.into_iter().next().ok_or_else(|| {
            darling::Error::unsupported_shape("unit struct").with_span(&base.ident)
        })?;
        Ok(NewtypeStruct {
            ident: base.ident,
            generics: base.generics,
            data: field,
        })
    }
}
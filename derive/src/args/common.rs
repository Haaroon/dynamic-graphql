use proc_macro2::TokenStream;
use quote::quote;

pub use args::*;
pub use fields::*;
pub use generics::*;
pub use interfaces::*;

use crate::utils::common::{CommonArg, CommonField, CommonObject};
use crate::utils::crate_name::get_crate_name;
use crate::utils::impl_block::BaseFnArg;
use crate::utils::rename_rule::{calc_enum_item_name, calc_input_field_name, calc_type_name};
use crate::utils::type_utils::get_owned_type;

mod args;
mod fields;
mod generics;
mod interfaces;

pub trait ArgImplementor: CommonArg {
    fn get_self_arg_definition(&self) -> darling::Result<TokenStream>;
    fn get_typed_arg_definition(&self) -> darling::Result<TokenStream>;
    fn get_arg_definition(&self) -> darling::Result<TokenStream> {
        match &self.get_arg() {
            BaseFnArg::Receiver(_) => self.get_self_arg_definition(),
            BaseFnArg::Typed(_) => self.get_typed_arg_definition(),
        }
    }
    fn get_self_arg_usage(&self) -> darling::Result<TokenStream>;
    fn get_typed_arg_usage(&self) -> darling::Result<TokenStream>;
    fn get_arg_usage(&self) -> darling::Result<TokenStream> {
        match &self.get_arg() {
            BaseFnArg::Receiver(_) => self.get_self_arg_usage(),
            BaseFnArg::Typed(_) => self.get_typed_arg_usage(),
        }
    }
}

pub trait FieldImplementor: CommonField {
    fn define_field(&self) -> darling::Result<TokenStream>;
    fn get_execute_code(&self) -> darling::Result<TokenStream>;
    fn get_resolve_code(&self) -> darling::Result<TokenStream>;
    fn get_field_argument_definition(&self) -> darling::Result<TokenStream>;
    fn get_field_description_code(&self) -> darling::Result<TokenStream>;
    fn get_field_deprecation_code(&self) -> darling::Result<TokenStream>;
    fn get_field_usage_code(&self) -> darling::Result<TokenStream>;
}

pub fn impl_object(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let object_ident = obj.get_ident();
    let name = get_type_name(obj)?;
    let crate_name = get_crate_name();
    let (impl_generics, ty_generics, where_clause) = obj.get_generics()?.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_name::ParentType for #object_ident #ty_generics #where_clause {
            type Type = #object_ident #ty_generics;
        }
        impl #impl_generics #crate_name::GraphqlType for #object_ident #ty_generics #where_clause {
            const NAME: &'static str = #name;
        }
        impl #impl_generics #crate_name::OutputType for #object_ident #ty_generics #where_clause {}
        impl #impl_generics #crate_name::Object for #object_ident #ty_generics #where_clause {}
        impl #impl_generics #crate_name::InterfaceTarget for #object_ident #ty_generics #where_clause {
            const TARGET: &'static str = #name;
        }
    })
}

pub fn impl_input_object(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let object_ident = obj.get_ident();
    let name = get_type_name(obj)?;
    let crate_name = get_crate_name();
    Ok(quote! {
        impl #crate_name::GraphqlType for #object_ident {
            const NAME: &'static str = #name;
        }
        impl #crate_name::InputType for #object_ident {}
        impl #crate_name::InputObject for #object_ident {}
    })
}

pub fn impl_graphql_doc(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let doc = obj.get_doc()?;
    let object_ident = obj.get_ident();
    let crate_name = get_crate_name();
    let doc = match doc {
        None => quote!(None),
        Some(ref doc) => quote!(Some(#doc)),
    };
    let (impl_generics, ty_generics, where_clause) = obj.get_generics()?.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_name::GraphqlDoc for #object_ident #ty_generics #where_clause {
            const DOC: Option<&'static str> = #doc;
        }
    })
}

pub fn impl_resolve_owned(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let crate_name = get_crate_name();
    let object_ident = obj.get_ident();
    let (_, ty_generics, where_clause) = obj.get_generics()?.split_for_impl();
    let (generics_with_lifetime, lifetime) = add_new_lifetime_to_generics(obj.get_generics()?);
    let (impl_generics, _, _) = generics_with_lifetime.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_name::ResolveOwned<#lifetime> for #object_ident #ty_generics #where_clause {
            fn resolve_owned(self, _ctx: &#crate_name::Context) -> #crate_name::Result<Option<#crate_name::FieldValue<#lifetime>>> {
                Ok(Some(#crate_name::FieldValue::owned_any(self)))
            }
        }
    })
}

pub fn impl_resolve_ref(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let crate_name = get_crate_name();
    let object_ident = obj.get_ident();
    let (_, ty_generics, where_clause) = obj.get_generics()?.split_for_impl();
    let (generics_with_lifetime, lifetime) = add_new_lifetime_to_generics(obj.get_generics()?);
    let (impl_generics, _, _) = generics_with_lifetime.split_for_impl();

    Ok(quote! {
        impl #impl_generics #crate_name::ResolveRef<#lifetime> for #object_ident #ty_generics #where_clause {
            fn resolve_ref(&#lifetime self, _ctx: &#crate_name::Context) -> #crate_name::Result<Option<#crate_name::FieldValue<#lifetime>>> {
                Ok(Some(#crate_name::FieldValue::borrowed_any(self)))
            }
        }
    })
}

pub fn impl_resolve_owned_by_value(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let crate_name = get_crate_name();
    let object_ident = obj.get_ident();

    Ok(quote! {
        impl<'__dynamic_graphql_lifetime> #crate_name::ResolveOwned<'__dynamic_graphql_lifetime> for #object_ident {
            fn resolve_owned(self, _ctx: &#crate_name::Context) -> #crate_name::Result<Option<#crate_name::FieldValue<'__dynamic_graphql_lifetime>>> {
                Ok(Some(#crate_name::FieldValue::value(&self)))
            }
        }
    })
}

pub fn impl_resolve_ref_by_value(obj: &impl CommonObject) -> darling::Result<TokenStream> {
    let crate_name = get_crate_name();
    let object_ident = obj.get_ident();
    Ok(quote! {
        impl<'__dynamic_graphql_lifetime> #crate_name::ResolveRef<'__dynamic_graphql_lifetime> for #object_ident {
            fn resolve_ref(&'__dynamic_graphql_lifetime self, _ctx: &#crate_name::Context) -> #crate_name::Result<Option<#crate_name::FieldValue<'__dynamic_graphql_lifetime>>> {
                Ok(Some(#crate_name::FieldValue::value(self)))
            }
        }
    })
}

pub fn impl_define_object() -> TokenStream {
    // todo get "object" from input
    let crate_name = get_crate_name();
    quote! {
        let object = #crate_name::dynamic::Object::new(<Self as #crate_name::Object>::NAME);
    }
}

pub fn impl_define_input_object() -> TokenStream {
    // todo get "object" from input
    let crate_name = get_crate_name();
    quote! {
        let object = #crate_name::dynamic::InputObject::new(<Self as #crate_name::InputObject>::NAME);
    }
}

pub fn register_object_code() -> TokenStream {
    quote!(registry.register_type(object))
}

pub fn object_description(doc: Option<&str>) -> darling::Result<TokenStream> {
    // todo get "object" from input
    if let Some(doc) = doc {
        Ok(quote! {
            let object = object.description(#doc);
        })
    } else {
        Ok(quote! {})
    }
}

pub fn get_type_name(obj: &impl CommonObject) -> darling::Result<String> {
    let name = obj.get_name();
    let object_ident = obj.get_ident();
    let name = calc_type_name(name, &object_ident.to_string());
    Ok(name)
}

pub fn get_enum_item_name(item: &impl CommonField) -> darling::Result<String> {
    let name = item.get_name();
    let item_ident = item.get_ident()?;
    let name = calc_enum_item_name(name, &item_ident.to_string(), item.get_field_rename_rule());
    Ok(name)
}

pub fn get_input_field_name(field: &impl CommonField) -> darling::Result<String> {
    Ok(calc_input_field_name(
        field.get_name(),
        &field.get_ident()?.to_string(),
        field.get_field_rename_rule(),
    ))
}

pub fn get_input_type_ref_code(field: &impl CommonField) -> darling::Result<TokenStream> {
    let crate_name = get_crate_name();
    let field_type = get_owned_type(field.get_type()?);
    Ok(quote! {
        <#field_type as #crate_name::GetInputTypeRef>::get_input_type_ref()
    })
}

pub fn get_new_input_value_code(field: &impl CommonField) -> darling::Result<TokenStream> {
    // todo get "field" from input
    let crate_name = get_crate_name();
    let field_name = get_input_field_name(field)?;
    let get_input_type_ref_code = get_input_type_ref_code(field)?;

    Ok(quote! {
        let field = #crate_name::dynamic::InputValue::new(#field_name, #get_input_type_ref_code);
    })
}

pub fn call_register_fns() -> TokenStream {
    let crate_name = get_crate_name();
    quote!(
        let registry = <Self as #crate_name::RegisterFns>::REGISTER_FNS.iter().fold(registry, |registry, f| f(registry));
    )
}

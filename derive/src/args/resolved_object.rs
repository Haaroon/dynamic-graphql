use crate::args::common::{
    field_deprecation, impl_define_object, impl_graphql_doc, impl_object, impl_resolve_owned,
    impl_resolve_ref,
};
use crate::utils::attributes::{Attributes, CleanAttributes};
use crate::utils::crate_name::get_create_name;
use crate::utils::deprecation::Deprecation;
use crate::utils::docs_utils::Doc;
use crate::utils::error::{GeneratorResult, IntoTokenStream};
use crate::utils::impl_block::{
    BaseFnArg, BaseItemImpl, BaseMethod, FromFnArg, FromItemImpl, FromMethod, TypedArg,
};
use crate::utils::rename_rule::{calc_arg_name, calc_field_name, RenameRule};
use crate::utils::type_utils::{
    get_owned_type, get_value_type, is_type_ref, is_type_slice, is_type_str,
};
use darling::{FromAttributes, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::FnArg;

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct ResolvedObject {
    pub ident: syn::Ident,
    pub attrs: Vec<syn::Attribute>,

    #[darling(default)]
    pub name: Option<String>,
}

#[derive(FromAttributes, Debug, Clone)]
#[darling(attributes(graphql))]
pub struct ResolvedObjectFieldsArgAttrs {
    #[darling(default)]
    pub name: Option<String>,

    #[darling(default)]
    pub ctx: bool,
}

impl Attributes for ResolvedObjectFieldsArgAttrs {
    const ATTRIBUTES: &'static [&'static str] = &["graphql"];
}

#[derive(Debug, Clone)]
pub struct ResolvedObjectFieldsArg {
    pub base: BaseFnArg,
    pub attrs: ResolvedObjectFieldsArgAttrs,
}

impl FromFnArg for ResolvedObjectFieldsArg {
    fn from_fn_arg(arg: &mut FnArg) -> GeneratorResult<Self> {
        let base = BaseFnArg::from_fn_arg(arg)?;
        let base_attrs = BaseFnArg::get_attrs_mut(arg);
        let attrs = ResolvedObjectFieldsArgAttrs::from_attributes(base_attrs)?;
        ResolvedObjectFieldsArgAttrs::clean_attributes(base_attrs);

        Ok(ResolvedObjectFieldsArg { base, attrs })
    }
}

#[derive(FromAttributes, Debug, Clone)]
#[darling(attributes(graphql))]
pub struct ResolvedObjectFieldsMethodAttrs {
    #[darling(default)]
    pub skip: bool,

    #[darling(default)]
    pub name: Option<String>,

    #[darling(default)]
    pub rename_args: Option<RenameRule>,

    #[darling(default)]
    pub deprecation: Deprecation,
}

impl Attributes for ResolvedObjectFieldsMethodAttrs {
    const ATTRIBUTES: &'static [&'static str] = &["graphql"];
}

#[derive(Debug, Clone)]
pub struct ResolvedObjectFieldsMethod {
    pub doc: Doc,
    pub base: BaseMethod<ResolvedObjectFieldsArg>,
    pub attrs: ResolvedObjectFieldsMethodAttrs,
}

impl FromMethod for ResolvedObjectFieldsMethod {
    fn from_method(method: &mut syn::ImplItemMethod) -> GeneratorResult<Self> {
        let doc = Doc::from_attributes(&method.attrs)?;
        let base = BaseMethod::from_method(method)?;
        let attrs = ResolvedObjectFieldsMethodAttrs::from_attributes(&method.attrs)?;
        ResolvedObjectFieldsMethodAttrs::clean_attributes(&mut method.attrs);
        Ok(Self { doc, base, attrs })
    }
}

#[derive(FromAttributes, Debug, Clone)]
#[darling(attributes(graphql))]
pub struct ResolvedObjectFieldsAttrs {
    #[darling(default)]
    pub rename_fields: Option<RenameRule>,

    #[darling(default)]
    pub rename_args: Option<RenameRule>,
}

impl Attributes for ResolvedObjectFieldsAttrs {
    const ATTRIBUTES: &'static [&'static str] = &["graphql"];
}

#[derive(Debug, Clone)]
pub struct ResolvedObjectFields {
    pub doc: Doc,
    pub base: BaseItemImpl<ResolvedObjectFieldsMethod>,
    pub attrs: ResolvedObjectFieldsAttrs,
}

impl FromItemImpl for ResolvedObjectFields {
    fn from_item_impl(item_impl: &mut syn::ItemImpl) -> GeneratorResult<Self> {
        let doc = Doc::from_attributes(&item_impl.attrs)?;
        let base = BaseItemImpl::from_item_impl(item_impl)?;
        let attrs = ResolvedObjectFieldsAttrs::from_attributes(&item_impl.attrs)?;
        ResolvedObjectFieldsAttrs::clean_attributes(&mut item_impl.attrs);

        Ok(Self { doc, base, attrs })
    }
}

impl ToTokens for ResolvedObject {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let impl_object = impl_object(&self.name, &self.ident).into_token_stream();
        let impl_resolve_owned = impl_resolve_owned(&self.ident);
        let impl_resolve_ref = impl_resolve_ref(&self.ident);
        let impl_graphql_doc = Doc::from_attributes(&self.attrs)
            .map(|doc| impl_graphql_doc(&self.ident, &doc))
            .into_token_stream();

        tokens.extend(quote! {
            #impl_object
            #impl_resolve_owned
            #impl_resolve_ref
            #impl_graphql_doc
        });
    }
}

fn field_description(doc: &Doc) -> GeneratorResult<TokenStream> {
    if let Some(doc) = &doc.doc {
        Ok(quote! {
            let field = field.description(#doc);
        })
    } else {
        Ok(quote! {})
    }
}

fn get_arg_ident(index: usize, arg: &ResolvedObjectFieldsArg) -> syn::Ident {
    syn::Ident::new(&format!("arg{}", index), arg.base.span())
}

fn is_arg_ctx(arg: &ResolvedObjectFieldsArg) -> bool {
    arg.attrs.ctx
        || matches!(arg.base, BaseFnArg::Typed(TypedArg{ref ident, ..}) if ident == "ctx" || ident == "_ctx")
}

fn get_arg_definition(
    index: usize,
    arg: &ResolvedObjectFieldsArg,
    rename_rule: Option<&RenameRule>,
) -> GeneratorResult<TokenStream> {
    let arg_ident = get_arg_ident(index, arg);

    match &arg.base {
        BaseFnArg::Receiver(_) => Ok(quote! {
            let parent = ctx.parent_value.try_downcast_ref::<Self>()?;
            let #arg_ident = parent;
        }),
        BaseFnArg::Typed(typed) => {
            if is_arg_ctx(arg) {
                Ok(quote! {
                    let #arg_ident = &ctx;
                })
            } else {
                let arg_name =
                    calc_arg_name(&arg.attrs.name, &typed.ident.to_string(), rename_rule);
                let value_type = get_value_type(&typed.ty);
                match value_type {
                    None => Ok(quote! {
                        let #arg_ident = ctx.args.try_get(#arg_name)?.deserialize()?;
                    }),
                    Some(ty) => Ok(quote! {
                        let #arg_ident: #ty = ctx.args.try_get(#arg_name)?.deserialize()?;
                    }),
                }
            }
        }
    }
}

fn get_args_definition(
    args: &[ResolvedObjectFieldsArg],
    rename_rule: Option<&RenameRule>,
) -> GeneratorResult<TokenStream> {
    args.iter()
        .enumerate()
        .map(|(index, arg)| get_arg_definition(index, arg, rename_rule))
        .collect()
}

fn get_arg_usage(index: usize, arg: &ResolvedObjectFieldsArg) -> TokenStream {
    let arg_ident = get_arg_ident(index, arg);
    match arg.base {
        BaseFnArg::Receiver(_) => {
            quote!(#arg_ident,)
        }
        BaseFnArg::Typed(ref typed) => {
            let is_ctx = is_arg_ctx(arg);
            let is_owned = !is_type_ref(&typed.ty);
            if is_ctx || is_owned {
                quote!(#arg_ident,)
            } else {
                quote!(&#arg_ident,)
            }
        }
    }
}

fn get_args_usage(args: &[ResolvedObjectFieldsArg]) -> TokenStream {
    args.iter()
        .enumerate()
        .map(|(index, arg)| get_arg_usage(index, arg))
        .collect()
}

fn get_argument_definition(
    arg: &ResolvedObjectFieldsArg,
    rename_rule: Option<&RenameRule>,
) -> TokenStream {
    if is_arg_ctx(arg) {
        return quote!();
    }
    let BaseFnArg::Typed(typed) = &arg.base else {
        return quote!();
    };
    let create_name = get_create_name();
    let arg_name = calc_arg_name(&arg.attrs.name, &typed.ident.to_string(), rename_rule);
    let arg_type = get_owned_type(&typed.ty);

    quote! {
        let arg = #create_name::dynamic::InputValue::new(#arg_name, <#arg_type as #create_name::GetInputTypeRef>::get_input_type_ref());
        let field = field.argument(arg);
    }
}

fn get_argument_definitions(
    args: &[ResolvedObjectFieldsArg],
    rename_rule: Option<&RenameRule>,
) -> TokenStream {
    args.iter()
        .map(|arg| get_argument_definition(arg, rename_rule))
        .collect()
}

fn define_fields(
    object: &ResolvedObjectFields,
    method: &ResolvedObjectFieldsMethod,
) -> GeneratorResult<TokenStream> {
    let field_ident = &method.base.ident;
    let field_name = calc_field_name(
        method.attrs.name.as_ref(),
        &field_ident.to_string(),
        &object.attrs.rename_fields,
    );
    let ty = method.base.output_type.as_ref().ok_or_else(|| {
        darling::Error::custom("Field must have return type").with_span(&method.base.ident)
    })?;
    let create_name = get_create_name();

    let is_str = is_type_str(ty);
    let is_slice = is_type_slice(ty);
    let is_ref = is_type_ref(ty);
    let is_owned = !is_ref;
    let owned_type = get_owned_type(ty);

    let resolve_ref = quote! {
        #create_name::ResolveRef::resolve_ref(value, &ctx)
    };
    let resolve_owned = quote! {
        #create_name::ResolveOwned::resolve_owned(value, &ctx)
    };
    let resolve = if is_owned || is_str || is_slice {
        resolve_owned
    } else {
        resolve_ref
    };
    let rename_rule = method
        .attrs
        .rename_args
        .as_ref()
        .or(object.attrs.rename_args.as_ref());
    let args_definition = get_args_definition(&method.base.args, rename_rule)?;
    let args = get_args_usage(&method.base.args);

    let execute = if method.base.asyncness {
        quote! {
            let value = Self::#field_ident(#args).await;
        }
    } else {
        quote! {
            let value = Self::#field_ident(#args);
        }
    };
    let argument_definitions = get_argument_definitions(&method.base.args, rename_rule);

    let description = field_description(&method.doc)?;
    let deprecation = field_deprecation(&method.attrs.deprecation);

    Ok(quote! {
        let field = #create_name::dynamic::Field::new(#field_name, <#owned_type as #create_name::GetOutputTypeRef>::get_output_type_ref(), |ctx| {
            #create_name::dynamic::FieldFuture::new(async move {
                #args_definition
                #execute
                #resolve
            })
        });
        #argument_definitions
        #description
        #deprecation
        let object = object.field(field);
    })
}

fn impl_object_description() -> TokenStream {
    let create_name = get_create_name();

    quote! {
        let object = <Self as #create_name::GraphqlDoc>::DOC.iter().fold(object, |object, doc| {
            object.description(doc.to_owned())
        });
    }
}

fn impl_register(object: &ResolvedObjectFields) -> GeneratorResult<TokenStream> {
    let create_name = get_create_name();
    let ty = &object.base.ty;
    let define_object = impl_define_object();
    let description = impl_object_description();
    let define_fields = object
        .base
        .methods
        .iter()
        .filter(|method| !method.attrs.skip)
        .map(|method| define_fields(object, method).into_token_stream())
        .collect::<Vec<_>>();

    Ok(quote! {
        impl #create_name::Register for #ty {
            fn register(registry: #create_name::Registry) -> #create_name::Registry {
                #define_object

                #(#define_fields)*

                #description

                registry.register_type(object)
            }
        }
    })
}

impl ToTokens for ResolvedObjectFields {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let impl_register = impl_register(self).into_token_stream();
        tokens.extend(quote! {
            #impl_register
        });
    }
}

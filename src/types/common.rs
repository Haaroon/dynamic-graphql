use crate::types::{GetInputTypeRef, GetOutputTypeRef, GraphqlType, InputType, OutputType};
use crate::{Register, Registry};
use async_graphql::{dynamic, MaybeUndefined};
use std::borrow::Cow;

impl<T> Register for &T
where
    T: Register + 'static,
{
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}

impl<'a, T> GraphqlType for &'a T
where
    T: GraphqlType + 'static,
{
    fn get_type_name() -> Cow<'static, str> {
        <T as GraphqlType>::get_type_name()
    }
}

impl<T: OutputType + 'static> OutputType for &T {}

impl<T: Register + Clone + 'static> Register for Cow<'_, T> {
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}
impl<T: OutputType + Clone + 'static> GraphqlType for Cow<'_, T> {
    fn get_type_name() -> Cow<'static, str> {
        <T as GraphqlType>::get_type_name()
    }
}

impl<T: OutputType + Clone + 'static> OutputType for Cow<'_, T> {}

impl Register for String {}
impl GraphqlType for String {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::STRING.into()
    }
}

impl InputType for String {}

impl OutputType for String {}

impl Register for &str {}
impl GraphqlType for &str {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::STRING.into()
    }
}

impl InputType for &str {}

impl OutputType for &str {}

impl GraphqlType for str {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::STRING.into()
    }
}

impl Register for str {}
impl InputType for str {}

impl OutputType for str {}

impl Register for async_graphql::ID {}
impl GraphqlType for async_graphql::ID {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::ID.into()
    }
}

impl InputType for async_graphql::ID {}

impl OutputType for async_graphql::ID {}

impl Register for bool {}
impl GraphqlType for bool {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::BOOLEAN.into()
    }
}

impl InputType for bool {}

impl OutputType for bool {}

impl Register for f32 {}
impl GraphqlType for f32 {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::FLOAT.into()
    }
}

impl InputType for f32 {}

impl OutputType for f32 {}

impl Register for f64 {}
impl GraphqlType for f64 {
    fn get_type_name() -> Cow<'static, str> {
        dynamic::TypeRef::FLOAT.into()
    }
}

impl InputType for f64 {}

impl OutputType for f64 {}

macro_rules! int_output_value {
    ($($t:ty),*) => {
        $(
            impl Register for $t {}
            impl GraphqlType for $t {
                fn get_type_name() -> Cow<'static, str> {
                    dynamic::TypeRef::INT.into()
                }
            }
            impl InputType for $t {}
            impl OutputType for $t {}
        )*
    };
}

int_output_value!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl<T> Register for Option<T>
where
    T: Register + 'static,
{
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}
impl<T> Register for MaybeUndefined<T>
where
    T: Register + 'static,
{
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}
impl<T, E> Register for Result<T, E>
where
    T: Register + 'static,
{
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}
impl<T> Register for Vec<T>
where
    T: Register + 'static,
{
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}
impl<T> Register for &[T]
where
    T: Register + 'static,
{
    fn register(registry: Registry) -> Registry {
        registry.register::<T>()
    }
}

pub trait TypeRefExt {
    fn optional(self) -> Self;
    fn list(self) -> Self;
}

pub enum TypeRefInner {
    Named(String),
    NamedNN(String),
    List(String),
    ListNN(String),
    NNList(String),
    NNListNN(String),
}

impl From<TypeRefInner> for dynamic::TypeRef {
    fn from(value: TypeRefInner) -> Self {
        match value {
            TypeRefInner::Named(name) => dynamic::TypeRef::named(name),
            TypeRefInner::NamedNN(name) => dynamic::TypeRef::named_nn(name),
            TypeRefInner::List(name) => dynamic::TypeRef::named_list(name),
            TypeRefInner::ListNN(name) => dynamic::TypeRef::named_list_nn(name),
            TypeRefInner::NNList(name) => dynamic::TypeRef::named_nn_list(name),
            TypeRefInner::NNListNN(name) => dynamic::TypeRef::named_nn_list_nn(name),
        }
    }
}

impl TypeRefExt for TypeRefInner {
    fn optional(self) -> Self {
        match self {
            TypeRefInner::Named(name) => TypeRefInner::Named(name),
            TypeRefInner::NamedNN(name) => TypeRefInner::Named(name),
            TypeRefInner::List(name) => TypeRefInner::List(name),
            TypeRefInner::ListNN(name) => TypeRefInner::List(name),
            TypeRefInner::NNList(name) => TypeRefInner::NNList(name),
            TypeRefInner::NNListNN(name) => TypeRefInner::NNList(name),
        }
    }

    fn list(self) -> Self {
        match self {
            TypeRefInner::Named(name) => TypeRefInner::ListNN(name),
            TypeRefInner::NamedNN(name) => TypeRefInner::NNListNN(name),
            TypeRefInner::List(name) => TypeRefInner::List(name),
            TypeRefInner::ListNN(name) => TypeRefInner::ListNN(name),
            TypeRefInner::NNList(name) => TypeRefInner::NNList(name),
            TypeRefInner::NNListNN(name) => TypeRefInner::NNListNN(name),
        }
    }
}

impl<T, E> GetOutputTypeRef for Result<T, E>
where
    T: GetOutputTypeRef,
{
    type Output = T::Output;
    #[inline]
    fn get_output_type_ref() -> Self::Output {
        T::get_output_type_ref()
    }
}

impl<T: OutputType> GetOutputTypeRef for T {
    type Output = TypeRefInner;
    #[inline]
    fn get_output_type_ref() -> Self::Output {
        TypeRefInner::NamedNN(T::get_output_type_name().to_string())
    }
}

impl<T: GetOutputTypeRef<Output = TypeRefInner>> GetOutputTypeRef for Option<T> {
    type Output = TypeRefInner;
    #[inline]
    fn get_output_type_ref() -> Self::Output {
        let t = T::get_output_type_ref();
        t.optional()
    }
}

impl<T: GetOutputTypeRef<Output = TypeRefInner>> GetOutputTypeRef for Vec<T> {
    type Output = TypeRefInner;
    #[inline]
    fn get_output_type_ref() -> Self::Output {
        T::get_output_type_ref().list()
    }
}

impl<T: GetOutputTypeRef<Output = TypeRefInner>> GetOutputTypeRef for &[T] {
    type Output = TypeRefInner;
    #[inline]
    fn get_output_type_ref() -> Self::Output {
        T::get_output_type_ref().list()
    }
}

impl<T: InputType> GetInputTypeRef for T {
    type Output = TypeRefInner;
    #[inline]
    fn get_input_type_ref() -> Self::Output {
        TypeRefInner::NamedNN(T::get_input_type_name().to_string())
    }
}

impl<T: GetInputTypeRef<Output = TypeRefInner>> GetInputTypeRef for Option<T> {
    type Output = TypeRefInner;
    #[inline]
    fn get_input_type_ref() -> Self::Output {
        T::get_input_type_ref().optional()
    }
}

impl<T: GetInputTypeRef<Output = TypeRefInner>> GetInputTypeRef for MaybeUndefined<T> {
    type Output = TypeRefInner;
    #[inline]
    fn get_input_type_ref() -> Self::Output {
        T::get_input_type_ref().optional()
    }
}

impl<T: GetInputTypeRef<Output = TypeRefInner>> GetInputTypeRef for Vec<T> {
    type Output = TypeRefInner;
    #[inline]
    fn get_input_type_ref() -> Self::Output {
        T::get_input_type_ref().list()
    }
}
impl<T: GetInputTypeRef<Output = TypeRefInner>> GetInputTypeRef for &[T] {
    type Output = TypeRefInner;
    #[inline]
    fn get_input_type_ref() -> Self::Output {
        T::get_input_type_ref().list()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_output_type_ref() {
        let type_ref: dynamic::TypeRef = <String as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "String!");
        let type_ref: dynamic::TypeRef =
            <Option<String> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "String");
        let type_ref: dynamic::TypeRef =
            <Vec<String> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]!");
        let type_ref: dynamic::TypeRef =
            <Option<Vec<String>> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]");
        let type_ref: dynamic::TypeRef =
            <Vec<Option<String>> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]!");
        let type_ref: dynamic::TypeRef =
            <Option<Vec<Option<String>>> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]");
        let type_ref: dynamic::TypeRef =
            <&[String] as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]!");
        let type_ref: dynamic::TypeRef =
            <Option<&[String]> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]");
        let type_ref: dynamic::TypeRef =
            <&[Option<String>] as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]!");
        let type_ref: dynamic::TypeRef =
            <Option<&[Option<String>]> as GetOutputTypeRef>::get_output_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]");
    }

    #[test]
    fn test_get_input_type_ref() {
        let type_ref: dynamic::TypeRef = <String as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "String!");
        let type_ref: dynamic::TypeRef =
            <Option<String> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "String");
        let type_ref: dynamic::TypeRef =
            <Vec<String> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]!");
        let type_ref: dynamic::TypeRef =
            <Option<Vec<String>> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]");
        let type_ref: dynamic::TypeRef =
            <Vec<Option<String>> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]!");
        let type_ref: dynamic::TypeRef =
            <Option<Vec<Option<String>>> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]");
        let type_ref: dynamic::TypeRef =
            <&[String] as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]!");
        let type_ref: dynamic::TypeRef =
            <Option<&[String]> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String!]");
        let type_ref: dynamic::TypeRef =
            <&[Option<String>] as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]!");
        let type_ref: dynamic::TypeRef =
            <Option<&[Option<String>]> as GetInputTypeRef>::get_input_type_ref().into();
        assert_eq!(type_ref.to_string(), "[String]");
    }
}

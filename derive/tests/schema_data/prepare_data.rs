use self::example::ExampleApp;
use self::expand_example::AnotherValueApp;
use self::root::Query;
use crate::schema_utils::normalize_schema;
use dynamic_graphql_derive::App;

#[derive(App)]
struct App(Query, ExampleApp, AnotherValueApp);

mod root {
    use dynamic_graphql_derive::SimpleObject;

    #[derive(SimpleObject)]
    #[graphql(root)]
    pub struct Query;
}

mod prepare {
    use dynamic_graphql::{Context, Data};
    use std::any::{Any, TypeId};

    pub fn get_data<T: Any + Sync + Send>(data: &Data) -> Option<&T> {
        data.get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    pub type PrepareFn<P> = fn(parent: &mut P, ctx: &Context);
}

mod example {
    use super::root;
    use crate::schema_data::prepare_data::prepare::PrepareFn;
    use dynamic_graphql::{Context, Data, GetSchemaData};
    use dynamic_graphql_derive::{
        App, ExpandObject, ExpandObjectFields, ResolvedObject, ResolvedObjectFields,
    };

    #[derive(Default)]
    pub struct ExamplePrepares(pub Vec<PrepareFn<Example>>);

    #[derive(ResolvedObject, Default)]
    pub struct Example {
        pub value: Option<i32>,
        pub data: Data,
    }

    #[ResolvedObjectFields]
    impl Example {
        fn value(&self) -> i32 {
            self.value.unwrap()
        }
    }

    impl Example {
        fn prepare(&mut self, ctx: &Context) {
            if ctx.look_ahead().field("value").exists() {
                // expensive calculation
                self.value = Some(42);
            }
            if let Some(prepares) = ctx.get_schema_data().get::<ExamplePrepares>() {
                prepares.0.iter().for_each(|prepare| prepare(self, ctx));
            }
        }
    }

    #[derive(ExpandObject)]
    struct ExampleQuery<'a>(&'a root::Query);

    #[ExpandObjectFields]
    impl ExampleQuery<'_> {
        fn example(ctx: &Context) -> Example {
            let mut example = Example::default();
            example.prepare(ctx);
            example
        }
    }

    #[derive(App)]
    pub struct ExampleApp(ExampleQuery<'static>);
}

mod expand_example {
    use super::example::Example;
    use super::example::ExamplePrepares;
    use super::prepare::get_data;
    use dynamic_graphql::{Register, Registry};
    use dynamic_graphql_derive::{App, ExpandObject, ExpandObjectFields};

    struct AnotherValue(i32);

    #[derive(ExpandObject)]
    #[graphql(register(PrepareAnotherValue))]
    struct ExampleAnotherValue<'a>(&'a Example);

    #[ExpandObjectFields]
    impl ExampleAnotherValue<'_> {
        fn another_value(&self) -> i32 {
            let value: &AnotherValue = get_data(&self.parent().data).unwrap();
            value.0
        }
    }

    struct PrepareAnotherValue;
    impl Register for PrepareAnotherValue {
        fn register(mut registry: Registry) -> Registry {
            let example_prepare: &mut ExamplePrepares = registry.data.get_mut_or_default();
            example_prepare.0.push(|example, ctx| {
                if ctx.look_ahead().field("anotherValue").exists() {
                    // expensive calculation
                    example.data.insert(AnotherValue(43));
                }
            });
            registry
        }
    }

    #[derive(App)]
    pub struct AnotherValueApp(ExampleAnotherValue<'static>);
}

#[tokio::test]
async fn test() {
    let schema = App::create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"
            type Example {
              value: Int!
              anotherValue: Int!
            }

            type Query {
              example: Example!
            }

            schema {
              query: Query
            }
        "#
        )
    );

    let query = r#"
        query {
            example {
                value
                anotherValue
            }
        }
    "#;
    let res = schema.execute(query).await;
    assert_eq!(
        res.data.into_json().unwrap(),
        serde_json::json!({
            "example": {
                "value": 42,
                "anotherValue": 43,
            }
        })
    );
}

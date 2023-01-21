use crate::schema_utils::normalize_schema;
use dynamic_graphql::{App, Interface, SimpleObject};

#[test]
fn test_impl_interface() {
    #[Interface(NodeInterface)]
    trait Node {
        fn id(&self) -> String;
    }

    assert_eq!(<NodeInterface as Interface>::NAME, "Node");
}

#[test]
fn test_impl_interface_with_name() {
    #[Interface(NodeInterface)]
    #[graphql(name = "Other")]
    trait Node {
        fn id(&self) -> String;
    }

    assert_eq!(<NodeInterface as Interface>::NAME, "Other");
}

#[test]
fn test_schema() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: String,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

            interface Node {
              theId: String!
            }

            type Query {
              foo: String!
            }

            schema {
              query: Query
            }

            "#
        ),
    );
}

#[test]
fn test_schema_with_name() {
    #[Interface(NodeInterface)]
    #[graphql(name = "Other")]
    trait Node {
        #[graphql(name = "id")]
        fn get_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: String,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

            interface Other {
              id: String!
            }

            type Query {
              foo: String!
            }

            schema {
              query: Query
            }

            "#
        ),
    );
}

#[test]
fn test_schema_with_rename() {
    #[Interface(NodeInterface)]
    #[graphql(rename_fields = "snake_case")]
    trait Node {
        #[graphql(name = "id")]
        fn get_id(&self) -> String;

        fn the_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: String,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

            interface Node {
              id: String!
              the_id: String!
            }

            type Query {
              foo: String!
            }

            schema {
              query: Query
            }

            "#
        ),
    );
}

#[test]
fn test_schema_description() {
    /// the interface
    #[Interface(NodeInterface)]
    trait Node {
        /// the id
        fn the_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: String,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

            """
              the interface
            """
            interface Node {
              """
                the id
              """
              theId: String!
            }

            type Query {
              foo: String!
            }

            schema {
              query: Query
            }

            "#
        ),
    );
}

#[test]
fn test_schema_with_deprecation() {
    #[Interface(NodeInterface)]
    trait Node {
        #[graphql(deprecation)]
        fn the_id(&self) -> String;

        #[graphql(deprecation = "deprecated")]
        fn old(&self) -> String;
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: String,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

            interface Node {
                theId: String! @deprecated
                old: String! @deprecated(reason: "deprecated")
            }

            type Query {
              foo: String!
            }

            schema {
              query: Query
            }

            "#
        ),
    );
}

#[test]
fn test_schema_with_skip() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
        #[graphql(skip)]
        fn old(&self) -> String;
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: String,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

            interface Node {
              theId: String!
            }

            type Query {
              foo: String!
            }

            schema {
              query: Query
            }

            "#
        ),
    );
}
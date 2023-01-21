use crate::schema_utils::normalize_schema;
use dynamic_graphql::{App, Interface, SimpleObject};
use dynamic_graphql_derive::{ResolvedObject, ResolvedObjectFields};

#[test]
fn test_schema_simple_object_mark_as() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    #[graphql(mark_as = "Node")]
    struct FooNode {
        the_id: String,
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

                type FooNode implements Node {
                    theId: String!
                }

                interface Node {
                    theId: String!
                }

                type Query {
                    foo: FooNode!
                }

                schema {
                    query: Query
                }

            "#
        ),
    );
}

#[test]
fn test_schema_simple_object_mark_with() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    #[graphql(mark_with = "NodeInterface")]
    struct FooNode {
        the_id: String,
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

                type FooNode implements Node {
                    theId: String!
                }

                interface Node {
                    theId: String!
                }

                type Query {
                    foo: FooNode!
                }

                schema {
                    query: Query
                }

            "#
        ),
    );
}

#[test]
fn test_schema_simple_object_with_implement() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> &String;
    }

    #[derive(SimpleObject)]
    #[graphql(implement = "NodeInterface")]
    struct FooNode {
        some_field: String,
        #[graphql(skip)]
        id: String,
    }

    impl Node for FooNode {
        fn the_id(&self) -> &String {
            &self.id
        }
    }

    println!("==============================================");

    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

                type FooNode implements Node {
                    someField: String!
                    theId: String!
                }

                interface Node {
                    theId: String!
                }

                type Query {
                    foo: FooNode!
                }


                schema {
                    query: Query
                }

            "#
        ),
    );
}

#[test]
fn test_schema_simple_object_with_error() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(SimpleObject)]
    #[graphql(mark_with = "NodeInterface")]
    struct FooNode {
        other_field: String,
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish();
    assert!(schema.is_err());
    assert_eq!(
        schema.err().unwrap().to_string(),
        r#"Object "FooNode" requires field "theId" defined by interface "Node""#
    );
}

#[test]
fn test_schema_resolved_object_mark_as() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(ResolvedObject)]
    #[graphql(mark_as = "Node")]
    struct FooNode;

    #[ResolvedObjectFields]
    impl FooNode {
        fn the_id(&self) -> String {
            "the_id".to_string()
        }
    }
    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

                type FooNode implements Node {
                    theId: String!
                }

                interface Node {
                    theId: String!
                }

                type Query {
                    foo: FooNode!
                }


                schema {
                    query: Query
                }

            "#
        ),
    );
}

#[test]
fn test_schema_resolved_object_mark_with() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(ResolvedObject)]
    #[graphql(mark_with = "NodeInterface")]
    struct FooNode;

    #[ResolvedObjectFields]
    impl FooNode {
        fn the_id(&self) -> String {
            "the_id".to_string()
        }
    }
    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

                type FooNode implements Node {
                    theId: String!
                }

                interface Node {
                    theId: String!
                }

                type Query {
                    foo: FooNode!
                }


                schema {
                    query: Query
                }

            "#
        ),
    );
}

#[test]
fn test_schema_resolved_object_with_implement() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(ResolvedObject)]
    #[graphql(implement = "NodeInterface")]
    struct FooNode;

    #[ResolvedObjectFields]
    impl FooNode {
        fn other_fields(&self) -> String {
            "other".to_string()
        }
    }

    impl Node for FooNode {
        fn the_id(&self) -> String {
            "the_id".to_string()
        }
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish().unwrap();
    let sdl = schema.sdl();
    assert_eq!(
        normalize_schema(&sdl),
        normalize_schema(
            r#"

                type FooNode implements Node {
                    otherFields: String!
                    theId: String!
                }

                interface Node {
                    theId: String!
                }

                type Query {
                    foo: FooNode!
                }


                schema {
                    query: Query
                }

            "#
        ),
    );
}

#[test]
fn test_schema_resolved_object_with_error() {
    #[Interface(NodeInterface)]
    trait Node {
        fn the_id(&self) -> String;
    }

    #[derive(ResolvedObject)]
    #[graphql(mark_with = "NodeInterface")]
    struct FooNode;

    #[ResolvedObjectFields]
    impl FooNode {
        fn other_field(&self) -> String {
            "the_id".to_string()
        }
    }

    #[derive(SimpleObject)]
    struct Query {
        foo: FooNode,
    }

    #[derive(App)]
    struct App(Query, NodeInterface<'static>, FooNode);

    let registry = dynamic_graphql::Registry::new();
    let registry = registry.register::<App>().set_root("Query");
    let schema = registry.create_schema().finish();
    assert!(schema.is_err());
    assert_eq!(
        schema.err().unwrap().to_string(),
        r#"Object "FooNode" requires field "theId" defined by interface "Node""#
    );
}
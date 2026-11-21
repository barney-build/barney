// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney Schema Implementation
//!
//! The bny_schema crate is built atop bny_syntax to
//! provide validation of `.bny` files and rich diagnostics.

mod compiled;
pub use compiled::{BnySchema, BnySchemaId};

mod diagnostics;

mod key;
pub use key::*;

mod schema;
pub use schema::{SchemaDef, SchemaRef};

mod node;
pub use node::NodeDef;

mod occurrence;
pub use occurrence::{Occurrence, OccurrenceError};

mod rule;
pub use rule::Rule;

mod property;
pub use property::PropertyDef;

mod spec;
pub use spec::{NodeSpec, PropertySpec};

mod registry;
pub use registry::{RegistryError, SchemaIndex, SchemaRegistry};

mod validated;
pub use validated::ValidatedDocument;

mod value;
pub use value::{AsBnyValueType, BnyValueType, ValueError};

#[cfg(test)]
mod tests {
    use bny_diag::DiagnosticSink;
    use bny_syntax::{BnyInterner, BnySpec};
    use itertools::Itertools;

    use crate::{
        BnySchema, SchemaRef, SchemaRegistry,
        tests::{test_recipe_schema::RecipeKeys, test_script_schema::TestKeys},
    };

    mod test_recipe_schema {

        use crate::{
            NodeDef, Occurrence, PropertyDef, Rule, SchemaDef, SchemaKey, SchemaName, SchemaRef,
            tests::test_script_schema,
        };

        // top level nodes
        #[derive(Debug, Clone, Copy)]
        #[repr(u8)]
        pub enum RecipeKeys {
            MetaNode,
            MetaId,
            Build,
            LinksNode,
            LinksNodeBugs,
        }

        impl<'a> SchemaKey<'a> for RecipeKeys {
            fn symbolic_id(self) -> u8 {
                self as u8
            }

            fn name(&self) -> SchemaName<'a> {
                match self {
                    RecipeKeys::MetaNode => SchemaName::Explicit("meta"),
                    RecipeKeys::MetaId => SchemaName::Explicit("id"),
                    RecipeKeys::Build => SchemaName::Explicit("build"),
                    RecipeKeys::LinksNode => SchemaName::Explicit("links"),
                    RecipeKeys::LinksNodeBugs => SchemaName::Explicit("bugs"),
                }
            }
        }

        pub static ID: SchemaRef = SchemaRef("barney.test.pkg.schema");
        pub static SCHEMA: SchemaDef<RecipeKeys> = SchemaDef::new(ID).children(&[
            &NodeDef::new(RecipeKeys::MetaNode)
                .occurrence(Occurrence::exactly(1))
                .properties(&[PropertyDef::new::<&str>(RecipeKeys::MetaId)])
                .children(Rule::Nodes(&[&NodeDef::new(RecipeKeys::LinksNode)
                    .children(Rule::Nodes(&[&NodeDef::new(
                        RecipeKeys::LinksNodeBugs,
                    )]))])),
            &NodeDef::new(RecipeKeys::Build).children(Rule::External(test_script_schema::ID)),
        ]);
    }

    mod test_script_schema {
        //! Testing embedded recursive script schema

        use crate::{NodeDef, Rule, SchemaDef, SchemaKey, SchemaName, SchemaRef};

        #[repr(u8)]
        #[derive(Debug, Clone, Copy)]
        pub enum TestKeys {
            WithNode,
            ForeachNode,
            InvokeNode,
        }

        impl<'a> SchemaKey<'a> for TestKeys {
            fn symbolic_id(self) -> u8 {
                self as u8
            }

            fn name(&self) -> crate::SchemaName<'a> {
                match self {
                    TestKeys::WithNode => SchemaName::Explicit("with"),
                    TestKeys::ForeachNode => SchemaName::Explicit("foreach"),
                    TestKeys::InvokeNode => SchemaName::Dynamic,
                }
            }
        }

        pub static ID: SchemaRef = SchemaRef("barney.test.script.schema");
        pub static SCHEMA: SchemaDef<TestKeys> = SchemaDef::new(ID).children(&[
            // `with` conditionals, recursive schema
            &NodeDef::new(TestKeys::WithNode).children(Rule::External(ID)),
            // `foreach` conditionals, recursive schema
            &NodeDef::new(TestKeys::ForeachNode).children(Rule::External(ID)),
            // anything else is an invocation
            &NodeDef::new(TestKeys::InvokeNode),
        ]);
    }
    // Basic schema const-verification
    #[test]
    fn test_const_schema() {
        assert_eq!(
            test_script_schema::SCHEMA.id(),
            SchemaRef("barney.test.script.schema")
        );

        let mut intern = BnyInterner::default();
        let schema = BnySchema::compile(&mut intern, &test_recipe_schema::SCHEMA)
            .expect("schema compilation failed");

        let mut reg = SchemaRegistry::new(test_recipe_schema::ID);
        reg.install(schema)
            .expect("Failed to install recipe schema");
        reg.install(
            BnySchema::compile(&mut intern, &test_script_schema::SCHEMA)
                .expect("schema compilation failed"),
        )
        .expect("Failed to install script schema");
        let linked = reg.link(&mut intern).expect("Failed to link schemas");

        let test_doc = r#"
            (barney)version 1
            meta id="beep bop" {
            }
            meta id="i am fine :D"
            build {
                with {
                    chickens
                    bobsled
                    with {

                    }
                }
            }
            build {
                second
            }
            build {
                third
                should-not-be-first
            }
            not_schema_matched {

            }
            "#;
        let spec = BnySpec::new(test_doc);
        let tree = spec.process(&mut intern);

        let mut sink = DiagnosticSink::new();
        let valid = linked.validate(&tree, &intern, &mut sink);
        let build = valid
            .nodes(RecipeKeys::Build)
            .expect("should have build nodes")
            .collect_vec();
        assert_eq!(build.len(), 3);

        let meta = valid
            .nodes(RecipeKeys::MetaNode)
            .expect("should have one meta")
            .collect_vec();
        assert_eq!(meta.len(), 2);

        assert_eq!(
            &meta[0]
                .property::<&str>(RecipeKeys::MetaId)
                .expect("missing id in [0]"),
            &"beep bop"
        );

        assert_eq!(
            &meta[1]
                .property::<&str>(RecipeKeys::MetaId)
                .expect("missing id in [1]"),
            &"i am fine :D"
        );

        let links = valid.nodes(RecipeKeys::LinksNode);
        assert!(links.is_err());

        let build_node = build.get(0).unwrap();
        assert_eq!(build_node.name(), "build");
        let build_with = build_node
            .nodes(TestKeys::WithNode)
            .expect("Should have WITH nodes")
            .collect_vec();

        assert_eq!(build_with.len(), 1);

        // shouldnt be able to grab schema elsewhere (bucketing)
        let build_with_wrong_schema = valid.nodes(RecipeKeys::LinksNodeBugs);
        assert!(build_with_wrong_schema.is_err());
    }
}

// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney Schema Implementation
//!
//! The bny_schema crate is built atop bny_syntax to
//! provide validation of `.bny` files and rich diagnostics.

mod compiled;
pub use compiled::{BnySchema, BnySchemaId};

mod key;
pub use key::*;

mod schema;
pub use schema::{SchemaDef, SchemaRef};

mod node;
pub use node::{NodeDef, node_def};

mod rule;
pub use rule::Rule;

mod spec;
pub use spec::NodeSpec;

mod registry;
pub use registry::{RegistryError, SchemaRegistry};

#[cfg(test)]
mod tests {
    use bny_syntax::BnyInterner;

    use crate::{BnySchema, SchemaRef, SchemaRegistry};

    mod test_recipe_schema {
        use crate::{
            Rule, SchemaDef, SchemaKey, SchemaName, SchemaRef, node_def, tests::test_script_schema,
        };

        // top level nodes
        #[derive(Debug, Clone, Copy)]
        #[repr(u16)]
        pub enum RecipeKeys {
            MetaNode,
            // MetaNodeName,
            Build,
            LinksNode,
            LinksNodeBugs,
        }

        impl<'a> SchemaKey<'a> for RecipeKeys {
            fn symbolic_id(self) -> u16 {
                self as u16
            }

            fn name(&self) -> SchemaName<'a> {
                match self {
                    RecipeKeys::MetaNode => SchemaName::Explicit("meta"),
                    // RecipeKeys::MetaNodeName => SchemaName::Explicit("name"),
                    RecipeKeys::Build => SchemaName::Explicit("build"),
                    RecipeKeys::LinksNode => SchemaName::Explicit("links"),
                    RecipeKeys::LinksNodeBugs => SchemaName::Explicit("bugs"),
                }
            }
        }

        pub static ID: SchemaRef = SchemaRef("barney.test.pkg.schema");
        pub static SCHEMA: SchemaDef<RecipeKeys> = SchemaDef::new(ID).children(&[
            &node_def(RecipeKeys::MetaNode)
                .children(Rule::Nodes(&[&node_def(RecipeKeys::LinksNode)
                    .children(Rule::Nodes(&[&node_def(RecipeKeys::LinksNodeBugs)]))])),
            &node_def(RecipeKeys::Build).children(Rule::External(test_script_schema::ID)),
        ]);
    }

    mod test_script_schema {
        //! Testing embedded recursive script schema

        use crate::{Rule, SchemaDef, SchemaKey, SchemaName, SchemaRef, node_def};

        #[repr(u16)]
        #[derive(Debug, Clone, Copy)]
        pub enum TestKeys {
            WithNode,
            ForeachNode,
            InvokeNode,
        }

        impl<'a> SchemaKey<'a> for TestKeys {
            fn symbolic_id(self) -> u16 {
                self as u16
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
            &node_def(TestKeys::WithNode).children(Rule::External(ID)),
            // `foreach` conditionals, recursive schema
            &node_def(TestKeys::ForeachNode).children(Rule::External(ID)),
            // anything else is an invocation
            &node_def(TestKeys::InvokeNode),
        ]);
    }
    // Basic schema const-verification
    #[test]
    fn test_const_schema() {
        assert_eq!(
            test_script_schema::SCHEMA.id(),
            SchemaRef("barney.test.script.schema")
        );
        eprintln!("Recipe structure: {:?}", test_recipe_schema::SCHEMA);
        eprintln!("Script structure: {:?}", test_script_schema::SCHEMA);

        let mut intern = BnyInterner::default();
        let schema = BnySchema::compile(&mut intern, &test_recipe_schema::SCHEMA);
        eprintln!("intern: {intern:?}");
        eprintln!("schema defs: {schema:?}");

        let mut reg = SchemaRegistry::new(test_recipe_schema::ID);
        reg.install(schema)
            .expect("Failed to install recipe schema");
        reg.install(BnySchema::compile(&mut intern, &test_script_schema::SCHEMA))
            .expect("Failed to install script schema");
        let linked = reg.link(&mut intern).expect("Failed to link schemas");
        eprintln!("linked: {linked:?}");
    }
}

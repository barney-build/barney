// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Barney Schema Implementation
//!
//! The bny_schema crate is built atop bny_syntax to
//! provide validation of `.bny` files and rich diagnostics.

mod key;
pub use key::*;

mod schema;
pub use schema::{SchemaDef, SchemaRef};

mod node;
pub use node::{NodeDef, node_def};

mod rule;
pub use rule::Rule;

#[cfg(test)]
mod tests {
    use crate::SchemaRef;

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
    }
}

// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Capability and schemas for loading `.bny` recipe files

use bny_schema::{Rule, SchemaDef, node_def};

use crate::source::schema::SourceKeys;

pub static MAGIC_ID: &str = "bny::recipe";

pub mod schema {
    use bny_schema::{SchemaKey, SchemaName, SchemaRef};

    pub static ID: SchemaRef = SchemaRef("build.barney.source.schema");

    /// All of the possible keys in the source recipe schema
    #[derive(Debug, Clone, Copy)]
    #[repr(u8)]
    pub enum SourceKeys {
        // `recipe` root node
        Root,
        //RecipeName, // TODO: ArgSpec
        /// upstream version
        Version,
        /// release
        Release,
        /// summary shared by `package` nodes
        Summary,
        /// description shared by `package` nodes
        Description,
        /// Static (immutable) properties
        Properties, // TODO: Property schema
        /// Variant definitions
        Variant,
        /// Build requirements
        Requires,
    }

    impl<'a> SchemaKey<'a> for SourceKeys {
        fn symbolic_id(self) -> u8 {
            self as u8
        }

        fn name(&self) -> bny_schema::SchemaName<'a> {
            match self {
                Self::Root => SchemaName::Explicit("recipe"),
                Self::Version => SchemaName::Explicit("version"),
                Self::Release => SchemaName::Explicit("release"),
                Self::Summary => SchemaName::Explicit("summary"),
                Self::Description => SchemaName::Explicit("description"),
                Self::Properties => SchemaName::Explicit("properties"),
                Self::Variant => SchemaName::Explicit("variant"),
                Self::Requires => SchemaName::Explicit("requires"),
            }
        }
    }
}

/// Public source recipe schema
/// Very skeletal right now as we flesh out the schema requirements
pub static SCHEMA: SchemaDef<SourceKeys> =
    SchemaDef::new(schema::ID).children(&[&node_def(SourceKeys::Root).children(Rule::Nodes(&[
        &node_def(SourceKeys::Version),
        &node_def(SourceKeys::Release),
        &node_def(SourceKeys::Summary),
        &node_def(SourceKeys::Description),
        &node_def(SourceKeys::Properties),
        &node_def(SourceKeys::Variant),
        &node_def(SourceKeys::Requires),
    ]))]);

#[cfg(test)]
mod tests {
    use bny_schema::{BnySchema, SchemaRegistry};
    use bny_syntax::{BnyInterner, BnySpec};

    use crate::source::{
        SCHEMA,
        schema::{self, SourceKeys},
    };

    #[test]
    fn load_single_bny() {
        let bny_file = r#"""
            recipe bob {
                version "1.2.1"
            }
        """#;
        let mut registry = SchemaRegistry::new(schema::ID);
        let mut interner = BnyInterner::default();
        registry
            .install(
                BnySchema::compile(&mut interner, &SCHEMA)
                    .expect("could not compile source schema"),
            )
            .expect("Failed to install compiled source schema");
        let linked = registry
            .link(&mut interner)
            .expect("failed to link registry");

        let spec = BnySpec::new(bny_file);
        let tree = spec.process(&mut interner);
        let validated = linked.validate(&tree, &interner);

        let recipe = validated
            .nodes(SourceKeys::Root)
            .expect("Failed to find recipes")
            .collect::<Vec<_>>();
        assert_eq!(recipe.len(), 1, "Couldn't find root recipe");

        let version = recipe[0]
            .nodes(SourceKeys::Version)
            .expect("Couldn't find version")
            .collect::<Vec<_>>();
        assert_eq!(version.len(), 1, "Couldn't find version inside recipe");
    }
}

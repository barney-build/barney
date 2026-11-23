// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Capability and schemas for loading `.bny` recipe files

use bny_schema::{ArgumentDef, NodeDef, Occurrence, PropertyDef, Rule, SchemaDef};

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
        RecipeName,
        /// upstream version
        Version,
        VersionVal,
        /// release
        Release,
        /// summary shared by `package` nodes
        Summary,
        /// description shared by `package` nodes
        Description,
        DescriptionVal,
        /// Static (immutable) properties
        Properties, // TODO: Property schema
        /// Variant definitions
        Variant,
        /// Build requirements
        Requires,
        /// where to grab sources
        Sources,

        // URI arg0
        SourceFetchURI,
        SourceFetchSha256sum,

        SourceFetch,
        VersionArg0,
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
                Self::Sources => SchemaName::Explicit("sources"),
                Self::SourceFetch => SchemaName::Explicit("fetch"),
                Self::SourceFetchSha256sum => SchemaName::Explicit("sha256sum"),
                _ => SchemaName::Dynamic,
            }
        }
    }
}

/// Public source recipe schema
/// Very skeletal right now as we flesh out the schema requirements
pub static SCHEMA: SchemaDef<SourceKeys> =
    SchemaDef::new(schema::ID).children(&[&NodeDef::new(SourceKeys::Root)
        .occurrence(Occurrence::exactly(1))
        .arguments(&[ArgumentDef::positional::<&str>(SourceKeys::RecipeName)])
        .children(Rule::Nodes(&[
            &NodeDef::new(SourceKeys::Version)
                .occurrence(Occurrence::exactly(1))
                .arguments(&[ArgumentDef::positional::<&str>(SourceKeys::VersionVal)]),
            &NodeDef::new(SourceKeys::Release).occurrence(Occurrence::exactly(1)),
            &NodeDef::new(SourceKeys::Summary).occurrence(Occurrence::exactly(1)),
            &NodeDef::new(SourceKeys::Description)
                .occurrence(Occurrence::minimum(1))
                .arguments(&[ArgumentDef::variadic::<&str>(SourceKeys::DescriptionVal)]),
            &NodeDef::new(SourceKeys::Properties).occurrence(Occurrence::maximum(1)),
            &NodeDef::new(SourceKeys::Variant),
            &NodeDef::new(SourceKeys::Requires).occurrence(Occurrence::maximum(1)),
            &NodeDef::new(SourceKeys::Sources)
                .occurrence(Occurrence::maximum(1))
                .children(Rule::Nodes(&[&NodeDef::new(SourceKeys::SourceFetch)
                    .arguments(&[ArgumentDef::positional::<&str>(SourceKeys::SourceFetchURI)])
                    .properties(&[PropertyDef::new::<&str>(
                        SourceKeys::SourceFetchSha256sum,
                    )])])),
        ]))]);

#[cfg(test)]
mod tests {
    use bny_diag::DiagnosticSink;
    use bny_schema::{BnySchema, SchemaRegistry};
    use bny_syntax::{BnyInterner, BnySpec};
    use miette::NamedSource;

    use crate::source::{
        SCHEMA,
        schema::{
            self,
            SourceKeys::{self, SourceFetch, SourceFetchSha256sum},
        },
    };

    #[test]
    fn load_single_bny() {
        let bny_file = r#"
            recipe bob {
                version "ya"
                release bob

                summary "Some kinda summary"
                description Some kinda description
                sources {
                    fetch "https://blahblah.org" sha256sum="i r a hashsum"
                }
            }
        "#;
        let mut registry = SchemaRegistry::new(schema::ID);
        let mut interner = BnyInterner::default();
        let mut sink = DiagnosticSink::new();

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
        let validated = linked.validate(&tree, &interner, &mut sink);
        for blah in sink.iter() {
            eprintln!(
                "report = {:?}",
                miette::Report::new(blah.as_miette(&sink))
                    .with_source_code(NamedSource::new("recipe.bny", bny_file))
            );
        }

        let recipe = validated
            .nodes(SourceKeys::Root)
            .expect("Failed to find recipes")
            .collect::<Vec<_>>();
        assert_eq!(recipe.len(), 1, "Couldn't find root recipe");
        let name: &str = recipe[0]
            .argument(SourceKeys::RecipeName)
            .expect("couldn't fetch name");
        assert_eq!(name, "bob");

        let version = recipe[0]
            .nodes(SourceKeys::Version)
            .expect("Couldn't find version")
            .collect::<Vec<_>>();
        assert_eq!(version.len(), 1, "Couldn't find version inside recipe");

        let source = recipe[0]
            .nodes(SourceKeys::Sources)
            .expect("Cannot get sources")
            .collect::<Vec<_>>();
        let fetch = source[0].nodes(SourceFetch).unwrap().next().unwrap();
        let sha = fetch
            .property::<&str>(SourceFetchSha256sum)
            .expect("Could not get sha256sum property");
        assert_eq!(sha, "i r a hashsum", "invalid return property");
    }
}

// SPDX-FileCopyrightText: 2026 Ikey Doherty
// SPDX-License-Identifier: MPL-2.0

//! Processing of KDL / DSL

use itertools::{Either, Itertools};
use kdl::{KdlDocument, KdlNode};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

use crate::{
    ArgSpec, Error, NodeDescent, NodeName, NodeSpec, NodeTag, NodeTagPolicy, ProcessedEntry,
    ProcessedNode,
};

/// Process KDL according to the given rule set
pub fn process_kdl<'a, I>(
    document: &'a KdlDocument,
    rules: &[&NodeSpec<'a, I>],
) -> Result<Vec<ProcessedNode<'a, I>>, Error>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    let mut ruleset = rules.iter().cloned().collect_vec();
    let mut results = vec![];

    // Toplevel descent + entry
    for node in document.nodes() {
        // Recurse children not siblings
        results.push(process_kdl_node(node, &mut ruleset)?);
    }

    Ok(results)
}

/// Process a single KDL node according to rules and if successful, return built
/// nodes according to our DSL requirements
/// If a rule is "spent" in the current context, remove it from the input rules
pub(crate) fn process_kdl_node<'a, I>(
    node: &'a KdlNode,
    rules: &mut Vec<&NodeSpec<'a, I>>,
) -> Result<ProcessedNode<'a, I>, Error>
where
    I: Into<usize> + Eq + PartialEq + PartialOrd + Hash + Debug + Clone,
{
    let mut rule_index = None;
    for (index, rule) in rules.iter().enumerate() {
        match rule.name {
            // Require explicit name match
            NodeName::Static(n) => {
                if n == node.name().value() {
                    rule_index = Some(index);
                    break;
                }
            }
            // Dynamic means it must match now
            NodeName::Dynamic => {
                rule_index = Some(index);
                break;
            }
        }
    }

    // Is this a legal identifier per nodespec?
    let idx = rule_index.ok_or_else(|| Error::UnexpectedIdentifier {
        span: node.span(),
        id: node.name().to_string(),
    })?;
    let rule = rules.get(idx).unwrap();

    // bake a property map and argument set
    let (properties, args): (HashMap<String, _>, Vec<_>) =
        node.entries().iter().partition_map(|n| {
            if let Some(name) = n.name() {
                Either::Left((name.to_string(), n))
            } else {
                Either::Right(n)
            }
        });

    // Ensure argument policy is sane
    match rule.args {
        // No arguments allowed?
        ArgSpec::None => {
            if !args.is_empty() {
                return Err(Error::WrongArgumentCount {
                    span: node.span(),
                    expected: 0,
                    found: args.len(),
                });
            }
        }
        // Only N args
        ArgSpec::Exactly(n) => {
            if n != args.len() {
                return Err(Error::WrongArgumentCount {
                    span: node.span(),
                    expected: n,
                    found: args.len(),
                });
            }
        }
        ArgSpec::Variable => {}
    }

    // check property sanity
    // TODO: Permit dynamic property matching like for arguments
    // TODO: Handle duplicate rules
    for (id, prop) in properties.iter() {
        let _ = rule
            .props
            .iter()
            .find_position(|r| r.name == id)
            .ok_or_else(|| Error::UnknownProperty {
                span: prop.span(),
                name: id.clone(),
            })?;
    }

    let mut children = vec![];

    // Recurse the child with subset of expendable rules
    let mut child_rules = rule.children.iter().collect_vec();
    match rule.descent {
        NodeDescent::Normal => {
            // Process child per rules
            for child in node.iter_children() {
                children.push(process_kdl_node(child, &mut child_rules)?);
            }
        }
        NodeDescent::Never => {}
    };

    // Full baked node

    // Process arguments into typed values
    let mut processed_args = vec![];
    for arg in args.into_iter() {
        let proc: ProcessedEntry = arg.try_into()?;
        processed_args.push(proc);
    }

    // TODO: Handle tag validation!
    let tag: Option<NodeTag> = if let Some(ty) = node.ty() {
        Some(ty.try_into()?)
    } else {
        None
    };

    match rule.tags {
        // no tags allowed.
        NodeTagPolicy::None => {
            if let Some(tag) = tag {
                return Err(Error::IllegalTypeAttribute {
                    span: node.span(),
                    tag: tag.to_string(),
                });
            }
        }
        // optional but ONE OF given set
        NodeTagPolicy::Optional(node_tags) => {
            if let Some(tag) = &tag {
                match node_tags.iter().find(|n| *n == tag) {
                    Some(_) => {}
                    None => {
                        return Err(Error::WrongTypeAttribute {
                            span: node.span(),
                            tag: tag.to_string(),
                        });
                    }
                }
            }
        }
        NodeTagPolicy::Required(node_tags) => {
            if let Some(tag) = &tag {
                match node_tags.iter().find(|n| *n == tag) {
                    Some(_) => {}
                    None => {
                        return Err(Error::WrongTypeAttribute {
                            span: node.span(),
                            tag: tag.to_string(),
                        });
                    }
                }
            } else {
                return Err(Error::MissingTypeAttribute { span: node.span() });
            }
        }
    }

    Ok(ProcessedNode {
        identity: rule.identity.clone(),
        tag,
        args: processed_args,
        props: properties
            .into_iter()
            .map(|(k, v)| (k, v.value().to_string()))
            .collect(),
        children,
        node,
    })
}

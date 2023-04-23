//! Defines JSONPath query structure and parsing logic.
//!
//! # Examples
//! To create a query from a query string:
//! ```
//! # use rsonpath_lib::query::{JsonPathQuery, JsonPathQueryNode, JsonPathQueryNodeType};
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let query_string = "$..phoneNumbers[*].number";
//! let query = JsonPathQuery::parse(query_string)?;
//!
//! // Query structure is a linear sequence of nodes:
//! // Root '$', descendant '..phoneNumbers', child wildcard, child 'number'.
//! let root_node = query.root();
//! let descendant_node = root_node.child().unwrap();
//! let child_wildcard_node = descendant_node.child().unwrap();
//! let child_node = child_wildcard_node.child().unwrap();
//!
//! assert!(root_node.is_root());
//! assert!(descendant_node.is_descendant());
//! assert!(child_wildcard_node.is_any_child());
//! assert!(child_node.is_child());
//! // Final node will have a None child.
//! assert!(child_node.child().is_none());
//!
//! assert_eq!(descendant_node.label().unwrap(), "phoneNumbers".as_bytes());
//! assert_eq!(child_wildcard_node.label(), None);
//! assert_eq!(child_node.label().unwrap(), "number".as_bytes());
//! # Ok(())
//! # }
//! ```
pub mod automaton;
pub mod builder;
pub mod error;
mod label;
mod parser;
pub use label::Label;

use log::*;
use std::fmt::{self, Display};

/// Linked list structure of a JSONPath query.
#[derive(Debug, PartialEq, Eq)]
pub enum JsonPathQueryNode {
    /// The first link in the list representing the root '`$`' character.
    Root(Option<Box<JsonPathQueryNode>>),
    /// Represents direct descendant with a label ('`.`' token).
    Child(Label, Option<Box<JsonPathQueryNode>>),
    /// Represents direct descendant with a wildcard ('`.*`' tokens).
    AnyChild(Option<Box<JsonPathQueryNode>>),
    /// Represents recursive descent ('`..`' token).
    Descendant(Label, Option<Box<JsonPathQueryNode>>),
    /// Represents recursive descendant with a wildcard ('`..*`' tokens).
    AnyDescendant(Option<Box<JsonPathQueryNode>>),
}

use JsonPathQueryNode::*;

use self::error::ParserError;

impl JsonPathQueryNode {
    /// Retrieve the child of the node or `None` if it is the last one
    /// on the list.
    #[must_use]
    #[inline(always)]
    pub fn child(&self) -> Option<&Self> {
        match self {
            Root(node)
            | Child(_, node)
            | AnyChild(node)
            | Descendant(_, node)
            | AnyDescendant(node) => node.as_deref(),
        }
    }

    /// Create an iterator over nodes of the query in sequence,
    /// starting from the root.
    #[must_use]
    #[inline(always)]
    pub fn iter(&self) -> JsonPathQueryIterator {
        JsonPathQueryIterator { node: Some(self) }
    }
}

/// JSONPath query structure represented by the root link of the
/// [`JsonPathQueryNode`] list.
#[derive(Debug, PartialEq, Eq)]
pub struct JsonPathQuery {
    root: Box<JsonPathQueryNode>,
}

/// Iterator over query nodes traversing the parent-child relation.
pub struct JsonPathQueryIterator<'a> {
    node: Option<&'a JsonPathQueryNode>,
}

impl<'a> Iterator for JsonPathQueryIterator<'a> {
    type Item = &'a JsonPathQueryNode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.node;

        if let Some(node) = result {
            self.node = node.child()
        }

        result
    }
}

impl JsonPathQuery {
    /// Retrieve reference to the root node.
    ///
    /// It is guaranteed that the root is the [`JsonPathQueryNode::Root`]
    /// variant and always exists.
    #[must_use]
    #[inline(always)]
    pub fn root(&self) -> &JsonPathQueryNode {
        self.root.as_ref()
    }

    /// Parse a query string into a [`JsonPathQuery`].
    ///
    /// # Errors
    ///
    /// Will return a [`ParserError`] if the `query_string` does
    /// not conform to the JSONPath grammar. See its documentation
    /// for details.
    #[inline(always)]
    pub fn parse(query_string: &str) -> Result<Self, ParserError> {
        self::parser::parse_json_path_query(query_string)
    }

    /// Create a query from a root node.
    ///
    /// If node is not the [`JsonPathQueryNode::Root`] variant it will be
    /// automatically wrapped into a [`JsonPathQueryNode::Root`] node.
    #[inline]
    #[must_use]
    pub fn new(node: Box<JsonPathQueryNode>) -> Self {
        let root = if node.is_root() {
            node
        } else {
            info!("Implicitly using the Root expression (`$`) at the start of the query.");
            Box::new(Root(Some(node)))
        };

        Self { root }
    }
}

impl Display for JsonPathQuery {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root.as_ref())
    }
}

impl Display for JsonPathQueryNode {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Root(_) => write!(f, "$"),
            Child(label, _) => write!(f, "['{}']", label.display()),
            AnyChild(_) => write!(f, "[*]"),
            Descendant(label, _) => write!(f, "..['{}']", label.display()),
            AnyDescendant(_) => write!(f, "..[*]"),
        }?;

        if let Some(child) = self.child() {
            write!(f, "{child}")
        } else {
            Ok(())
        }
    }
}

/// Equips a struct with information on the type of [`JsonPathQueryNode`] it represents
/// and methods to extract query elements from it.
pub trait JsonPathQueryNodeType {
    /// Returns `true` iff the type is [`JsonPathQueryNode::Root`].
    fn is_root(&self) -> bool;

    /// Returns `true` iff the type is [`JsonPathQueryNode::Descendant`].
    fn is_descendant(&self) -> bool;

    /// Returns `true` iff the type is [`JsonPathQueryNode::AnyDescendant`].
    fn is_any_descendant(&self) -> bool;

    /// Returns `true` iff the type is [`JsonPathQueryNode::Child`].
    fn is_child(&self) -> bool;

    /// Returns `true` iff the type is [`JsonPathQueryNode::AnyChild`].
    fn is_any_child(&self) -> bool;

    /// If the type is [`JsonPathQueryNode::Descendant`] or [`JsonPathQueryNode::Child`]
    /// returns the label it represents; otherwise, `None`.
    fn label(&self) -> Option<&Label>;
}

impl JsonPathQueryNodeType for JsonPathQueryNode {
    #[inline(always)]
    fn is_root(&self) -> bool {
        matches!(self, Root(_))
    }

    #[inline(always)]
    fn is_descendant(&self) -> bool {
        matches!(self, Descendant(_, _))
    }

    #[inline(always)]
    fn is_any_descendant(&self) -> bool {
        matches!(self, AnyDescendant(_))
    }

    #[inline(always)]
    fn is_child(&self) -> bool {
        matches!(self, Child(_, _))
    }

    #[inline(always)]
    fn is_any_child(&self) -> bool {
        matches!(self, AnyChild(_))
    }

    #[inline(always)]
    fn label(&self) -> Option<&Label> {
        match self {
            Child(label, _) | Descendant(label, _) => Some(label),
            Root(_) | AnyChild(_) | AnyDescendant(_) => None,
        }
    }
}

/// Utility blanket implementation for a [`JsonPathQueryNode`] wrapped in an [`Option`].
///
/// If the value is `None` automatically returns `false` or `None` on all calls in
/// the natural manner.
impl<T: std::ops::Deref<Target = JsonPathQueryNode>> JsonPathQueryNodeType for Option<T> {
    #[inline(always)]
    fn is_root(&self) -> bool {
        self.as_ref().map_or(false, |x| x.is_root())
    }

    #[inline(always)]
    fn is_descendant(&self) -> bool {
        self.as_ref().map_or(false, |x| x.is_descendant())
    }

    #[inline(always)]
    fn is_any_descendant(&self) -> bool {
        self.as_ref().map_or(false, |x| x.is_any_descendant())
    }

    #[inline(always)]
    fn is_child(&self) -> bool {
        self.as_ref().map_or(false, |x| x.is_child())
    }

    #[inline(always)]
    fn is_any_child(&self) -> bool {
        self.as_ref().map_or(false, |x| x.is_any_child())
    }

    #[inline(always)]
    fn label(&self) -> Option<&Label> {
        self.as_ref().and_then(|x| x.label())
    }
}

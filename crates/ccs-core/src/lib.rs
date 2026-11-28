//! Shared, side-effect-free core for Clean Code Shit 2.0.
//!
//! Milestone 1 provides only the typed, schema-versioned catalog and its
//! validation. Nothing here touches the filesystem, launches a process, or
//! renders a UI. The CLI and the future desktop app will build on this data.

#![forbid(unsafe_code)]

pub mod catalog;

pub use catalog::{
    ActionId, BUILTIN_CATALOG, Catalog, CatalogError, Platform, Reference, Risk, RootKey,
    SCHEMA_VERSION, Source, Target, WindowsItem, WindowsStatus,
};

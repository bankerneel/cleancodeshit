//! Shared core for Clean Code Shit 2.0.
//!
//! Includes the typed catalog and a Windows filesystem adapter. The adapter
//! works on named entries below a pinned profile and exposes handle-based
//! operations for a future planner.

#![deny(unsafe_code)]

pub mod catalog;
#[cfg(windows)]
pub mod windows_fs;

pub use catalog::{
    ActionId, BUILTIN_CATALOG, Catalog, CatalogError, Platform, Reference, Risk, RootKey,
    SCHEMA_VERSION, Source, Target, WindowsItem, WindowsStatus,
};

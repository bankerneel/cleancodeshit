//! Strict, typed schema for the shared TOML catalog.
//!
//! The catalog is *data only*. Root keys, action identifiers, platforms and
//! statuses are interpreted by Rust; catalog data cannot carry shell snippets,
//! absolute paths or an override of the approved home boundary. Every table
//! denies unknown fields so a typo in the data is a parse error rather than a
//! silently ignored instruction.

use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::{Deserialize, Serialize};

/// The only catalog schema version this parser accepts.
pub const SCHEMA_VERSION: u32 = 1;

/// The compiled-in catalog text.
pub const BUILTIN_CATALOG: &str = include_str!("../catalog/items.toml");

/// A schema-versioned catalog of current Bash items and planned Windows entries.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub windows: Vec<WindowsItem>,
}

/// A current Bash item and its macOS and Linux availability.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub group: String,
    pub label: String,
    pub risk: Risk,
    pub action_id: ActionId,
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub roots: Vec<Target>,
    #[serde(default)]
    pub must_close: Vec<String>,
    pub docs: Reference,
}

/// A Windows-specific item. It carries an availability `status` instead of a
/// `platforms` list because it is implicitly Windows-only.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WindowsItem {
    pub id: String,
    pub group: String,
    pub label: String,
    pub risk: Risk,
    pub action_id: ActionId,
    #[serde(default)]
    pub roots: Vec<Target>,
    #[serde(default)]
    pub must_close: Vec<String>,
    pub docs: Reference,
    pub status: WindowsStatus,
}

/// A root-relative target pattern. `path` is data interpreted later by the
/// platform adapters; `overrides` are the names of environment/configuration
/// variables the adapter may honour, never shell snippets.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub root: RootKey,
    pub path: String,
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub overrides: Vec<String>,
}

/// Documentation and citation metadata for an item.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub section: String,
    pub path: String,
    pub action: String,
    #[serde(default)]
    pub sources: Vec<Source>,
    #[serde(default)]
    pub source_note: Option<String>,
}

/// A single official source link.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub label: String,
    pub url: String,
}

/// Cleanup risk tier. `safe` items are selectable by the safe preset; danger
/// items require explicit typed consent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Safe,
    Caution,
    Danger,
}

impl Risk {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Caution => "caution",
            Self::Danger => "danger",
        }
    }
}

impl fmt::Display for Risk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Supported operating systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Linux,
    Macos,
    Windows,
}

impl Platform {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Windows => "windows",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Availability of a Windows item in the preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowsStatus {
    Planned,
    Deferred,
    Excluded,
    InventoryOnly,
}

impl WindowsStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Deferred => "deferred",
            Self::Excluded => "excluded",
            Self::InventoryOnly => "inventory_only",
        }
    }
}

impl fmt::Display for WindowsStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed root keys. Rust maps these to approved OS roots; catalog data cannot
/// invent a root or change the home boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RootKey {
    Home,
    CacheHome,
    ConfigHome,
    DataHome,
    LocalAppData,
    RoamingAppData,
    Temp,
    NpmCache,
    PnpmStore,
    YarnCache,
    GoCache,
    DockerEndpoint,
    StaleProject,
    NugetCache,
}

impl RootKey {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::CacheHome => "cache_home",
            Self::ConfigHome => "config_home",
            Self::DataHome => "data_home",
            Self::LocalAppData => "local_app_data",
            Self::RoamingAppData => "roaming_app_data",
            Self::Temp => "temp",
            Self::NpmCache => "npm_cache",
            Self::PnpmStore => "pnpm_store",
            Self::YarnCache => "yarn_cache",
            Self::GoCache => "go_cache",
            Self::DockerEndpoint => "docker_endpoint",
            Self::StaleProject => "stale_project",
            Self::NugetCache => "nuget_cache",
        }
    }
}

impl fmt::Display for RootKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed action identifiers. The executor will match on these; the catalog
/// can only select an action, never provide executable text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionId {
    RemoveEntries,
    NpmCacheClean,
    PnpmStorePrune,
    UvCachePrune,
    GoCacheClean,
    BrewCleanup,
    DockerRemoveAllContainers,
    DockerPruneStoppedContainers,
    DockerRemoveMatchingImages,
    DockerPruneImages,
    DockerPruneBuildCache,
    DockerPruneNetworks,
    DockerPruneVolumes,
    RemoveStaleArtifacts,
    NugetClearHttpCache,
    NugetClearPluginsCache,
    NugetClearTemp,
    InventoryOnly,
    Unavailable,
}

impl ActionId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RemoveEntries => "remove_entries",
            Self::NpmCacheClean => "npm_cache_clean",
            Self::PnpmStorePrune => "pnpm_store_prune",
            Self::UvCachePrune => "uv_cache_prune",
            Self::GoCacheClean => "go_cache_clean",
            Self::BrewCleanup => "brew_cleanup",
            Self::DockerRemoveAllContainers => "docker_remove_all_containers",
            Self::DockerPruneStoppedContainers => "docker_prune_stopped_containers",
            Self::DockerRemoveMatchingImages => "docker_remove_matching_images",
            Self::DockerPruneImages => "docker_prune_images",
            Self::DockerPruneBuildCache => "docker_prune_build_cache",
            Self::DockerPruneNetworks => "docker_prune_networks",
            Self::DockerPruneVolumes => "docker_prune_volumes",
            Self::RemoveStaleArtifacts => "remove_stale_artifacts",
            Self::NugetClearHttpCache => "nuget_clear_http_cache",
            Self::NugetClearPluginsCache => "nuget_clear_plugins_cache",
            Self::NugetClearTemp => "nuget_clear_temp",
            Self::InventoryOnly => "inventory_only",
            Self::Unavailable => "unavailable",
        }
    }
}

impl fmt::Display for ActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a catalog could not be accepted.
#[derive(Debug)]
pub enum CatalogError {
    Parse(toml::de::Error),
    UnsupportedSchemaVersion { found: u32, expected: u32 },
    DuplicateIdentifier { id: String, scope: String },
    InvalidItem { id: String, reason: &'static str },
    InvalidTargetPath { id: String, path: String },
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "invalid catalog TOML: {error}"),
            Self::UnsupportedSchemaVersion { found, expected } => {
                write!(f, "unsupported schema version {found}; expected {expected}")
            }
            Self::DuplicateIdentifier { id, scope } => {
                write!(f, "duplicate item identifier `{id}` in {scope}")
            }
            Self::InvalidItem { id, reason } => write!(f, "item `{id}`: {reason}"),
            Self::InvalidTargetPath { id, path } => {
                write!(f, "item `{id}` has non-relative target path `{path}`")
            }
        }
    }
}

impl std::error::Error for CatalogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            _ => None,
        }
    }
}

impl From<toml::de::Error> for CatalogError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error)
    }
}

impl Catalog {
    /// Parse and validate a catalog from TOML text.
    pub fn parse(input: &str) -> Result<Self, CatalogError> {
        let catalog: Self = toml::from_str(input)?;
        catalog.validate()?;
        Ok(catalog)
    }

    /// Parse and validate the compiled-in catalog.
    pub fn builtin() -> Result<Self, CatalogError> {
        Self::parse(BUILTIN_CATALOG)
    }

    fn validate(&self) -> Result<(), CatalogError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(CatalogError::UnsupportedSchemaVersion {
                found: self.schema_version,
                expected: SCHEMA_VERSION,
            });
        }

        // An identifier may occur at most once per platform. Windows entries
        // live in their own list, so they are checked separately.
        let mut seen: HashMap<Platform, HashSet<&str>> = HashMap::new();
        for item in &self.items {
            if item.platforms.is_empty() || item.platforms.contains(&Platform::Windows) {
                return Err(invalid_item(
                    &item.id,
                    "expected macOS or Linux availability",
                ));
            }
            for platform in &item.platforms {
                if !seen.entry(*platform).or_default().insert(item.id.as_str()) {
                    return Err(CatalogError::DuplicateIdentifier {
                        id: item.id.clone(),
                        scope: format!("platform {}", platform),
                    });
                }
            }
        }

        let mut windows_ids: HashSet<&str> = HashSet::new();
        for item in &self.windows {
            if !windows_ids.insert(item.id.as_str()) {
                return Err(CatalogError::DuplicateIdentifier {
                    id: item.id.clone(),
                    scope: "windows list".to_string(),
                });
            }
            let valid_status = match item.status {
                WindowsStatus::Planned => {
                    !item.roots.is_empty()
                        && !matches!(
                            item.action_id,
                            ActionId::Unavailable | ActionId::InventoryOnly
                        )
                }
                WindowsStatus::Deferred | WindowsStatus::Excluded => {
                    item.roots.is_empty() && item.action_id == ActionId::Unavailable
                }
                WindowsStatus::InventoryOnly => item.action_id == ActionId::InventoryOnly,
            };
            if !valid_status {
                return Err(invalid_item(
                    &item.id,
                    "action and roots must match Windows status",
                ));
            }
        }

        for item in &self.items {
            for target in &item.roots {
                validate_target(&item.id, target)?;
            }
        }
        for item in &self.windows {
            for target in &item.roots {
                validate_target(&item.id, target)?;
            }
        }

        Ok(())
    }
}

fn invalid_item(id: &str, reason: &'static str) -> CatalogError {
    CatalogError::InvalidItem {
        id: id.to_owned(),
        reason,
    }
}

/// Reject anything that could escape the approved root. Paths are patterns
/// relative to a typed root, never absolute paths or drive/UNC references.
fn validate_target(owner: &str, target: &Target) -> Result<(), CatalogError> {
    if is_relative_pattern(&target.path) {
        Ok(())
    } else {
        Err(CatalogError::InvalidTargetPath {
            id: owner.to_string(),
            path: target.path.clone(),
        })
    }
}

fn is_relative_pattern(path: &str) -> bool {
    // An empty pattern addresses the typed root itself and is relative.
    path.is_empty()
        || (!path.contains(['\\', ':'])
            && !path.chars().any(char::is_control)
            && path.split('/').all(|part| !matches!(part, "" | "." | "..")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_block(id: &str, platform: &str) -> String {
        format!(
            r#"
[[items]]
id = "{id}"
group = "g"
label = "L"
risk = "safe"
action_id = "remove_entries"
platforms = ["{platform}"]

[[items.roots]]
root = "home"
path = ".cache/example"

[items.docs]
section = "s"
path = "p"
action = "a"
"#
        )
    }

    fn catalog_with(extra: &str) -> String {
        format!("schema_version = 1\n{extra}")
    }

    #[test]
    fn builtin_catalog_parses_with_expected_version() {
        let catalog = Catalog::builtin().expect("compiled-in catalog must be valid");
        assert_eq!(catalog.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn enum_strings_are_snake_case() {
        assert_eq!(Risk::Safe.as_str(), "safe");
        assert_eq!(Risk::Caution.to_string(), "caution");
        assert_eq!(Risk::Danger.as_str(), "danger");
        assert_eq!(WindowsStatus::Planned.as_str(), "planned");
        assert_eq!(WindowsStatus::InventoryOnly.to_string(), "inventory_only");
    }

    #[test]
    fn unknown_field_is_rejected() {
        let input = catalog_with(&format!("{}\nbogus = 1\n", item_block("a", "linux")));
        assert!(matches!(
            Catalog::parse(&input),
            Err(CatalogError::Parse(_))
        ));
    }

    #[test]
    fn wrong_schema_version_is_rejected() {
        let input = format!("schema_version = 2\n{}", item_block("a", "linux"));
        assert!(matches!(
            Catalog::parse(&input),
            Err(CatalogError::UnsupportedSchemaVersion { found: 2, .. })
        ));
    }

    #[test]
    fn duplicate_identifier_per_platform_is_rejected() {
        let input = catalog_with(&format!(
            "{}{}",
            item_block("dup", "linux"),
            item_block("dup", "linux")
        ));
        assert!(matches!(
            Catalog::parse(&input),
            Err(CatalogError::DuplicateIdentifier { .. })
        ));
    }

    #[test]
    fn same_identifier_on_different_platforms_is_allowed() {
        let input = catalog_with(&format!(
            "{}{}",
            item_block("shared", "linux"),
            item_block("shared", "macos")
        ));
        assert!(Catalog::parse(&input).is_ok());
    }

    #[test]
    fn non_relative_paths_are_rejected() {
        for path in [
            "/etc",
            "../escape",
            "a/../b",
            "a//b",
            "a/./b",
            "a/file:stream",
            "C:\\cache",
            "\\\\server\\share",
        ] {
            let input = catalog_with(&format!(
                r#"
[[items]]
id = "a"
group = "g"
label = "L"
risk = "safe"
action_id = "remove_entries"
platforms = ["linux"]

[[items.roots]]
root = "home"
path = '{path}'

[items.docs]
section = "s"
path = "p"
action = "a"
"#
            ));
            assert!(
                matches!(
                    Catalog::parse(&input),
                    Err(CatalogError::InvalidTargetPath { .. })
                ),
                "path {path:?} should be rejected"
            );
        }
    }

    #[test]
    fn invalid_windows_status_is_rejected() {
        let input = catalog_with(
            r#"
[[windows]]
id = "w"
group = "g"
label = "L"
risk = "safe"
action_id = "unavailable"
status = "unknown_status"

[windows.docs]
section = "s"
path = "p"
action = "a"
"#,
        );
        assert!(matches!(
            Catalog::parse(&input),
            Err(CatalogError::Parse(_))
        ));
    }

    #[test]
    fn excluded_windows_item_cannot_carry_cleanup_action() {
        let input = catalog_with(
            r#"
[[windows]]
id = "w"
group = "g"
label = "L"
risk = "danger"
action_id = "remove_entries"
status = "excluded"

[windows.docs]
section = "s"
path = "p"
action = "a"
"#,
        );
        assert!(matches!(
            Catalog::parse(&input),
            Err(CatalogError::InvalidItem { .. })
        ));
    }

    #[test]
    fn catalog_serializes_to_json_for_the_generator() {
        let catalog = Catalog::builtin().expect("compiled-in catalog must be valid");
        let json = serde_json::to_string(&catalog).expect("catalog is serializable");
        assert!(json.contains("schema_version"));
    }
}

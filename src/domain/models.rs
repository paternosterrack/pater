use crate::cli::InstallScope;
use serde::{Deserialize, Serialize};

fn default_scope() -> InstallScope {
    InstallScope::User
}

#[derive(Serialize)]
pub struct JsonOut<T: Serialize> {
    pub ok: bool,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct State {
    pub marketplaces: Vec<MarketRef>,
    pub installed: Vec<InstalledPlugin>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MarketRef {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstalledPlugin {
    pub name: String,
    pub marketplace: String,
    #[serde(default)]
    pub marketplace_source: String,
    pub source: String,
    /// Runtime materialization path managed by pater.
    /// Kept as `local_path` for backward-compatible state/lock schema.
    #[serde(default)]
    pub local_path: String,
    pub version: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default = "default_scope")]
    pub scope: InstallScope,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub version: u32,
    pub plugins: Vec<InstalledPlugin>,
}

#[derive(Debug, Deserialize)]
pub struct PolicyFile {
    #[serde(default)]
    pub general: PolicyGeneral,
}

#[derive(Debug, Deserialize, Default)]
pub struct PolicyGeneral {
    #[serde(default)]
    pub require_signed_marketplace: bool,
    #[serde(default)]
    pub allowed_sources: Vec<String>,
    #[serde(default)]
    pub denied_plugins: Vec<String>,
    #[serde(default)]
    pub blocked_permissions: Vec<String>,
    #[serde(default)]
    pub block_unknown_licenses: bool,
    #[serde(default)]
    pub allow_unknown_license_plugins: Vec<String>,
    #[serde(default)]
    pub allow_external_reference_installs: bool,
    #[serde(default)]
    pub allow_external_reference_plugins: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct DiscoverItem {
    pub marketplace: String,
    pub marketplace_source: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub source: String,
    pub distribution: Option<String>,
    pub license_status: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct Recommendation {
    pub plugin: String,
    pub marketplace: String,
    pub score: i32,
    pub reason: String,
    pub permission_count: usize,
    pub distribution: Option<String>,
    pub license_status: Option<String>,
}

#[derive(Serialize)]
pub struct UpdateReport {
    pub name: String,
    pub status: String,
    pub old_version: Option<String>,
    pub new_version: Option<String>,
    pub added_permissions: Vec<String>,
}

#[derive(Serialize)]
pub struct SmokeReport {
    pub adapter: String,
    pub status: String,
    pub checked_plugins: usize,
    pub missing_plugins: Vec<String>,
}

#[derive(Serialize)]
pub struct CheckItem {
    pub name: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct DoctorReport {
    pub overall: String,
    pub path_has_local_bin: bool,
    pub smoke: Vec<SmokeReport>,
    pub configs: Vec<CheckItem>,
    pub wrappers: Vec<CheckItem>,
}

#[derive(Serialize)]
pub struct CapabilitiesReport {
    pub installed_count: usize,
    pub installed_plugins: Vec<String>,
    pub adapter_smoke: Vec<SmokeReport>,
}

#[derive(Serialize)]
pub struct PolicyEvalReport {
    pub plugin: String,
    pub agent: String,
    pub allowed: bool,
    pub reason: String,
}

#[derive(Serialize)]
pub struct PlanReport {
    pub intent: String,
    pub agent: String,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Serialize)]
pub struct TrustStatus {
    pub require_signed_marketplace: bool,
    pub trusted_key_count: usize,
    pub default_marketplace: String,
    pub default_marketplace_signature_ok: bool,
}

#[derive(Serialize)]
pub struct ReleaseCheckReport {
    pub overall: String,
    pub trust: TrustStatus,
    pub doctor: DoctorReport,
    pub rack_license_audit: String,
    pub recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RackLicenseAuditSummary {
    pub permissive: usize,
    pub copyleft: usize,
    pub unknown_count: usize,
    pub total: usize,
}

#[derive(Serialize)]
pub struct RackDoctorReport {
    pub overall: String,
    pub checks: Vec<CheckItem>,
}

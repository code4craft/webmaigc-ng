use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrawlerProject {
    pub manifest: ProjectManifest,
    pub runtime: RuntimeContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectManifest {
    pub id: ProjectId,
    pub version: ProjectVersion,
    pub title: String,
    pub description: Option<String>,
    pub target: TargetMetadata,
    pub inputs: Vec<InputParameter>,
    pub outputs: Vec<OutputField>,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectId(pub String);

impl ProjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectVersion(pub String);

impl ProjectVersion {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectReference {
    pub project_id: ProjectId,
    pub version: ProjectVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetMetadata {
    pub site: TargetSite,
    pub start_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetSite {
    pub name: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputParameter {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub parameter_type: ParameterType,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Integer,
    Number,
    Boolean,
    StringList,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputField {
    pub name: String,
    pub description: Option<String>,
    pub field_type: ParameterType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeContract {
    pub entrypoint: Entrypoint,
    pub dependencies: Vec<RuntimeDependency>,
    pub resources: ResourceRequirements,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entrypoint {
    pub kind: EntrypointKind,
    pub target: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntrypointKind {
    Command,
    ModuleFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeDependency {
    pub kind: DependencyKind,
    pub name: String,
    pub version_req: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    Browser,
    Binary,
    EnvironmentVariable,
    Capability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceRequirements {
    pub timeout_secs: u64,
    pub max_concurrency: u16,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn crawler_project_round_trips_with_serde_json() {
        let project = CrawlerProject {
            manifest: ProjectManifest {
                id: ProjectId::new("news-listing"),
                version: ProjectVersion::new("2026.04.25"),
                title: "News Listing".to_string(),
                description: Some("Extract article cards from the landing page".to_string()),
                target: TargetMetadata {
                    site: TargetSite {
                        name: "Example News".to_string(),
                        base_url: "https://example.com".to_string(),
                    },
                    start_urls: vec!["https://example.com/news".to_string()],
                },
                inputs: vec![InputParameter {
                    name: "category".to_string(),
                    description: Some("Optional category slug".to_string()),
                    required: false,
                    parameter_type: ParameterType::String,
                    default_value: Some("top".to_string()),
                }],
                outputs: vec![OutputField {
                    name: "title".to_string(),
                    description: Some("Article title".to_string()),
                    field_type: ParameterType::String,
                    required: true,
                }],
                labels: BTreeMap::from([("team".to_string(), "crawler-platform".to_string())]),
            },
            runtime: RuntimeContract {
                entrypoint: Entrypoint {
                    kind: EntrypointKind::Command,
                    target: "bin/run_project".to_string(),
                    arguments: vec!["--mode".to_string(), "quick-start".to_string()],
                },
                dependencies: vec![RuntimeDependency {
                    kind: DependencyKind::Browser,
                    name: "chromium".to_string(),
                    version_req: Some(">=124".to_string()),
                    required: true,
                }],
                resources: ResourceRequirements {
                    timeout_secs: 120,
                    max_concurrency: 4,
                },
                environment: BTreeMap::from([(
                    "PLAYWRIGHT_BROWSERS_PATH".to_string(),
                    "/opt/browsers".to_string(),
                )]),
            },
        };

        let json = serde_json::to_string_pretty(&project).expect("serialize project");
        let decoded: CrawlerProject = serde_json::from_str(&json).expect("deserialize project");

        assert_eq!(decoded, project);
    }
}

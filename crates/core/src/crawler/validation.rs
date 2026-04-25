use serde::{Deserialize, Serialize};

use crate::crawler::ProjectReference;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationReport {
    pub project: ProjectReference,
    pub static_issues: Vec<StaticValidationIssue>,
    pub environment_issues: Vec<EnvironmentValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticValidationIssue {
    pub severity: ValidationSeverity,
    pub code: StaticValidationCode,
    pub field: Option<IssueField>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentValidationIssue {
    pub severity: ValidationSeverity,
    pub code: EnvironmentValidationCode,
    pub field: Option<IssueField>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IssueField {
    pub section: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Warning,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StaticValidationCode {
    MissingField,
    InvalidField,
    InvalidEntrypoint,
    InvalidSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentValidationCode {
    MissingDependency,
    MissingConfiguration,
    UnsupportedCapability,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawler::{ProjectId, ProjectVersion};

    #[test]
    fn validation_report_round_trips_with_serde_json() {
        let report = ValidationReport {
            project: ProjectReference {
                project_id: ProjectId::new("news-listing"),
                version: ProjectVersion::new("2026.04.25"),
            },
            static_issues: vec![StaticValidationIssue {
                severity: ValidationSeverity::Fatal,
                code: StaticValidationCode::MissingField,
                field: Some(IssueField {
                    section: "manifest".to_string(),
                    name: "title".to_string(),
                }),
                message: "title is required".to_string(),
            }],
            environment_issues: vec![EnvironmentValidationIssue {
                severity: ValidationSeverity::Warning,
                code: EnvironmentValidationCode::MissingDependency,
                field: Some(IssueField {
                    section: "runtime.dependencies".to_string(),
                    name: "chromium".to_string(),
                }),
                message: "chromium is not installed".to_string(),
            }],
        };

        let json = serde_json::to_string_pretty(&report).expect("serialize report");
        let decoded: ValidationReport = serde_json::from_str(&json).expect("deserialize report");

        assert_eq!(decoded, report);
    }

    #[test]
    fn project_version_is_path_independent() {
        let reference = ProjectReference {
            project_id: ProjectId::new("news-listing"),
            version: ProjectVersion::new("release-2026-04-25"),
        };

        assert_eq!(reference.project_id.0, "news-listing");
        assert_eq!(reference.version.0, "release-2026-04-25");
    }
}

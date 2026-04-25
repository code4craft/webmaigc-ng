## ADDED Requirements

### Requirement: A crawler project must be portable across local and server execution
The system SHALL define a crawler project format that can be created locally and executed unchanged by the server runtime.

#### Scenario: Reuse project artifact across modes
- **WHEN** a crawler project validated in Quick Start mode is published to the server
- **THEN** the server accepts the same project artifact without requiring manual reauthoring of crawl logic

### Requirement: A crawler project must declare execution contract
The system SHALL define project metadata for input parameters, target definitions, execution entrypoints, output schema, and dependency requirements.

#### Scenario: Validate project contract before execution
- **WHEN** a project is prepared for local run or server publish
- **THEN** the system validates that required metadata and runtime declarations are present and rejects incomplete artifacts

### Requirement: A crawler project must support versioned publishing
The system SHALL assign a version identity to each publishable crawler project so that task runs can be traced back to an exact project revision.

#### Scenario: Submit tasks against a published version
- **WHEN** a task is created for a crawler project on the server
- **THEN** the task references a specific published project version and the platform can retrieve its code and configuration deterministically

### Requirement: A crawler project must support environment checks
The system SHALL provide a validation mechanism that checks whether required runtime dependencies and configuration are available before execution.

#### Scenario: Detect missing runtime prerequisites
- **WHEN** a user runs validation on a crawler project
- **THEN** the system reports missing browser dependencies, configuration values, or unsupported runtime settings before execution begins

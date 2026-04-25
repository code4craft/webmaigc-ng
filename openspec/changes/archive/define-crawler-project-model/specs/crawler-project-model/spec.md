## ADDED Requirements

### Requirement: A crawler project must define portable project metadata
The system SHALL define crawler project metadata that can be interpreted consistently by local CLI flows and server-side publish or execution flows.

#### Scenario: Load project metadata in different runtimes
- **WHEN** the same crawler project is read by the local CLI and the server publish flow
- **THEN** both runtimes interpret the same project identity, target metadata, and declared entry information

### Requirement: A crawler project must declare execution contract
The system SHALL define a project contract that includes input parameters, output schema, execution entrypoint, and declared runtime dependencies.

#### Scenario: Validate required contract fields
- **WHEN** a crawler project is prepared for validation or publish
- **THEN** the system rejects the project if required contract fields are missing or malformed

### Requirement: A crawler project must expose a stable version identity
The system SHALL assign or accept a stable version identity for each publishable crawler project so downstream task runs can reference an exact project revision.

#### Scenario: Bind task execution to a project version
- **WHEN** a task is created from a published crawler project
- **THEN** the task references a specific project version that can be resolved without using local file paths

### Requirement: A crawler project must support preflight validation
The system SHALL support both static validation and environment validation before execution begins.

#### Scenario: Report static contract violations
- **WHEN** a project manifest is incomplete or structurally invalid
- **THEN** the system returns static validation failures without attempting runtime execution

#### Scenario: Report environment readiness failures
- **WHEN** a project requires unavailable runtime dependencies or missing configuration
- **THEN** the system returns environment validation failures before the project is executed

### Requirement: Shared crawler project types must live in the shared core boundary
The system SHALL keep the stable crawler project contract in the shared Rust core module so that CLI, server, and worker components can reuse the same definitions.

#### Scenario: Reuse project types across modules
- **WHEN** multiple runtime components need to parse or validate a crawler project
- **THEN** they depend on shared core definitions instead of redefining separate project types


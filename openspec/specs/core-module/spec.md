## Requirements

### Requirement: Core module owns the shared crawler project contract
The `crates/core` module SHALL define the stable crawler project contract used across local CLI flows, server publish flows, and worker execution flows.

#### Scenario: Reuse crawler project definitions across runtimes
- **WHEN** CLI, server, and worker components need to load or validate the same crawler project
- **THEN** they reuse shared types from `crates/core` instead of defining separate project models

### Requirement: Core module defines manifest and runtime contract separately
The `crates/core` module SHALL represent a crawler project using a project manifest layer and a runtime contract layer.

#### Scenario: Interpret project identity and execution requirements
- **WHEN** a crawler project is parsed by a runtime component
- **THEN** the manifest provides project identity and declared metadata, and the runtime contract provides execution entry and runtime requirements

### Requirement: Core module exposes stable project version identity
The `crates/core` module SHALL define a stable project version identity that does not depend on local file paths or temporary runtime state.

#### Scenario: Reference a published project revision
- **WHEN** downstream services create or inspect a task run
- **THEN** they can resolve the project to a stable version identifier defined by the shared core contract

### Requirement: Core module defines preflight validation semantics
The `crates/core` module SHALL define separate static validation and environment validation result types for crawler projects.

#### Scenario: Report invalid project structure
- **WHEN** a crawler project is missing required metadata or contains malformed contract fields
- **THEN** the system returns static validation failures from the shared core validation model

#### Scenario: Report missing runtime readiness
- **WHEN** a crawler project depends on unavailable runtime capabilities or configuration
- **THEN** the system returns environment validation failures from the shared core validation model before execution begins

### Requirement: Core module remains runtime-agnostic
The `crates/core` module SHALL not depend directly on browser automation libraries, HTTP frameworks, databases, or task queue implementations.

#### Scenario: Consume core from multiple runtime modules
- **WHEN** application modules depend on `crates/core`
- **THEN** they can use the shared contract without inheriting unrelated infrastructure dependencies


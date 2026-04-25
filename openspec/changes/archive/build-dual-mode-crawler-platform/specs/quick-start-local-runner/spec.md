## ADDED Requirements

### Requirement: Natural language prompt can initialize a crawler project
The system SHALL allow a user to describe a crawl target and desired fields in natural language and generate an initial crawler project scaffold for local execution.

#### Scenario: Generate scaffold from prompt
- **WHEN** a user provides a target website description and extraction intent
- **THEN** the system creates a crawler project with runnable entrypoints, extraction placeholders, and project metadata

### Requirement: Page analysis must produce inspectable extraction suggestions
The system SHALL use browser automation to inspect the target page and produce extraction suggestions that can be reviewed or refined before execution.

#### Scenario: Analyze DOM structure
- **WHEN** the user runs page analysis for a target page
- **THEN** the system returns candidate selectors, candidate fields, and analysis logs tied to that page

### Requirement: Local runner must execute the generated crawler from CLI
The system SHALL provide a command-line interface that can execute a crawler project locally with explicit input parameters and runtime logs.

#### Scenario: Run crawler from CLI
- **WHEN** a user executes the generated crawler with the local CLI
- **THEN** the system starts the crawl, streams runtime logs, and outputs extracted records or structured errors

### Requirement: Local execution logs must support iterative repair
The system SHALL emit structured logs for analysis, code generation, execution, and extraction failures so that a Coding Agent or developer can iterate on fixes.

#### Scenario: Capture actionable failure logs
- **WHEN** local execution fails because selectors, navigation, or extraction logic are invalid
- **THEN** the system records the failed step, error payload, and related project context for retry or modification

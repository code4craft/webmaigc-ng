## ADDED Requirements

### Requirement: Administrators must have a global task operations view
The system SHALL provide an administrative console that shows crawler projects, task instances, statuses, failure counts, and active schedules across the platform.

#### Scenario: Review fleet health
- **WHEN** an administrator opens the operations dashboard
- **THEN** the system displays aggregated platform task status and highlights failed or delayed workloads

### Requirement: Administrators must inspect failure details
The system SHALL allow administrators to open a task instance and inspect structured logs, error summaries, and execution metadata needed for triage.

#### Scenario: Investigate a failed task
- **WHEN** an administrator selects a failed task instance
- **THEN** the system shows task metadata, execution timeline, and diagnostic logs for that task

### Requirement: Submitters must only see their own operational data
The system SHALL provide a scoped operational view for non-admin users that includes only projects, tasks, schedules, and logs they own.

#### Scenario: Open scoped task list
- **WHEN** a non-admin submitter accesses the operations interface
- **THEN** the system displays only their own projects and task instances

### Requirement: Admin interfaces must support web-based access
The system SHALL provide a web-based admin entrypoint for platform operations and diagnostics.

#### Scenario: Access admin console from browser
- **WHEN** an authorized administrator accesses the admin interface from a browser
- **THEN** the system allows them to browse task state, logs, and operational summaries without using the CLI

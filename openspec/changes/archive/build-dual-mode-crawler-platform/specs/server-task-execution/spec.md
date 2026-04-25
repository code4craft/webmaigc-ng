## ADDED Requirements

### Requirement: Server must accept authenticated crawler task submissions
The system SHALL expose authenticated interfaces for publishing crawler projects and creating execution tasks using API Key based access.

#### Scenario: Submit task with valid credentials
- **WHEN** a client submits a crawler project version and task request with a valid API Key
- **THEN** the server creates the task and associates it with the authenticated submitter

### Requirement: Server must support scheduled and continuous execution
The system SHALL support both ad hoc task execution and Cron-based recurring task schedules for published crawler projects.

#### Scenario: Trigger recurring crawl
- **WHEN** a user creates a Cron schedule for a published crawler project
- **THEN** the scheduler enqueues task instances according to the configured schedule

### Requirement: Worker execution must be isolated from control plane services
The system SHALL execute crawler tasks in isolated worker runtimes so that crawl failures do not terminate API or scheduling services.

#### Scenario: Worker task crashes during execution
- **WHEN** a crawler task exits unexpectedly or exceeds its runtime budget
- **THEN** the worker marks the task failed, releases execution resources, and leaves control plane services available

### Requirement: Task lifecycle must be queryable
The system SHALL persist task lifecycle state transitions including queued, running, succeeded, failed, and cancelled states.

#### Scenario: Inspect task status
- **WHEN** a user queries a previously submitted task
- **THEN** the system returns the latest task state, timestamps, project version, and terminal outcome if available

### Requirement: Task logs must be isolated by task and submitter
The system SHALL store execution logs in a way that supports per-task retrieval and enforces submitter-scoped access.

#### Scenario: Retrieve own task logs
- **WHEN** a submitter requests logs for a task they own
- **THEN** the system returns logs for that task and denies access to logs belonging to unrelated submitters

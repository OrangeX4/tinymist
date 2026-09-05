## ADDED Requirements

### Requirement: Memory exports work without system capabilities
Tinymist SHALL support memory exports with the web and export features enabled.

#### Scenario: Query compiled metadata
- **WHEN** a Web client calls exportQuery with write=false on published source
- **THEN** the response contains the actual compiled query result as base64 data
- **AND** no filesystem write or native thread is required

#### Scenario: Export physical pages
- **WHEN** a Web client requests PDF or SVG output with write=false
- **THEN** the response uses the same page ordering and export options as native hosts

#### Scenario: Reject disk writes on a Web host
- **WHEN** a Web client requests write=true
- **THEN** the command reports that filesystem export is unavailable
- **AND** the published workspace remains unchanged

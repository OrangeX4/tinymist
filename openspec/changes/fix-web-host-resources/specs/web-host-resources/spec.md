## ADDED Requirements

### Requirement: Virtual file URI round trips
The Web language service SHALL preserve Unicode, spaces, percent signs, fragment markers,
and query markers in virtual file paths when converting to and from file URIs.

#### Scenario: Unicode import with reserved characters
- **WHEN** an open document imports a file whose name contains Unicode and reserved URI characters
- **THEN** language queries and compilation SHALL resolve the same host-provided file bytes.

### Requirement: Host resource publication
An initialized Web language service SHALL accept host font resolvers and filesystem updates
without discarding open source documents.

#### Scenario: A font finishes downloading
- **WHEN** the host publishes a resolver containing the downloaded font
- **THEN** the project SHALL invalidate font-dependent compilation and preserve its current source.

### Requirement: Memory workspace inventory
The analysis workspace inventory SHALL include editor-owned memory files alongside filesystem files.

#### Scenario: Rename an imported binding in an unsaved workspace
- **WHEN** an open memory file imports a binding from another open memory file
- **THEN** rename SHALL include references in the importing file and the binding declaration.

### Requirement: VFS snapshot publication
VFS mutation SHALL advance the snapshot revision when byte state or file inventory changes.
Batch invalidation SHALL accumulate, and identical repeated bytes SHALL retain the current revision.

#### Scenario: A new memory document is opened
- **WHEN** the host publishes a source file that no previous snapshot has read
- **THEN** the next language query SHALL observe the newly published source.

#### Scenario: A package batch ends with unchanged files
- **WHEN** a filesystem batch updates a dependency and also repeats existing unchanged files
- **THEN** the next snapshot SHALL include the updated dependency regardless of entry order.

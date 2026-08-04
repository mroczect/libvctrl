# libvcrtl

A robust, content-addressed version control engine for arbitrary data, designed for embedding into applications. It provides a cryptographically secure foundation for building versioning, branching, and merging of structured data without relying on a filesystem.

## Status

This project is under active, full-time development and has not yet reached a stable release. The API may change significantly between versions. Use at your own risk.

## Design Principles

- **Security first**: All objects are hashed with SHA-512. Commit and tree integrity is guaranteed by content-addressed storage.
- **No defaults**: The engine exposes traits for storage and reference management. Users choose or implement their own backends (in-memory, file-based, remote, etc.).
- **Data agnostic**: Unlike Git, the engine does not operate on files. Users define what a blob or tree represents. This makes it suitable for databases, game states, configuration systems, or collaborative editing.
- **Explicit error handling**: Every fallible operation returns a `Result`. No panics or unwraps in library code.

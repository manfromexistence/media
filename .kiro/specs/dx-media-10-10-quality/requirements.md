
# Requirements Document

## Introduction

This specification defines the requirements for elevating dx-media from a 7/10 production-ready codebase to a true 10/10 professional-grade Rust library. The focus is on eliminating technical debt, removing blanket lint suppressions, cleaning up dead code, and establishing patterns that exemplify Rust best practices.

## Glossary

- Blanket_Suppression: A crate-level `#![allow(...)]` attribute that suppresses warnings across the entire codebase
- Dead_Code: Code that is never executed or fields that are never read
- Item_Level_Suppression: A `#[allow(...)]` attribute applied to a specific item (function, struct, field)
- Clippy: The Rust linter that enforces code quality and best practices
- Serde_Deserialization_Field: A struct field that exists solely for JSON/YAML deserialization via serde

## Requirements

### Requirement 1: Eliminate Blanket Clippy Suppressions

User Story: As a library maintainer, I want all clippy warnings fixed at the source, so that the codebase demonstrates Rust best practices.

#### Acceptance Criteria

- THE Codebase SHALL have no more than 10 crate-level `#![allow(clippy::...)]` suppressions in lib.rs
- WHEN a clippy warning occurs, THE Developer SHALL fix the underlying code rather than suppress it
- WHERE a suppression is truly unavoidable, THE Developer SHALL use item-level `#[allow(...)]` with a comment explaining why
- THE lib.rs file SHALL contain only these justified crate-level suppressions:-`module_name_repetitions` (semantic naming choice)
- `similar_names` (semantic naming choice)
- `doc_markdown` (technical terms)
- `missing_errors_doc` (errors documented via DxError)
- `missing_panics_doc` (functions designed not to panic)
- `unused_async` (trait conformance)

### Requirement 2: Remove All Dead Code

User Story: As a library maintainer, I want no dead code in the codebase, so that the library is lean and maintainable.

#### Acceptance Criteria

- THE Codebase SHALL have zero `#[allow(dead_code)]` attributes on production code
- WHEN a struct field is only used for serde deserialization, THE Field SHALL be annotated with `#[serde(skip_serializing)]` or the struct SHALL derive only `Deserialize`
- WHEN code is intended for future use, THE Code SHALL be removed and tracked in an issue instead
- THE Codebase SHALL compile with `#![deny(dead_code)]` enabled (excluding test code)

### Requirement 3: Fix Numeric Cast Issues

User Story: As a library user, I want numeric operations to be safe, so that I don't encounter unexpected truncation or overflow.

#### Acceptance Criteria

- WHEN casting between numeric types, THE Code SHALL use `TryFrom`/`TryInto` for fallible conversions
- WHEN truncation is intentional and safe, THE Code SHALL use item-level `#[allow(clippy::cast_possible_truncation)]` with a safety comment
- THE Codebase SHALL not have blanket suppressions for `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss`, or `cast_lossless`

### Requirement 4: Fix String and Collection Patterns

User Story: As a library maintainer, I want idiomatic Rust patterns, so that the code is efficient and readable.

#### Acceptance Criteria

- WHEN building strings, THE Code SHALL use `write!` macro or `push_str` with string literals instead of `format!` in loops
- WHEN iterating with index, THE Code SHALL use `enumerate()` instead of range loops where appropriate
- WHEN mapping and unwrapping, THE Code SHALL use `map_or` or `map_or_else` instead of `map().unwrap_or()`
- THE Codebase SHALL not have blanket suppressions for string/collection patterns

### Requirement 5: Fix Function Signature Issues

User Story: As a library user, I want consistent and ergonomic APIs, so that the library is easy to use correctly.

#### Acceptance Criteria

- WHEN a function returns a value that should be used, THE Function SHALL be annotated with `#[must_use]`
- WHEN a builder method returns Self, THE Method SHALL be annotated with `#[must_use]`
- WHEN a function takes ownership unnecessarily, THE Function SHALL take a reference instead
- WHEN a function has more than 7 parameters, THE Function SHALL use a builder pattern or options struct

### Requirement 6: Fix Control Flow Patterns

User Story: As a library maintainer, I want clear control flow, so that the code is easy to understand and maintain.

#### Acceptance Criteria

- WHEN using if-else with negation, THE Code SHALL restructure to put the positive case first
- WHEN matching on a single variant, THE Code SHALL use `if let` instead of `match`
- WHEN match arms have identical bodies, THE Code SHALL combine them with `|` pattern
- THE Codebase SHALL not have blanket suppressions for control flow patterns

### Requirement 7: Establish Item-Level Suppression Standards

User Story: As a library maintainer, I want consistent suppression practices, so that exceptions are documented and justified.

#### Acceptance Criteria

- WHEN an item-level suppression is needed, THE Suppression SHALL include a comment explaining why
- THE Comment format SHALL be: `// SAFETY: <reason>` for unsafe code or `// LINT: <reason>` for clippy
- WHEN multiple items need the same suppression, THE Code SHALL be refactored to avoid the pattern if possible
- THE Codebase SHALL have no more than 50 item-level clippy suppressions total

### Requirement 8: Provider Response Struct Cleanup

User Story: As a library maintainer, I want clean provider implementations, so that API response handling is consistent.

#### Acceptance Criteria

- WHEN a provider response struct has unused fields, THE Struct SHALL derive only `Deserialize` (not `Serialize`)
- WHEN a field is needed for deserialization but not used in code, THE Field SHALL use `#[serde(skip_serializing)]` or be removed
- THE Provider response structs SHALL not use `#[allow(dead_code)]` attributes
- WHEN a provider is disabled, THE Provider code SHALL be feature-gated or removed entirely

### Requirement 9: Test Code Quality

User Story: As a library maintainer, I want high-quality test code, so that tests are reliable and maintainable.

#### Acceptance Criteria

- THE Test code SHALL be allowed to have `#[allow(dead_code)]` for test fixtures only
- WHEN test helpers are unused, THE Helpers SHALL be removed
- THE Test code SHALL pass clippy with the same configuration as production code
- THE Property tests SHALL cover all critical correctness properties

### Requirement 10: Deprecated API Cleanup

User Story: As a library user, I want clear migration paths, so that I can update my code when APIs change.

#### Acceptance Criteria

- WHEN an API is deprecated, THE Deprecation message SHALL include the replacement API
- IF a deprecated API has been deprecated for more than one major version, THE API SHALL be removed
- THE `try_build()` method SHALL be removed and replaced with `build()` returning `Result`
- THE Codebase SHALL have no deprecated APIs without clear migration documentation

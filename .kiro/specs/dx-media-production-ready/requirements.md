
# Requirements Document

## Introduction

This specification defines the requirements for making the `dx-media` crate production-ready. The crate currently has a solid foundation but contains technical debt that must be addressed before it can be considered a professional, production-grade codebase. Key issues include blanket clippy suppressions hiding real problems, unsafe lock handling that can panic, undocumented magic numbers, a fake browser User-Agent, missing integration tests, and silent error swallowing in builder patterns.

## Glossary

- DX_Media: The main media acquisition engine library and CLI tool
- Circuit_Breaker: Component that prevents cascading failures by tracking provider failures
- Provider: External API service that supplies media assets (NASA, Openverse, Pexels, etc.)
- Rate_Limiter: Component that throttles requests to respect API limits
- HTTP_Client: Shared HTTP client with retry logic and rate limiting
- Media_Asset_Builder: Builder pattern for constructing MediaAsset instances
- Clippy: Rust's official linter for catching common mistakes and enforcing idioms

## Requirements

### Requirement 1: Remove Blanket Clippy Suppressions

User Story: As a maintainer, I want all clippy warnings to be addressed at the source rather than suppressed globally, so that real issues are not hidden and code quality is enforced.

#### Acceptance Criteria

- THE DX_Media lib.rs SHALL contain no more than 10 clippy allow attributes at the crate level
- WHEN a clippy warning is suppressed, THE suppression SHALL be placed at the item level with a justification comment
- THE DX_Media crate SHALL pass `cargo clippy
- -
- D warnings` with only justified, item-level suppressions
- WHEN clippy identifies a legitimate issue, THE code SHALL be refactored to fix the issue rather than suppressed

### Requirement 2: Safe Lock Handling in Circuit Breaker

User Story: As a developer, I want the circuit breaker to handle lock poisoning gracefully, so that a panic in one thread does not crash the entire application.

#### Acceptance Criteria

- WHEN a RwLock is poisoned, THE Circuit_Breaker SHALL recover gracefully instead of panicking
- THE Circuit_Breaker SHALL NOT use `.unwrap()` on RwLock read or write operations
- WHEN lock acquisition fails, THE Circuit_Breaker SHALL log the error and return a safe default state
- IF a lock is poisoned, THEN THE Circuit_Breaker SHALL reset to a closed state and continue operation

### Requirement 3: Document and Configure Magic Numbers

User Story: As a developer, I want all magic numbers to be documented constants or configurable values, so that I understand their purpose and can tune them for different environments.

#### Acceptance Criteria

- THE SearchMode early-exit multiplier (currently hardcoded as 3x) SHALL be a documented constant with explanation
- THE Circuit_Breaker default failure threshold (3) and timeout (60s) SHALL be documented constants
- THE Rate_Limiter default configuration (100 requests/60 seconds) SHALL be a documented constant
- THE HTTP_Client exponential backoff base delay (1000ms) SHALL be a documented constant
- WHEN a magic number affects behavior, THE constant SHALL include a doc comment explaining its purpose and recommended values

### Requirement 4: Honest User-Agent String

User Story: As an API consumer, I want to identify myself honestly to API providers, so that I comply with API terms of service and maintain good relationships with providers.

#### Acceptance Criteria

- THE DX_Media USER_AGENT SHALL identify the library name and version (e.g., "dx-media/0.1.0")
- THE USER_AGENT SHALL NOT impersonate a web browser
- THE USER_AGENT SHALL include a contact URL or repository link for API providers to reach out
- WHERE a provider requires a browser-like User-Agent, THE Provider SHALL override the default with a documented justification

### Requirement 5: Integration Tests with Mocked HTTP

User Story: As a developer, I want integration tests that verify provider parsing without hitting real APIs, so that tests are fast, reliable, and don't depend on external services.

#### Acceptance Criteria

- THE DX_Media crate SHALL have integration tests using wiremock for HTTP mocking
- WHEN testing a Provider, THE test SHALL mock the API response with realistic sample data
- THE integration tests SHALL verify correct parsing of provider responses into MediaAsset structures
- THE integration tests SHALL verify error handling for malformed responses
- THE integration tests SHALL verify rate limiting behavior with mocked responses
- THE integration tests SHALL achieve at least 80% code coverage on provider parsing logic

### Requirement 6: Explicit Error Handling in Builders

User Story: As a developer, I want builder validation errors to be explicit and traceable, so that I can debug issues when asset construction fails.

#### Acceptance Criteria

- THE Media_Asset_Builder `try_build()` method SHALL be deprecated with a warning message
- WHEN `try_build()` is called, THE builder SHALL log a warning with the missing field names
- THE Media_Asset_Builder SHALL provide a `build_or_log()` method that logs errors before returning None
- WHEN a required field is missing, THE error message SHALL specify which field is missing

### Requirement 7: Remove Dead Code Markers

User Story: As a maintainer, I want dead code to be either removed or properly feature-gated, so that the codebase is clean and intentional.

#### Acceptance Criteria

- THE DX_Media crate SHALL NOT contain `#[allow(dead_code)]` with "potential future use" comments
- WHEN code is intended for future use, THE code SHALL be feature-gated with a descriptive feature flag
- WHEN code is truly unused, THE code SHALL be removed from the codebase
- THE `timeout` field in HTTP_Client SHALL either be used or removed

### Requirement 8: Production Version and Changelog

User Story: As a user, I want accurate version information and changelog, so that I can track changes and assess stability.

#### Acceptance Criteria

- WHEN the production-ready milestone is complete, THE version SHALL be updated to 1.0.0
- THE CHANGELOG.md SHALL accurately reflect all breaking changes, new features, and bug fixes
- THE CHANGELOG.md SHALL follow Keep a Changelog format
- THE README.md SHALL remove any "work in progress" or "alpha" disclaimers after 1.0.0 release

### Requirement 9: Deployment Documentation

User Story: As a DevOps engineer, I want clear documentation on external dependencies and deployment requirements, so that I can successfully deploy the application.

#### Acceptance Criteria

- THE README.md SHALL document all required external tools (FFmpeg, ImageMagick, etc.)
- THE documentation SHALL include a Docker example or Dockerfile for containerized deployment
- THE documentation SHALL specify minimum versions for external dependencies
- WHEN an external tool is optional, THE documentation SHALL indicate which features require it
- THE documentation SHALL include troubleshooting steps for common deployment issues

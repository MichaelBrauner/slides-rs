# QS-010: Isolated Testable Functions

**Quality Attribute:** [Maintainability - Testability](../10-quality-requirements.md#106-maintainability)

## Stimulus
Developer wants to verify change doesn't break existing functionality

## Environment
Development and CI/CD pipeline

## Artifact
Test suite, module boundaries

## Response
1. Core functions are pure and side-effect free where possible
2. I/O operations isolated at module boundaries
3. Unit tests cover business logic
4. Integration tests verify end-to-end workflows

## Response Measure
Test coverage over 70% for core modules; unit tests run under 5 seconds; all tests pass in CI before merge

## Related
- [ADR-001: Rust as Implementation Language](../adr/ADR-001-rust-as-implementation-language.md)

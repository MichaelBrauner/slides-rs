# QS-005: Single Binary Installation

**Quality Attribute:** [Operability - Installability](../10-quality-requirements.md#103-operability)

## Stimulus
User wants to install slides-rs

## Environment
Clean system without development tools

## Artifact
Binary distribution

## Response
1. User downloads single binary file
2. Binary is self-contained with all dependencies
3. No runtime dependencies (no Node.js, Python, etc.)
4. Runs immediately after download

## Response Measure
Installation requires downloading exactly 1 file; works on systems without development tools installed

## Related
- [ADR-001: Rust as Implementation Language](../adr/ADR-001-rust-as-implementation-language.md)

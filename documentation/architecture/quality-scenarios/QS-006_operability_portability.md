# QS-006: Cross-Platform Support

**Quality Attribute:** [Operability - Portability](../10-quality-requirements.md#103-operability)

## Stimulus
User wants to use slides-rs on different operating systems or share project with team

## Environment
Development across Linux, macOS, and Windows

## Artifact
Build system, binary targets, project structure

## Response
1. Pre-built binaries for Linux, macOS, Windows
2. Same project files work across all platforms
3. Output HTML is platform-independent
4. Path handling works correctly on all platforms

## Response Measure
Project created on one platform builds identically on others; generated HTML works in all modern browsers

## Related
- [ADR-001: Rust as Implementation Language](../adr/ADR-001-rust-as-implementation-language.md)
- [ADR-003: Static HTML Output](../adr/ADR-003-static-html-output.md)

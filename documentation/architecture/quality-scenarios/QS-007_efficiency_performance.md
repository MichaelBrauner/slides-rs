# QS-007: Fast Builds

**Quality Attribute:** [Efficiency - Build Performance](../10-quality-requirements.md#104-efficiency)

## Stimulus
User runs `slides build` or `slides watch` during development

## Environment
Normal development workflow, typical presentation with 10-50 slides

## Artifact
Build pipeline, template engine

## Response
1. Templates parsed and rendered efficiently
2. Assets copied without unnecessary processing
3. Only changed files trigger rebuilds in watch mode
4. Parallel processing where beneficial

## Response Measure
Build time for 20-slide presentation under 500ms (without thumbnails), under 5s (with thumbnails); watch mode rebuild under 200ms

## Related
- [ADR-001: Rust as Implementation Language](../adr/ADR-001-rust-as-implementation-language.md)
- [ADR-004: Headless Chrome for PDF/Thumbnails](../adr/ADR-004-headless-chrome-for-thumbnails.md)

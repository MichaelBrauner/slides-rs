# QS-001: Extensible Templates

**Quality Attribute:** [Flexibility - Extensibility](../10-quality-requirements.md#101-flexibility)

## Stimulus
Need to add custom layouts or template features for specific presentation needs

## Environment
User working on their presentation project

## Artifact
Template engine, layout system

## Response
1. New layouts created by adding `.html.twig` files to `templates/layouts/`
2. Layouts immediately available via `{% extends %}`
3. All Jinja2 features work (extends, blocks, includes, macros)
4. No modification of slides-rs source code required

## Response Measure
New layouts usable immediately after creation without rebuilding slides-rs or modifying configuration

## Related
- [ADR-002: MiniJinja as Template Engine](../adr/ADR-002-minijinja-as-template-engine.md)

# QS-008: Git and AI Friendly

**Quality Attribute:** [Suitable - Interoperability](../10-quality-requirements.md#105-suitable)

## Stimulus
User wants to version control presentation or use AI tools to create content

## Environment
Development workflow with Git, code review, AI assistants

## Artifact
Project file formats, template syntax

## Response
1. All project files are plain text (HTML, YAML, CSS, JS)
2. Templates use well-known Jinja2 syntax
3. Changes produce meaningful diffs
4. AI assistants can read, understand, and generate valid templates

## Response Measure
100% of project files are plain text; AI tools (ChatGPT, Claude, Copilot) can generate valid templates without special training

## Related
- [ADR-002: MiniJinja as Template Engine](../adr/ADR-002-minijinja-as-template-engine.md)
- [ADR-003: Static HTML Output](../adr/ADR-003-static-html-output.md)

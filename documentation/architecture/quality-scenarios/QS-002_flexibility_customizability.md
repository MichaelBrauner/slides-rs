# QS-002: Customizable Styles

**Quality Attribute:** [Flexibility - Customizability](../10-quality-requirements.md#101-flexibility)

## Stimulus
Need to change visual appearance (colors, fonts, animations) or add custom JavaScript

## Environment
User working on their presentation project

## Artifact
Asset pipeline, CSS/JS files

## Response
1. User modifies files in `slides/assets/`
2. Any CSS/JS features can be used
3. External libraries can be included
4. Changes reflected in next build

## Response Measure
100% of CSS and JavaScript features available; any external library includable without restrictions

## Related
- [ADR-003: Static HTML Output](../adr/ADR-003-static-html-output.md)

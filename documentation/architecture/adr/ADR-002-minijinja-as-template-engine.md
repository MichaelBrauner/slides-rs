# 002. MiniJinja as Template Engine

In the context of **providing a template syntax for slide authors**,
facing **the need for a powerful yet familiar syntax**,
we decided for **MiniJinja (Jinja2 compatible)**
to achieve **low learning curve and powerful features**,
accepting **Rust-specific limitations vs. full Jinja2**.

## Context

Users need to write templates for their slides. The template engine should:

- Be familiar to web developers
- Support inheritance (extends/blocks)
- Allow custom filters and functions
- Have good error messages
- Be fast

Alternatives considered:

| Option | Pros | Cons |
|--------|------|------|
| **MiniJinja** | Jinja2 syntax, well-maintained, good errors | Not 100% Jinja2 compatible |
| **Tera** | Jinja2-like, Rust-native | Different syntax details, less familiar |
| **Handlebars** | Logic-less, simple | Too limited for complex layouts |
| **Askama** | Compile-time checked | Templates compiled into binary, not user-editable |

## Decision

We chose MiniJinja because:

1. **Jinja2 Familiarity**: Most web developers know Jinja2/Twig/Django templates.
2. **IDE Support**: `.twig` extension gives autocompletion in IDEs.
3. **Power**: Inheritance, blocks, includes, macros all work.
4. **Error Messages**: Clear, helpful errors with line numbers.
5. **Actively Maintained**: Regular updates and good documentation.

## Consequences

### Positive

- Users familiar with Jinja2/Twig/Django can start immediately
- Complex layouts are possible (extends, blocks, includes)
- Good IDE support with syntax highlighting
- AI tools understand Jinja2 syntax well

### Negative

- Not 100% Jinja2 compatible (some edge cases differ)
- Users unfamiliar with template engines need to learn
- Runtime template errors (vs. compile-time with Askama)

## Related
- [QS-001: Extensible Templates](../quality-scenarios/QS-001_flexibility_extensibility.md)
- [QS-003: Easy Onboarding](../quality-scenarios/QS-003_usability_learnability.md)
- [QS-008: Git and AI Friendly](../quality-scenarios/QS-008_suitable_interoperability.md)

Date: 2025-11-07

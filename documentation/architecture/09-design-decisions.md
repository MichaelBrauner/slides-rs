# 09. Design Decisions

Architecture Decision Records (ADRs) inspired by Michael Nygard's format (2011), adapted for our needs.

## ADRs

| #                                                                  | Title                              | Date       |
|--------------------------------------------------------------------|------------------------------------|------------|
| [ADR-001](./adr/ADR-001-rust-as-implementation-language.md)        | Rust as Implementation Language    | 2025-11-07 |
| [ADR-002](./adr/ADR-002-minijinja-as-template-engine.md)           | MiniJinja as Template Engine       | 2025-11-07 |
| [ADR-003](./adr/ADR-003-static-html-output.md)                     | Static HTML Output (No Server)     | 2025-11-07 |
| [ADR-004](./adr/ADR-004-headless-chrome-for-thumbnails.md)         | Headless Chrome for PDF/Thumbnails | 2025-11-07 |

## Format

Our ADR format is **inspired by Michael Nygard** but simplified:
- We **omit "Status"** section (all ADRs are implicitly accepted once written)
- We keep: Context, Decision, Consequences, Date
- We use numbered format: `ADR-XXX-title.md`

**Template**
```markdown
# ADR-XXX: Title

In the context of **<use case>**,
facing **<concern>**,
we decided for **<option>**
to achieve **<quality>**,
accepting **<downside>**.

## Context
Problem description, alternatives considered (2-4 sentences)

## Decision
What we decided and why (1-2 sentences)

## Consequences

### Positive
- Benefit 1
- Benefit 2

### Negative
- Trade-off 1
- Trade-off 2

Date: YYYY-MM-DD
```

**Formatting:**
- Break the one-liner into multiple lines for readability
- Each clause (context/facing/decided/achieve/accepting) on separate line

---

*This document follows the [ARC42 architecture documentation template](https://docs.arc42.org/)*

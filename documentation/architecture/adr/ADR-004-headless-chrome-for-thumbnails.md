# 004. Headless Chrome for PDF/Thumbnails

In the context of **generating PDF exports and slide thumbnails**,
facing **the need for accurate rendering that matches browser display**,
we decided for **headless Chrome**
to achieve **pixel-perfect consistency with zero rendering differences**,
accepting **dependency on Chrome installation**.

## Context

slides-rs needs to:

- Generate PDF exports of presentations
- Create thumbnail images for overview and presenter views

The rendering must match what users see in their browser exactly.

Alternatives considered:

| Option | Pros | Cons |
|--------|------|------|
| **Headless Chrome** | Perfect browser fidelity | Requires Chrome installed |
| **wkhtmltopdf** | Standalone binary | Outdated WebKit, rendering differences |
| **WeasyPrint** | Python-based, good CSS support | Different rendering engine, CSS subset |
| **Puppeteer** | Same as Chrome, Node.js API | Requires Node.js runtime |

## Decision

We chose headless Chrome (via `headless_chrome` crate) because:

1. **Rendering Fidelity**: Chrome renders exactly like Chrome - no surprises.
2. **No Debugging**: "It looks different in PDF" is eliminated.
3. **CSS Support**: Full modern CSS support (grid, flexbox, custom properties).
4. **Simplicity**: One rendering engine to understand, not two.
5. **Maintained**: Chrome is actively developed and improved.

## Consequences

### Positive

- PDF and thumbnails look exactly like the browser
- No CSS compatibility issues between preview and export
- Modern CSS features work in exports
- Users can debug in Chrome DevTools

### Negative

- Chrome must be installed on the system
- Larger resource usage during PDF generation
- Slower than purpose-built PDF generators
- Chrome updates could theoretically cause issues

## Related
- [QS-007: Fast Builds](../quality-scenarios/QS-007_efficiency_performance.md)

Date: 2025-11-07

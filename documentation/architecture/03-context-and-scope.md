# 03. Context and Scope

## 3.1 Business Context

Slides RS operates as a command-line tool that transforms template-based source files into a deployable HTML presentation website.

**System Boundaries:**

```
┌─────────────────────────────────────────────────────────────────┐
│                        Developer                                 │
│                            │                                     │
│                            ▼                                     │
│  ┌──────────┐    ┌─────────────────┐    ┌──────────────────┐   │
│  │ Templates│───▶│    slides-rs    │───▶│ Static HTML/CSS  │   │
│  │ (Twig)   │    │      CLI        │    │ + Thumbnails     │   │
│  ├──────────┤    └────────┬────────┘    └──────────────────┘   │
│  │ Assets   │             │                      │              │
│  ├──────────┤             │                      ▼              │
│  │decks.yaml│             │             ┌──────────────────┐   │
│  ├──────────┤             │             │    Browser /     │   │
│  │ i18n     │             │             │    Webserver     │   │
│  └──────────┘             │             └──────────────────┘   │
│                           │                                     │
│                           ▼                                     │
│                  ┌─────────────────┐                           │
│                  │ Headless Chrome │                           │
│                  │  (Thumbnails)   │                           │
│                  └─────────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

### External Entities

| Entity | Interface | Description |
|--------|-----------|-------------|
| **Developer** | File system, CLI commands | Creates and maintains presentation source files |
| **Templates** | File system (read) | HTML/Twig files containing slide content |
| **Assets** | File system (read) | Static files (CSS, JS, images) |
| **decks.yaml** | File system (read) | YAML configuration defining presentation structure |
| **i18n** | File system (read) | Translation YAML files for multi-language support |
| **Headless Chrome** | Process execution | Browser instance for thumbnail generation and PDF export |
| **Browser/Webserver** | HTTP/File protocol | Displays the generated presentation |

## 3.2 Technical Context

### Input Interfaces

| Channel | Format | Purpose |
|---------|--------|---------|
| CLI Arguments | Command line | User commands (build, watch, export, etc.) |
| File System | Twig/HTML files | Template source files |
| File System | YAML | Configuration (decks.yaml) and translations |
| File System | Static files | CSS, JavaScript, images, fonts |

### Output Interfaces

| Channel | Format | Purpose |
|---------|--------|---------|
| File System | HTML | Generated presentation pages |
| File System | PNG | Slide thumbnails for overview page |
| File System | PDF | Exported presentation (via Chrome) |
| stdout/stderr | Text | User feedback, progress, errors |

### External Dependencies

| Dependency | Purpose | Integration |
|------------|---------|-------------|
| Chrome/Chromium/Edge | Thumbnail generation, PDF export | Headless browser automation via DevTools protocol |
| File System | Source input, build output | Direct read/write operations |

---

*This document follows the [ARC42 architecture documentation template](https://docs.arc42.org/)*

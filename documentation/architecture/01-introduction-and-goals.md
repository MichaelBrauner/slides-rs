# 01. Introduction and Goals

## 1.1 Requirements Overview

Slides-rs is a static site generator for HTML presentations. It enables developers and technical teams to create presentations using the same tools and workflows they use for code.

### Key Features

| Feature | Description |
|---------|-------------|
| **Template-based** | Jinja2/Twig-compatible templates (MiniJinja) for flexible slide creation |
| **Deck Configuration** | YAML-based presentation definition with sections |
| **Multi-language** | Translations via YAML files with `trans` filter |
| **Asset Pipeline** | Automatic copying of CSS, JS, images |
| **Thumbnails** | Automatic PNG generation via Headless Chrome |
| **Watch Mode** | Live-reload on file changes |
| **Views** | Overview, presenter view with speaker notes |

### Non-Goals

Slides-rs is **not**:
- A WYSIWYG editor (no GUI)
- A PowerPoint/Keynote replacement for non-technical users
- A Markdown-to-slides converter (templates are HTML/Twig)

## 1.2 Quality Goals

| Priority | Quality Goal | Description |
|----------|--------------|-------------|
| 1 | **Developer Experience** | Create presentations with familiar tools: Git, IDE, CLI |
| 2 | **Flexibility** | Full control over HTML/CSS/JS without restrictions |
| 3 | **Version Control** | All artifacts are text-based and git-friendly |
| 4 | **Performance** | Fast build process, even with many slides |
| 5 | **Extensibility** | Custom templates, layouts, Stimulus controllers |

## 1.3 Stakeholders

| Role | Expectations |
|------|--------------|
| **Developers** | Treat presentations like code: version, review, automate |
| **Trainers/Educators** | Reusable slide components, easy multi-language support |
| **Agencies** | Branded templates, consistent corporate design across projects |
| **DevOps** | CI/CD integration, automated builds, PDF export |

---

*This document follows the [ARC42 architecture documentation template](https://docs.arc42.org/)*

# Slides RS 

Static site generator for HTML presentations.

## Features

- [Write slides](#writing-slides) in HTML with Jinja2 templating (extends, blocks, includes, macro)
- Organize [decks](#decks) via `decks.yaml`
- [Watch mode](#commands) with auto-rebuild
- [Export to PDF](#commands)
- [Import from PowerPoint](#commands) (.pptx)
- [Multi-language support](#translations)

## Why Slides RS?

|  | PowerPoint | Slides RS |
|---|---|---|
| **Format** | Binary (.pptx) | Text (HTML/YAML) |
| **Versioning** | "final_v3_NEW.pptx" | Git (diffs, blame, PRs) |
| **Collaboration** | OneDrive/SharePoint | Git (GitHub, GitLab) |
| **Multi-language** | Manual copy | One source, N languages |
| **Reusability** | Copy & paste | Macros, includes, extends |
| **Automation** | Difficult | CI/CD, scripting |
| **AI integration** | Limited (Copilot) | Full (text-based) |
| **Learning curve** | Low | Higher (HTML, Git) |
| **WYSIWYG** | Yes | No (browser preview) |
| **Offline** | Yes | Yes |
| **Cost** | Microsoft 365 | Free (MIT) |

## Installation

See [Installation Guide](documentation/installation.md) for Windows, Linux, and macOS.

## Requirements

- **Chrome, Chromium, or Microsoft Edge** (for overview page and PDF export)

## Quick Start

```bash
# Create a new project
slides new my-presentation
cd my-presentation

# Build the presentation
slides build

# Watch for changes (auto-rebuild)
slides watch

# Export to PDF
slides export
```

## Writing Slides

### Templates

Slides live in `slides/templates/` as HTML files with [MiniJinja](https://docs.rs/minijinja/latest/minijinja/) syntax.

```html
{% extends "_base.html" %}

{% block content %}
  <h1>Hello World</h1>
  <p>{{ intro | markdown }}</p>
  <pre><code>{{ source("code/example.js") }}</code></pre>
{% endblock %}
```

*Supported extensions: `.html`, `.twig`, `.jinja2`, `.html.twig`, `.html.jinja`*

### Assets

Static files in `slides/assets/` (CSS, JS, images) are copied to the output directory.  
Reference them with the `asset` function:

```html
<img src="{{ asset('images/logo.png') }}">
```

### Translations

YAML files in `slides/translations/` provide translation strings for the `trans` filter.

```html
<h1>{{ "welcome.title" | trans }}</h1>
```

```yaml
# en.yaml
welcome:
  title: "Hello World"
```

```bash
slides build --lang en
```

### Decks

`decks.yaml` defines which templates to include and in what order.

```yaml
my-talk:
  intro:
    - welcome.html
  main:
    - topic-a.html

short-version:
  - welcome.html
```

*Slides can be shared across multiple decks.*

```bash
slides build --deck my-talk
```

## Commands

| Command                | Description                            |
|------------------------|----------------------------------------|
| `slides new <name>`    | Create a new presentation project      |
| `slides init`          | Initialize slides in current directory |
| `slides build`         | Generate HTML presentation             |
| `slides watch`         | Watch for changes and rebuild          |
| `slides export`        | Export presentation as PDF             |
| `slides import <file>` | Import from PowerPoint (.pptx)         |

## License

MIT

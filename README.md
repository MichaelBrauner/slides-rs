# Slides <img src="assets/templates/init/slides/assets/images/favicon.svg" width="40" height="40" alt="RS" style="vertical-align: middle; margin-bottom: -2px;">

Static site generator for HTML presentations.

**Your browser is the most powerful presentation tool you already have.**

## Why Slides RS?

**Web technologies are ideal for presentations:**

- **Reusable** - Templating allows sharing slides across decks
- **Version control** - Text-based files work perfectly with Git
- **AI-friendly** - Text formats enable AI assistants to read, generate, and edit content
- **Universal** - Runs in any browser, no special software needed
- **Accessible** - HTML supports screen readers and accessibility standards
- **Flexible styling** - CSS offers unlimited design possibilities
- **Rich media** - Native support for video, audio, SVG, animations
- **Responsive** - Adapts to any screen size

See [Comparison to PowerPoint](documentation/comparison.md) for details.

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

## Project Structure

```
my-presentation/
├── decks.yaml              # Deck configurations
├── slides/
│   ├── templates/          # All templates (slides, layouts, partials)
│   │   ├── _base.html      # Base layout (convention: _ prefix)
│   │   ├── overview.html   # Overview page template (optional)
│   │   ├── presenter.html  # Presenter view template (optional)
│   │   ├── intro.html
│   │   └── content.html
│   ├── assets/             # Static files (CSS, JS, images)
│   └── translations/       # Translation files (en.yaml, de.yaml)
└── output/                 # Generated HTML (gitignored)
```

## Writing Slides

### Templates

Templates live in `slides/templates/` as HTML files with [MiniJinja](https://docs.rs/minijinja/latest/minijinja/) syntax.

```html
{% extends "_base.html" %}

{% block content %}
  <h1>Hello World</h1>
  <p>{{ intro | markdown }}</p>
{% endblock %}

{% block notes %}
  Speaker notes go here - visible in presenter view
{% endblock %}
```

**Available filters and functions:**

- `{{ text | markdown }}` - Render Markdown to HTML
- `{{ "key" | trans }}` - Translate using translation files
- `{{ variable | dump }}` - Debug output for a variable
- `{{ dump() }}` - Debug output for all template variables
- `{{ dump(variable) }}` - Debug output for a specific variable
- `{{ asset("path/to/file") }}` - Reference asset files
- `{{ source("path/to/file") }}` - Include source code from file

**Available variables:**

| Description | Variable |
|-------------|----------|
| Deck information | `app.total`, `app.first`, `app.last` |
| Navigation | `slide.current`, `slide.prev`, `slide.next` |
| Position checks | `slide.isFirst`, `slide.isLast` |

*Supported extensions: `.html`, `.twig`, `.jinja2`, `.html.twig`, `.html.jinja`*

### Assets

Static files in `slides/assets/` (CSS, JS, images) are copied to the output directory.  
Reference them with the `asset` function:

```html
<img src="{{ asset('images/logo.png') }}">
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

### Special Templates

Two optional templates in `slides/templates/` generate extra pages:

| Template | Output | Unique Variables |
|----------|--------|------------------|
| `overview.html` | `output/overview.html` | `sections` (list with `key` and `slides`) |
| `presenter.html` | `output/presenter/slide-N.html` | `notes` (HTML of current slide's notes) |

*If missing, a warning is shown and the page is skipped. Run `slides init` to see code examples.*

## Commands

| Command                | Description                            |
|------------------------|----------------------------------------|
| `slides new <name>`    | Create a new presentation project      |
| `slides init`          | Initialize slides in current directory |
| `slides build`         | Generate HTML presentation             |
| `slides watch`         | Watch for changes and rebuild          |
| `slides export`              | Export presentation as PDF             |
| `slides import-images <file>`| Extract images from PowerPoint (.pptx) |

## Documentation

- [Installation Guide](documentation/installation.md)
- [Comparison to PowerPoint](documentation/comparison.md)
- [Architecture (Arc42)](documentation/architecture/)

## License

MIT

# 1. Einführung und Ziele

## 1.1 Aufgabenstellung

Slides-rs ist ein Static Site Generator für HTML-Präsentationen. Es ermöglicht Entwicklern und technischen Teams, Präsentationen mit denselben Werkzeugen und Workflows zu erstellen, die sie für Code verwenden.

### Wesentliche Features

| Feature | Beschreibung |
|---------|--------------|
| **Template-basiert** | Jinja2/Twig-kompatible Templates (MiniJinja) für flexible Slide-Erstellung |
| **Deck-Konfiguration** | YAML-basierte Definition von Präsentationen mit Sektionen |
| **Mehrsprachigkeit** | Übersetzungen via YAML-Dateien mit `trans`-Filter |
| **Asset-Pipeline** | Automatisches Kopieren von CSS, JS, Bildern |
| **Thumbnails** | Automatische PNG-Generierung via Headless Chrome |
| **Watch-Modus** | Live-Reload bei Dateiänderungen |
| **Views** | Overview, Presenter-Ansicht mit Speaker Notes |

### Abgrenzung

Slides-rs ist **kein**:
- WYSIWYG-Editor (kein GUI)
- PowerPoint/Keynote-Ersatz für nicht-technische Nutzer
- Markdown-zu-Slides Konverter (Templates sind HTML/Twig)

## 1.2 Qualitätsziele

| Priorität | Qualitätsziel | Beschreibung |
|-----------|---------------|--------------|
| 1 | **Entwickler-Experience** | Präsentationen mit vertrauten Tools erstellen: Git, IDE, CLI |
| 2 | **Flexibilität** | Volle Kontrolle über HTML/CSS/JS ohne Einschränkungen |
| 3 | **Versionierbarkeit** | Alle Artefakte sind textbasiert und git-freundlich |
| 4 | **Performance** | Schneller Build-Prozess, auch bei vielen Slides |
| 5 | **Erweiterbarkeit** | Eigene Templates, Layouts, Stimulus-Controller |

## 1.3 Stakeholder

| Rolle | Erwartungshaltung |
|-------|-------------------|
| **Entwickler** | Präsentationen wie Code behandeln: versionieren, reviewen, automatisieren |
| **Trainer/Dozenten** | Wiederverwendbare Slide-Komponenten, einfache Mehrsprachigkeit |
| **Agenturen** | Branded Templates, konsistentes Corporate Design über Projekte hinweg |
| **DevOps** | CI/CD-Integration, automatisierte Builds, PDF-Export |

## 1.4 Kontext

```
┌─────────────────────────────────────────────────────────────────┐
│                        Entwickler                                │
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

## 1.5 Technologie-Stack

| Komponente | Technologie | Begründung |
|------------|-------------|------------|
| Sprache | Rust | Performance, Safety, Single Binary |
| Templates | MiniJinja | Jinja2-kompatibel, kein Python-Dependency |
| CLI | clap | Standard für Rust CLIs |
| File Watching | notify | Cross-platform, performant |
| Thumbnails | headless_chrome | Pixel-perfekte Screenshots |
| YAML Parsing | serde_yaml | Standard für Rust |

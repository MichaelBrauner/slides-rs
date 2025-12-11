# QS-004: Minimal Configuration

**Quality Attribute:** [Usability - Simplicity](../10-quality-requirements.md#102-usability)

## Stimulus
User wants to create presentation without extensive configuration

## Environment
Normal operation, users with varying technical backgrounds

## Artifact
Configuration system, CLI interface

## Response
1. Single configuration file (`decks.yaml`)
2. Sensible defaults for all optional settings
3. Convention over configuration
4. Three commands cover 95% of use cases: `init`, `build`, `watch`

## Response Measure
Minimum viable presentation requires only `decks.yaml` and template files; zero configuration needed for asset handling, translations lookup, output structure

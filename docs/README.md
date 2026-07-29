# Godlint documentation

## Using Godlint

| | |
| --- | --- |
| [Rule reference](rules.md) | Every rule, what it measures, and what it cannot see yet |
| [Configuration](configuration.md) | `godlint.yaml`: suites, thresholds, severities, exclusions |
| [Using Godlint in CI](ci.md) | The GitHub Action, output formats, annotations |
| [Inline suppression](suppressions.md) | Exempting a single site, accountably |

## Understanding Godlint

| | |
| --- | --- |
| [Product scope](product-scope.md) | The promise, the MVP boundary, and the non-goals |
| [Rule roadmap](rule-roadmap.md) | What is shipped, what is next, and why each threshold is the number it is |
| [Architecture](architecture.md) | Crate boundaries, and how a language's parser details stay behind one |
| [Dogfooding](dogfooding.md) | How Godlint enforces policy on its own repository |

## Working on Godlint

| | |
| --- | --- |
| [Local development](local-development.md) | Building, testing, and running Godlint on itself |
| [Testing strategy](testing.md) | Fixture-first testing and the required validation layers |
| [Releasing](releasing.md) | The tag-driven release, the registries, and the floating `v1` tag |
| [Contributing](../CONTRIBUTING.md) | Proposing a rule, branch naming, labels |

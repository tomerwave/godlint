# Godlint documentation

Godlint is the executable engineering constitution for repositories maintained by
humans and coding agents. Start with the rule reference to use it or the product scope
to understand what belongs in it.

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
| [Product scope](product-scope.md) | The vision, product promise, boundaries, and success criteria |
| [Rule roadmap](rule-roadmap.md) | What is shipped, what is next, and why each threshold is the number it is |
| [Enforceable practices research](enforceable-practices.md) | Which Node, SOLID/DRY/KISS, Rust, and PEP 8 ideas belong in Godlint |
| [Architecture](architecture.md) | Crate boundaries, and how a language's parser details stay behind one |
| [Dogfooding](dogfooding.md) | How Godlint enforces policy on its own repository |

## Working on Godlint

| | |
| --- | --- |
| [Local development](local-development.md) | Building, testing, and running Godlint on itself |
| [Testing strategy](testing.md) | Fixture-first testing and the required validation layers |
| [Releasing](releasing.md) | The tag-driven release, the registries, and the floating `v1` tag |
| [Contributing](../CONTRIBUTING.md) | Proposing a rule, branch naming, labels |
| [Propose a rule](skills/propose-a-rule.md) | The three decidability filters and the standard issue shape |
| [Add a rule](skills/add-a-rule.md) | The ten places one rule touches, mirrored from the validator |
| [Propose a threshold](skills/proposing-a-threshold.md) | Measuring a limit against this repository instead of borrowing one |

# Product scope

## Vision

Godlint is the executable engineering constitution for repositories maintained by
humans and coding agents.

Teams already record engineering decisions in architecture guides, contribution
rules, review checklists, and prompts. Those instructions are useful, but they are
easy to miss and expensive to enforce consistently. Godlint turns the decisions that
can be checked mechanically into deterministic, versioned repository policy.

## Problem

Coding agents increase implementation speed, but they do not reliably preserve a
repository's intent. More code can mean more architectural drift, accidental
dependencies, restricted API use, inconsistent conventions, and unowned exceptions.

Human review cannot scale linearly with generated output. Prompts help agents reason,
but remain probabilistic and vary across tools. Repositories need a policy layer that
is faster than review, more durable than a prompt, and deterministic enough to block
CI.

## Product promise

Define an engineering policy once and enforce it across supported languages,
regardless of whether a human or an agent wrote the code.

The same repository and configuration must produce the same findings, ordering, and
exit status. An LLM may help author or remediate policy, but it must never decide
whether CI passes.

## Primary users

- Teams using coding agents that need generated changes to respect repository rules.
- Platform and architecture teams that maintain standards across many repositories.
- Maintainers who want review time spent on judgment rather than repeatable checks.
- Coding agents that need precise, machine-readable feedback before opening a pull
  request.

## Core workflow

```text
Engineering decision
        ↓
Versioned Godlint configuration
        ↓
Local, agent, and CI checks
        ↓
Precise finding or accountable exception
        ↓
Repository standards remain intact as output scales
```

## Product principles

### Deterministic enforcement

Policy must be reproducible, reviewable, and safe to make blocking. Probabilistic
analysis cannot determine pass or fail.

### Repository policy over language trivia

Godlint owns organization-level and repository-level decisions that should hold
across languages. It delegates formatting, compiler errors, type checking, and broad
language-specific bug finding to existing tools.

### High confidence

A smaller set of explainable findings is more valuable than noisy coverage. A rule
earns default enforcement only when its behavior and false-positive boundary are
understood.

### Agent-readable by design

Findings need stable identifiers, exact locations, evidence, and actionable
remediation. Structured output is a product surface, not an afterthought.

### Local and private by default

Source analysis runs locally. Godlint must not require source upload, a hosted
service, or network access to enforce policy.

### Accountable exceptions

Suppressions are explicit, scoped, owned, and time-bound. An exception is visible
policy debt, not a hidden escape hatch.

### Incremental adoption

Teams can start with a suite, tune individual rules, exclude generated inputs, and
introduce stricter policy without rewriting their toolchain.

## Current scope

Godlint currently provides:

- One CLI and one versioned configuration file.
- Composable policy suites and per-rule overrides.
- Rust, Python, JavaScript, and TypeScript analysis, and GitHub Actions workflow analysis.
- Thirty-seven deterministic maintainability, policy, security, testing, and style rules.
- Terminal, GitHub annotation, JSON, and SARIF output.
- Scoped inline suppressions with owner and expiry metadata.
- Local execution and CI integration without a hosted service.

## Strategic rule areas

Future rules should deepen the constitution where deterministic evidence exists:

- Architectural boundaries and dependency direction.
- Restricted APIs, packages, and infrastructure access.
- Configuration, environment, and secret-handling policy.
- Test and error-handling requirements.
- Similarity and mutation evidence where syntax alone cannot prove quality.
- Suppression ownership and policy-debt visibility.
- Cross-language consistency for shared engineering decisions.

Godlint may compose pinned deterministic companion tools when a mature specialist owns the
evidence. It preserves the specialist's rule identity and does not present delegated analysis as a
native Godlint rule.

## Non-goals

Godlint is not:

- An AI code reviewer or a replacement for engineering judgment.
- A compiler, formatter, type checker, or general-purpose language linter.
- A hosted source-analysis service.
- A probabilistic CI gate.
- An automatic architecture designer or large-scale code rewriter.
- A universal cross-language semantic model.
- An arbitrary third-party plugin runtime in the early product.

## Success

Godlint succeeds when:

- A repository can adopt useful policy in minutes.
- Coding agents can detect and correct violations before opening a pull request.
- The same policy behaves consistently across supported languages and environments.
- Teams trust high-confidence rules enough to block changes.
- False positives are measured, rare, and straightforward to inspect.
- Exceptions remain visible and accountable.

The north-star outcome is:

> More code can be safely produced without weakening the repository's engineering
> standards.

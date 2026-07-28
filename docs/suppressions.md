# Inline suppression

A rule is only adoptable if a repository can record the cases it cannot fix yet. Godlint
had two ways to narrow a rule and neither could carry accountability: an `exclude` glob
removes a path from the scan entirely, and `allow-names` on
`maintainability/empty-function` applies repository-wide. Neither can say *this one site
is a known exception, for this reason, owned by this person, until this date* — which is
exactly what [the dogfooding policy](dogfooding.md) requires of an exception.

Inline suppression closes that gap. A comment at the site names the rules it silences and
carries the reason.

## Syntax

```text
godlint-ignore-next-line <rule-id>[,<rule-id>...] [owner=<name>] [expires=<YYYY-MM-DD>] -- <reason>
godlint-ignore-enclosing <rule-id>[,<rule-id>...] [owner=<name>] [expires=<YYYY-MM-DD>] -- <reason>
```

The directive must open its line, ignoring leading whitespace and the comment's own
punctuation, so prose that merely mentions a directive is not one. Which punctuation counts
depends on the comment: `/`, `#`, `*` and `!` always, and a quote only on the line where a
quote opens the comment, which is a Python docstring. Without that restriction
`// 'godlint-ignore-next-line …'` would be a live suppression rather than a sentence about
one, and a quoted line in the middle of a docstring would be a directive.

Opening a directive and closing a comment are separate questions. A docstring's final
`"""` is still comment furniture on whatever line it falls, so a directive on its own line
inside a multiline docstring reaches the code after the docstring rather than the closing
delimiter.

An option given twice keeps its first value and is reported. Otherwise an expiry could be
renewed by appending a second one, invisibly to both `check` and the audit. An option with
an empty value, `owner=`, reads as absent rather than as satisfied. It works in every comment syntax
Godlint reads, including Python docstrings, because it is resolved from `CommentFact`
rather than by re-scanning the file:

```rust
// godlint-ignore-next-line maintainability/function-nesting owner=tomer expires=2026-12-31 -- flattening in #482
fn nested(flag: bool) {
    if flag {
        if flag {
            work();
        }
    }
}
```

```python
def blank():
    # godlint-ignore-enclosing maintainability/empty-function owner=tomer -- awaiting #483
    pass
```

A comment that is **only** directives is exempt from `style/no-comments`. A directive is
machine-readable policy metadata rather than prose beside the code, and a rule that
forbade it would make suppression unusable in any repository that adopts that policy —
including this one.

The exemption is scoped to directive-only comments on purpose. Exempting any comment that
*contains* a directive would be a bypass: one valid directive would launder arbitrary
prose past a rule set to `error`.

```rust
/*
This prose is reported, because the comment is not only a directive.
godlint-ignore-next-line maintainability/empty-function -- reason
*/
```

Blank lines and the comment's own delimiters do not count against the exemption, so a
block comment wrapping a directive on its own line is still exempt. Prose that belongs
with an exception goes in the justification after `--`, where the audit can see it.

## Scope

| Directive | Silences findings on |
| --- | --- |
| `godlint-ignore-next-line` | the first line after the directive that is not the rest of its own comment |
| `godlint-ignore-enclosing` | the innermost declaration containing the directive, excluding any declaration nested inside it |

There is deliberately no file-wide directive. A file-wide suppression is an `exclude`
entry with less visibility, and the point of this feature is visibility.

"The rest of its own comment" matters for a block comment. Taken literally, the next line
after the directive below is `*/`, so the directive would silence nothing and say nothing
— a silent no-op in a feature whose purpose is that exceptions are visible. The closing
delimiter is skipped instead, and this reaches `fn example`:

```rust
/*
godlint-ignore-next-line maintainability/empty-function -- reason
*/
fn example() {}
```

Which scope to reach for follows from where a finding is anchored. A function-level
finding is reported at the line the function opens, so a directive above the declaration
reaches it with `next-line`. A finding anchored inside a body — a comment, a nested block
— is reached by putting `enclosing` inside that body. `enclosing` also covers the
declaration line, so a directive inside a function can silence a finding about the
function itself.

`enclosing` covers a byte range, not a line range, and it excludes declarations nested
inside the one it resolves to. Both matter:

```ts
export const a = (): void => { /* godlint-ignore-enclosing … -- a is a no-op */ }; export const b = (): void => {};
```

`b` shares a line with `a` and is still reported, because it is a different declaration.

```rust
fn outer() {
    // godlint-ignore-enclosing maintainability/empty-function -- outer is a stub
    let inner = || {};
}
```

The closure is still reported. A justification describes one site, and a declaration nested
inside the one it names is a different site with a reason of its own — the same rule the
function metrics follow, where a nested function is measured on its own body rather than
folded into its parent. To except the closure, put a directive inside the closure; it then
resolves to the closure, because resolution picks the innermost declaration.

`enclosing` needs a function to enclose it. At the top level of a file there is none, and
Godlint reports the directive rather than silently ignoring it.

File-level rules such as `maintainability/file-size` cannot be suppressed inline: their
findings sit at line 1, where no preceding line exists, and no function encloses them.
That is what `exclude` remains for.

## Accountability

Suppression is only trustworthy if the suppressions themselves are checked, so
`policy/accountable-suppression` reports a directive that cannot account for itself:

| Reported when | What to do |
| --- | --- |
| No `-- <reason>` | State why the exception exists |
| No rule named | List the rule IDs the directive applies to |
| An unknown rule ID | Fix the typo; the directive was silencing nothing |
| `policy/accountable-suppression` named | It cannot be suppressed; nothing else would hold suppressions to account |
| An unrecognised option or stray word | Fix the directive |
| The same option set twice | Keep one value; the first is the one that applies |
| `expires` that is not a calendar date | Write it `YYYY-MM-DD` |
| `expires` in the past | Fix the code, or renew the exception deliberately |
| No `owner`, when `require-owner` is set | Name someone accountable |
| No `expires`, when `require-expiry` is set | Set a date |
| `godlint-ignore-enclosing` with nothing to enclose | Move it inside the declaration |

```yaml
rules:
  policy/accountable-suppression:
    severity: error
    require-owner: false
    require-expiry: false
```

Two decisions are worth stating plainly.

**A defective directive still suppresses.** An unjustified or expired directive silences
what it names and is reported against itself. The alternative — revoking its power —
means that the day an expiry passes, CI fails with an avalanche of unrelated findings
instead of one finding that names the directive and the date. Accountability is preserved
because the report cannot itself be suppressed; set the rule to `error` and an expiry is
a build failure with a single clear cause.

**A repository that never enables the rule gets unaccountable suppressions.** Directives
work whether or not `policy/accountable-suppression` is configured. That is a
configuration choice like any severity, and the rule is enabled in Godlint's own
`godlint.yaml` and belongs in any suite that promotes rules to blocking.

`policy/unused-suppression` reports a directive that names an enabled, suppressible rule
but does not silence any finding. It is how exceptions disappear once the code is fixed.
A directive for an off rule is dormant, not unused: enabling a rule gradually must not
turn the inactive parts of a policy into failures. Like
`policy/accountable-suppression`, this rule cannot be suppressed.

Because expiry compares against the current date, `godlint check` is time-dependent by
design. It is the only such input, it is passed in explicitly rather than read inside a
rule, and the fixtures pin dates far in the past and future so the corpus stays
deterministic.

## Auditing

```bash
godlint suppressions
godlint suppressions crates
```

The command lists every directive in scope with its location, scope, rules, owner,
expiry, and reason, then the total. A suppression with no reason is listed as
`(no justification)` rather than omitted, so the audit shows the gap. This is the
listing the roadmap asks for: the total is a number someone can look at, rather than
something discovered one grep at a time.

## Godlint's own exceptions

There are none. `godlint suppressions` prints `No suppressions.` and that is the whole
inventory.

That is worth recording, because there briefly was one and how it went away is the point
of the feature. `impl fmt::Display for SuppressionDefect` is an eleven-arm exhaustive
`match` in which every arm is a single `write!`, and
`maintainability/decision-complexity` reported it at 11 against a limit of 10. Raising the
limit would have weakened the rule everywhere to accommodate one site, and splitting the
impl would have made it less readable purely to move a number, so the site took a
directive with an owner, an expiry, and a reason, and
[#30](https://github.com/tomerwave/godlint/issues/30) recorded the question it rested on:
should the metric count each arm of an exhaustive `match` at all?

Answering that question changed the metric — a multiway branch now counts once, and a
guard on an arm counts, which it previously did not — and the exception was deleted rather
than renewed. An exception with an expiry and a stated reason is a question someone can
answer. That is the difference between it and a widened threshold, which is an answer
nobody will revisit.

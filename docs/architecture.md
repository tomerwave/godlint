# Architecture

Godlint has a Rust core that coordinates configuration, file discovery, language
analysis, rules, repository facts, and reporting.

```text
CLI -> config + discovery -> language analyzers -> shared facts
    -> rules -> normalized diagnostics -> reports
```

Source code carries no explanatory comments, so this document is where the reasoning
behind a boundary lives. If a decision below is not obvious from the code, that is
expected: read it here.

## Module map

| Module | Owns |
| --- | --- |
| `source` | Repository-relative paths, the language enum, byte ranges, and position derivation |
| `paths` | Every decision about path safety |
| `glob` | Path-glob matching for configured exclusions |
| `discovery` | Walking a scope and selecting supported files |
| `scan` | Reading files and turning them into facts, recording per-file failures |
| `analyzers` | Parsing, and extraction of language-neutral facts |
| `analyzers::vocabulary` | The questions the extractor asks a grammar |
| `analyzers::metrics` | Derivation of the function metrics |
| `facts` | The fact types rules consume |
| `rules` | Rule declarations, the drivers that run them, and the registry |
| `config` | The configuration schema and its validation |

## Facts

Language adapters retain native AST and parser details. They emit a small shared fact
model that rules consume without a universal AST. `FunctionFact`, `CommentFact`,
`CallFact`, and `AccessFact` exist today. `CallFact` records a direct callee path and
source range; `AccessFact` does the same for direct member access. Neither resolves
aliases, types, or dynamically computed properties. `Import`, `EnvironmentRead`,
`ErrorHandler`, `Assertion`, `Mock`, and `DependencyEdge` are planned and are described in the
[rule roadmap](rule-roadmap.md).

Source files are identified with repository-relative paths and a shared language enum.
Ranges use byte offsets internally; the source contract validates them and derives
one-based line and Unicode-scalar-column positions only at reporting boundaries. That
derivation binary-searches a line-start index built once per file, because scanning to
the offset instead would make reporting cost grow with a finding's distance into the
file — which is invisible while rules fire rarely and quadratic once one fires per
comment. A
UTF-8 byte-order mark is stripped when a `SourceFile` is created, so byte offsets, line
accounting, and reported columns all describe the source a reader sees rather than an
invisible prefix.

`FunctionFact` carries a source file, an optional name, the whole-function range, the
body range, and the function metrics. Each metric is its own type rather than a bare
`u32`. They are all counts, so a single numeric type would let any two of them be
transposed at a construction site without the compiler objecting.

`CommentFact` records a range and a kind: a line comment, a block comment, a
documentation comment, a Python docstring, or an interpreter shebang. A docstring is a
string expression rather than a comment token, but it plays the role a block comment plays
elsewhere, so policy that skips or inspects commentary has to be able to see it. A shebang
is a comment token that is not commentary at all, and classifying it once means no rule
has to recognise it again: without that, one rule exempts it and the next silently does
not.

Which syntax counts as documentation is a per-language judgement and belongs to the
language module. Rust documents with `///`, `//!`, `/**` and `/*!`. JavaScript and
TypeScript document with JSDoc `/** */` only — `///` there introduces a compiler
directive such as `/// <reference types="node" />`, which is a line comment and not
documentation. Python documents with a docstring. Deciding this centrally would have
exempted TypeScript directives from comment policy as though they were prose.

## The language boundary

Rules consume language-neutral facts, and the extractor that builds them is
language-neutral too. Every judgement about what a given node *means* is answered by the
owning language module through `analyzers::vocabulary::Vocabulary`: whether a node is a
function, a block, a conditional, a branch point, an exit, a placeholder body, a
receiver parameter, an abstract declaration, or a kind of commentary.

Nothing in `analyzers::metrics` names a grammar node kind. A new language is added by
describing it in one module rather than by editing the walks, and no walk can quietly
grow a special case for one grammar. `analyzers::ecmascript` is shared by the
JavaScript, TypeScript, and TSX analyzers because those grammars name these nodes
identically, so the three stay in lockstep by construction.

One consequence is worth stating explicitly, because it is the reason the boundary is
drawn here rather than inside each rule: a grammar names a keyword token after the
construct it introduces, so Python's `lambda` keyword has the same node kind as the
lambda itself. Structural predicates therefore apply only to named nodes.

Tree-sitter and its official Rust, JavaScript, TypeScript/TSX, and Python grammars
provide the syntax boundary. The adapters retain Tree-sitter nodes and byte spans; no
parser type crosses into rules, findings, or reporters.

## Rules

A rule declares its identity and how to read its severity, then implements one trait per
fact scope: a function rule, a file rule, or a comment rule. A shared driver per scope
runs it over the fact set. This is why the severity gate is evaluated once rather than
per function, and why no rule can forget to honour it.

Most rules compare one measurement against one ceiling, and those implement a limit
trait instead: they declare the metric they report under, how to measure it, and how to
read the ceiling from configuration. They do not express the comparison. Writing
`actual > max` once in the driver rather than once per rule means a rule cannot invert
the test, and pairing the metric with the rule as an associated constant means it cannot
report under another rule's metric — neither mistake is available to make. The
measurement receives the configuration, because whether blank lines and commentary count
is part of measuring, not part of comparing.

The registry is a table of evaluators. Adding a rule appends an entry rather than growing
a branch in a dispatcher, which had previously pushed the dispatcher's own decision
complexity to the repository's configured limit.

A finding carries a typed violation rather than a prepared sentence. Reporters other than
the terminal need the numbers, and a rendered message must never be load-bearing:
findings are ordered by path, line, column, and rule identifier, so output order cannot
depend on wording.

`rules::line_count` identifies commentary from the comment facts the analyzer already
produced rather than by re-scanning text for `//` and `#`. Re-lexing there would
duplicate the parser, put language knowledge in the rules layer, and get string
literals, nested block comments, and Python docstrings wrong.

It relies on comments being reported in ascending source order, which the extractor's
pre-order walk guarantees and a test pins. That ordering is what lets the line walk carry
a cursor forward instead of re-examining every comment: a comment that ends before the
current line cannot matter to any later line. Without it, counting a file costs its line
count times its comment count, which is invisible on ordinary source and pronounced on a
heavily annotated file.

## Configuration

Two rules share `LineLimitRule` because `function-size` and `file-size` ask the same
question of different ranges. The rules whose whole configuration is a severity and one
ceiling are generated from one declaration, since each needs its own YAML key and cannot
literally be one type.

A ceiling of zero is accepted where forbidding a construct outright is a real policy —
depth, parameters, returns, statements — and rejected where it falls below the metric's
own floor, which is the case for lines and complexity.

`fail-on` decides which severity blocks. Without it every severity blocks equally, and
the confidence ladder that lets a rule be adopted as a warning first cannot exist.

Exclusions are configured policy rather than a constant, so a repository can keep a
virtual environment, a build directory, or deliberately non-conforming fixture data out
of its results without weakening a rule for everyone. `glob` is deliberately small:
repository policy needs `*`, `?`, and `**` over path segments, and nothing there
justifies a dependency or a regex engine.

Configuration discovery stops at a directory containing `.git`. Walking to the filesystem
root would let a stray `godlint.yaml` in a parent or home directory silently govern an
unrelated repository and relocate the reported path root.

## Path safety

`paths` is the single place path safety is decided. Answering "does this path escape the
repository" or "is a symlink involved" in several modules invites them to disagree, and
this is the boundary that keeps analysis inside the tree the operator pointed at.

Discovery also stops at a nested `.git` boundary, so walking a parent does not descend
into an embedded repository or submodule. `paths::is_repository_root` answers what counts
as a repository for both configuration discovery and scan discovery; deciding it twice is
how the two would come to disagree about where a repository ends.

The rule is about recursion, not about policy ownership: a child reached by walking is
skipped, while a child named as a requested path is scanned under whatever configuration
`config_root` resolves. `godlint check . nested` therefore does scan `nested` under the
parent's policy, because the first requested path decides the configuration root. What a
user cannot do is `godlint check nested` from a parent whose configuration it would need,
since configuration discovery stops at the same boundary — that invocation asks for a
repository that must carry its own `godlint.yaml`.

Two properties of the test are deliberate rather than incidental. It is `exists`, so any
entry named `.git` marks a boundary whether or not git would agree, which keeps the check
independent of git's on-disk formats; a `.git` file for a worktree or submodule counts
exactly as a `.git` directory does. And it fails open: an unreadable or dangling `.git`
reads as "not a boundary" and the subtree is walked, because a linter that scans too much
reports something a reader can dismiss, while one that scans too little reports nothing at
all.

Skipping is silent. A nested repository produces no finding and no issue, which is the
same shape as an `exclude` entry and the reason both belong in documentation rather than
only in output.

## Failure handling

A failure specific to one file — unreadable bytes, invalid syntax — is recorded against
that file and reported alongside the findings. It must not abort the run, because
discarding every other finding turns one bad file into a silent pass.

## Crate boundaries

Start with only `godlint-cli` and `godlint-core`. Add fixture-test support,
configuration, diagnostics, analyzers, rules, graph, cache, SARIF, and external tools
as dedicated crates only after their ownership boundaries are proven by real code.

Semantic workers and external ecosystem-tool adapters are post-MVP capabilities.

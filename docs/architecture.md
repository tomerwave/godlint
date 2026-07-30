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
model that rules consume without a universal AST. `FunctionFact`, `CommentFact`, `CallFact`,
`AccessFact`, `ErrorHandlerFact`, `ImportFact`, and `TestFact` exist today. `CallFact` records a direct callee path, a
source range, argument count, and whether the call site was a macro invocation; `AccessFact` does the same
for direct member access. Neither resolves aliases, types, or dynamically computed
properties.

A callee and an access target are resolved when the fact is built rather than read back out
of the range, because the path a rule matches is not always the text a file spells. The
range still locates the finding, so a report points at what the author wrote. Which
spellings denote the same path is a per-language judgement and belongs to the language
module: `process?.env` and `process.env` are one read, so JavaScript and TypeScript
normalize optional member access before answering, while Rust and Python have no such form
and answer with the plain spelling. Deciding this in the shared extractor would have put
one language's punctuation in code that must not know any. The argument count excludes comments: a grammar reports them as named nodes
inside the argument list, so counting named children alone would read
`setTimeout(work /*, 100 */)` as a call that passes two arguments.

The macro flag exists because a name is not enough to identify a callee in Rust. A grammar
names a macro without its `!`, so a `fn dbg` and the `dbg!` macro reach a rule as the same
string, and restricting one restricted the other. Rules spell a macro callee with its `!`,
which is both how Rust writes it and the name a finding reports, so the name a reader sees
is the name they configure.

Naming a callee under `calls` scopes the restriction that already exists rather than
redefining it. A built-in name stays bound to the language that defines it, so giving
Python's `sys.exit` an `allow-in` boundary leaves a call spelled `sys.exit` in TypeScript alone.
A name the project invents belongs to no language and applies wherever it is called, which is
what a policy about `loadConfig` means. Moving the debug-output built-ins to
`logging/no-production-log` moved their binding with them: `print` named under `calls` is now an
invented name and applies everywhere, while the logging rule keeps it bound to Python.

The unstated cost is that a project cannot restrict a callee of its own whose name a built-in
already claims, and the failure is silence rather than a diagnostic. Resolving it needs a
language key in the configuration, which is a schema addition rather than a restructuring:
the table already carries the dialect, so the key would narrow a configured entry the same
way the table narrows a built-in.

One table pairs each built-in callee with the dialect that speaks it, and both questions the
rule asks — is this name a built-in anywhere, and is it restricted in this call's dialect —
are answered from that table. Splitting them across a list per language meant a new
restriction had to be added twice, and forgetting the second made a built-in silently
language-agnostic again. A macro carries its own `!`, so one dialect per
language suffices and the name alone separates `dbg!` from a `fn dbg`.
`rules::catalogue` owns that table shape, the dialect a language speaks, the macro-aware
spelling of a callee, and the path allowance every one of these rules needs, so four rules
answer those questions the same way rather than four similar ways.
`EnvironmentRead`, `Assertion`, `Mock`, and `DependencyEdge` are planned and are
described in the [rule roadmap](rule-roadmap.md).

`TestFact` records that a declaration is a test: its range, its name, the marker that made it one,
and whether that marker carried focus or skipping. What counts as a test is a framework question
rather than a language one, and the three conventions have nothing structurally in common, so each
language module answers it. Rust reads the attributes preceding a function, which are siblings rather
than children and which stack, so `#[test]` and `#[ignore]` in either order describe the same test.
Python reads a `test_` prefix or a `pytest.mark` decorator, taking the callee when the decorator is
called so `@pytest.mark.parametrize('x', [1])` reports the marker and not its arguments. JavaScript
and TypeScript read a call to a runner and its member, so `it.only` and `describe.skip` carry focus in
the name.

The fact stops at syntax, which is narrower than the proposal it came from. That asked for the marker
*or the path* that made something a test, and for framework names to be configurable. An analyzer
sees neither: it is handed one file and no configuration, and adding either would make fact
extraction depend on policy. A rule has both, so a rule that wants to treat everything under `tests/`
as a test combines this fact with a path glob, the way the call rules already combine a callee with
`allow-in`.

Nothing about a test's contents is stored. The seven rules waiting on this ask whether a test is
empty, whether it branches, and whether it sleeps or reaches the network — all of which are questions
about other facts falling inside the test's range, which `TestFact::contains` answers. Copying a
body's statistics into the fact would duplicate what `FunctionFact` already measured.

A call fact carries two ranges. `range` locates the callee, which is where a finding points, and `extent`
spans the whole call expression. The second exists because some findings are a *shape* rather than a name:
`testing/no-sleep-in-test` recognises JavaScript's commonest test sleep as a timer nested inside a
`Promise`, and nesting is a question about extents. A callee range cannot answer it — the callee of
`setTimeout` sits outside the callee of `Promise`, not inside it.
`AssertionFact` records an assertion's range, its spelled name, whether it was a macro, and how many
operands it took. Which calls count is a framework question, so each language module answers it the way
it answers what a test is, and each answers a different shape of question. Python has assertion
*syntax*: `assert value == 1` is a statement, not a call, so no call fact would ever have seen it.
Rust's assertions are macros, which the six-name list matches exactly. JavaScript has neither — an
assertion is `expect`, a type assertion such as `expectTypeOf` or `assertType`, or the `assert` module —
so the fact reads the callee. Type assertions are in that list because a typed suite may have no other
kind: measured against zod, they account for most of what its tests assert.

Two choices in that matching are deliberate. The names are explicit sets rather than an `assert`
prefix, because a prefix silently claims a domain helper called `assert_invariant`, and #90 rejected
that before the code was written; a test asserts exactly that. And `expect(value).toBe(1)` produces one
assertion rather than two: the matcher is a second call on the same chain, and counting it would double
every count that `assertion-required` is meant to read.

Rust has one assertion that is not a call at all: `#[should_panic]`. The attribute *is* the assertion,
so the fact records it, named `should_panic` with no operands. Its range is the function's rather than
the attribute's, because the attribute precedes the function and would otherwise fall outside the test
that owns it, and every rule asking "does this test assert" works by range containment. Without this,
`assertion-required` reports every `should_panic` test in every Rust repository.

`operands` counts what the assertion was given, so a Rust `assert!(value, "explains")` is two. Reaching
that number in Rust means counting commas in a `token_tree`, because tree-sitter does not parse a macro's
arguments, and every part of that count was wrong before it was right. Commas inside a nested tree belong
to that tree. A trailing comma is punctuation. And a comma between *type* arguments separates nothing:
`assert_eq!(HashMap::<String, u32>::new(), m)` takes two operands, not three. Type arguments are found by
the turbofish, because in expression position generics need one — a bare `<` is a comparison, and
swallowing everything after it would lose the message in `assert!(a < b, "explains")`. tree-sitter spells
`>>` as a single token, which closes two levels and is also the shift operator, so it saturates rather
than underflows. Each of those four is pinned by a test.

Three boundaries are deliberate and none is a defect, but none was obvious either. A path-qualified macro
is not an assertion: `static_assertions::assert_eq!` is a compile-time check rather than a test
assertion, and the cost is that `core::assert_eq!` is missed too. `should`-style JavaScript —
`x.should.equal(1)` — is not recorded, because the assertion is a property access rather than a call.
Neither is `raises(...)` reached through `from pytest import raises`; that is the same alias limit the
call rules document. All three want either a name list a repository can extend or import resolution.

The fact deliberately does not record whether an operand *was* the message. That takes a per-name arity
table for three ecosystems — `assert.equal(a, b, msg)` puts it third and `chai.expect(value, msg)`
second, while Jest's `expect` has no message argument at all — and a wrong table would make
`assertion-message-required` demand a message where the framework has none. The rule that needs it
brings it, with its own review.

`ErrorHandlerFact` records the handler's range and whether its body only stands in for one. The
adapter finds that body by looking for it rather than by position: a Python `except_clause` puts the
exception it caught before the block, so the first named child of `except ValueError as error:` is
the exception, and reading position instead of kind silently limited the rule to a bare `except:` —
the one form of the clause that a real codebase writes least often.

Source files are identified with repository-relative paths and a shared language enum.
Ranges use byte offsets internally and derive one-based line and Unicode-scalar-column
positions only at reporting boundaries.

A range is built by the file it indexes and by nothing else: `SourceFile::range` is the only
constructor that takes offsets, and it rejects an offset past the end, an offset off a UTF-8 boundary, and a
start after its end. A `SourceRange` that exists is therefore a range that has already been
checked, so locating one cannot fail, and no fact needs to re-validate what its type already
records. That is what makes rule evaluation infallible — the single error a rule could once
report was a location failure that could not occur, and an error variant that cannot happen
still has to be handled by every caller. That
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
fact scope: a function rule, a file rule, or a comment rule. A file rule receives the source
file rather than its facts, which is what lets `architecture/filename-case` judge a path without
reading any syntax. `SourceFile` keeps its path as text beside the `Path`, so a rule that reads a
name does not allocate one per file. That text is spelled with forward slashes on every platform,
which is not cosmetic: a configuration writes `src/ui/**`, and on Windows a native separator made
every such pattern match nothing, so excluded directories were scanned and a file name was reported
as its whole path. A shared driver per scope
runs it over the fact set, and every scope's driver reports through one kernel. This is why
the severity gate is evaluated once rather than per function, and why no rule can forget to
honour it.

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

A suite is one entry in a table of expansions, looked up by name. A name that is not in the
table is a configuration error rather than a suite that expands to nothing, because the
lookup that finds the expansion is the same lookup that decides the name is real — there is
no second list of valid names to keep in step. Expansion runs before validation, so the
validators see the configuration that will actually run, and it inserts only where a rule is
absent, which is what makes an explicit `rules:` entry win.

The suite stays typed Rust rather than an embedded document. Its thresholds are `NonZeroU32`
constants and its severities are enum variants, so a mistyped value fails the build; a parsed
document would move that failure to the user's first run. What a document would have bought —
one home for each default value — comes instead from the suite calling the same `serde`
default functions the configuration schema uses, and the published thresholds are held to
`docs/rule-roadmap.md` by a test.

A reporter decides how a finding is spelled, and the rules know nothing about it. That is what lets
the same run produce a terminal line, a GitHub annotation, a JSON document, or SARIF without a rule
choosing a wording for a machine — and it is why a finding carries a typed violation rather than a
prepared sentence.

A finding carries a typed violation rather than a prepared sentence. Reporters other than
the terminal need the numbers, and a rendered message must never be load-bearing:
findings are ordered by path, line, column, and rule identifier, so output order cannot
depend on wording.

A call and an access carry a range and read their text from it, the way `CommentFact`
already does, rather than storing a copy of the text beside the range. Two fields for one
truth can disagree, and a test asserting a callee of `inner` for a range spelling `inner()`
is what that drift looks like. `SourceFile` holds its path behind an `Arc` because a fact
clones the file it came from, and an owned path allocated on every one of those clones.

`ImportFact` carries the range that spells the imported module and reads the module from it.
Which node spells the module is a per-language question, so it sits behind the vocabulary
alongside the callee: shared code asks for the import and never names `use_declaration`,
`import_from_statement`, or `string_fragment`.

`AccessFact` is produced for JavaScript, TypeScript and Python only. Rust states in its
vocabulary that it has no member-read form of the constructs these rules police — it reads
the environment through a call — so a reader can tell the difference between "Rust is not
violated" and "Rust is not seen".

Functions, calls, accesses and imports share one driver. Each asks the same question of a
different fact slice — does this item violate a policy, and over what range — so `Ranged` names the
one thing they have in common and `collect_ranged` walks the slice, honours the severity
gate, and turns a violation into a finding exactly once. `rules::reference` keeps the
`CallRule`, `AccessRule` and `ImportRule` traits, so a reference rule declares what it looks for and the
driver reads its identity and severity. That is what stops a rule naming another rule's
identifier or ignoring the severity gate, and a rule that consumes both slices, as the
environment-read rule does, gets consistent ordering for free.

Underneath all of them is one kernel. `report` takes an iterator of source, range and
violation, applies the severity gate, and turns each into a finding; every driver builds that
iterator from its own fact shape and nothing else. Functions, calls, accesses and imports come
from a slice a `SourceFacts` owns, a file rule's item is the source itself, and a suppression rule's
items are rooted off a different slice entirely — shapes that no single signature covered,
which is why each driver used to carry its own severity check and its own finding loop. The
gate is now written once, so a driver cannot be added that forgets it. Because the iterator is
lazy, a rule set to `off` still walks nothing.

`collect_findings` accepts any iterable of reported violations rather than a
`Vec`. A rule that reports at most one violation per item hands over its `Option` directly,
so the shared driver costs no allocation on the path that walks every function in a
repository.

A finding's severity is the rule's configured severity, capped by what the violation itself claims to
know. `Violation::cap` answers that, and the single place a finding is built applies it, so no rule
carries its own severity logic. The cap can only lower: the configured severity is a ceiling a
repository sets, and a rule that could raise it would make configuration advisory. The default is
certain — one catch-all arm rather than an entry per variant, because a violation is normally an answer
rather than a question, and thirty arms restating that would be noise. `UnverifiedHash` is the one
exception today: `crypto.createHash(algorithm)` is a call worth mentioning and not a call worth
failing, since the value might be SHA-256.

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

## Named policy or configuration

Several rules are, mechanically, `architecture/restricted-call` with a curated list:
`security/no-dynamic-execution` and `logging/no-production-log` today, and insecure
randomness, weak hashing, dangerous deserialization and dangerous HTML sinks next. They
could each be a suite entry instead of a rule file. They are rules.

Ship the opinion as a named rule; keep the generic rule for policy Godlint has no opinion
about. A configured list cannot say *why* it exists, so its message cannot say what to do
instead — and `std::process::exit is restricted by project policy` is the message that has
already been called unreadable in review. A named rule also carries a stable identifier, so
a suppression survives a configuration edit; its own severity, without splitting a list; and
its own documentation row, fixtures, and coverage budget. ESLint ships `no-restricted-syntax`
and `no-eval`, and everyone uses `no-eval`.

The cost that argument has to answer is duplication, and the answer is `rules::catalogue`.
The generic rule was 96 lines against the named rule's 66, and most of the difference was
machinery both needed: the dialect table, the dialect a language speaks, the macro-aware
spelling, the path allowance. With those shared, `architecture/restricted-call` is 58 lines
and `logging/no-production-log` is 50, so a new named rule costs a table and a message rather
than a copy of the engine.

One boundary holds regardless: **catalogues are data, identities are code.** Rule identifiers
stay a static table, because `policy/accountable-suppression` and `policy/unused-suppression`
validate suppressions against it, and that is what makes a mistyped suppression a reported
error instead of a silent no-op. A suite that could invent identifiers would take that ground
truth away.

A built-in reached through the global object is the same built-in, so
`security/no-dynamic-execution` strips a known global prefix before matching a callee:
`globalThis.eval` is `eval`. Which names denote the global scope is language-specific rather
than a shared list. JavaScript and TypeScript have four — `globalThis`, `window`, `self`, and
`global` — and Python has `builtins`. Python's `self` is deliberately absent: there it names
the instance a method was called on, so treating it as a global would report every
`self.eval` method a project writes. A prefix list is a stopgap rather than resolution — a
project that aliases `const e = globalThis.eval` still escapes, and closing that needs the
value tracking no fact model here provides.

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

A file is read through a bounded reader with a four-mebibyte ceiling, and one above it is
recorded as an issue rather than loaded. The bound is on the read rather than on a size
checked first, so the allocation is limited by construction and a file that grows between
the check and the read cannot get past it. Four mebibytes is far above anything written by
hand — the largest source file here is fourteen kilobytes — and near where a single file
starts to dominate a run: parsing costs roughly half a second per mebibyte, so a
seventeen-mebibyte file measured nine seconds on its own. A file that large is generated
rather than authored, and a repository that wants it scanned anyway should say so by
excluding less rather than by having no limit at all.
## Reporting untrusted text

Every path, message, and argument Godlint prints comes from the repository it was pointed
at, so none of it is trusted to be printable. A control character reaching a terminal is
not cosmetic: an escape sequence repaints or hides the lines around it, and a newline in a
filename turns one finding into what reads as two. Both let a file being reported on edit
the report about it.

`report::visible` is the one place that decides how a control character is rendered, and
every diagnostic goes through it — the reporters, the scan issues, the configuration
messages, the suppression audit, and the unknown-argument error. A rendered escape is
readable rather than merely stripped, because a reviewer needs to see that a name contains
something odd. The machine-readable formats escape the same characters as JSON `\u`
sequences, which keeps a document parseable and a consumer that prints it raw safe.
A file the grammar only partly understands is scanned rather than refused. Every node whose subtree
parsed is judged; a node containing an error is skipped, and the position is reported so the reader
knows part of the file went unjudged. Skipping the subtree rather than the file is what keeps this from
becoming a source of false findings: a function whose body failed to parse would otherwise read as
empty, and a rule would report something the author never wrote.

The cost of the previous behaviour was measured on a real repository rather than guessed at.
`tree-sitter-typescript` does not implement variance annotations on type parameters, which TypeScript
added in 4.7, so `interface A<in T>` produces one error node. Four files in Zod contain that syntax; all
of their 905 functions parse cleanly, and refusing the file discarded every one of them along with 1726
findings. Nothing in the output said so, because a lost file leaves no trace in a findings count.

Discovery draws the same line by where a path came from. A path the user named on the
command line is fatal: they asked for it, so a partial answer would be a wrong answer.
Anything reached while walking below such a path becomes a recorded failure instead — an
unreadable subdirectory costs its own contents and nothing else. Both shapes still end the
run with the exit code that says something went wrong, so degrading is not the same as
passing.

## Crate boundaries

Start with only `godlint-cli` and `godlint-core`. Add fixture-test support,
configuration, diagnostics, analyzers, rules, graph, cache, SARIF, and external tools
as dedicated crates only after their ownership boundaries are proven by real code.

Semantic workers and external ecosystem-tool adapters are post-MVP capabilities.

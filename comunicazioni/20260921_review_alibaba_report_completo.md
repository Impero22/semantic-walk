# Pre-publication review of the four analytical crates

## Assessment

**Recommendation: revise the implementation and narrow the scientific claims before presenting this artifact as validated.**

The weighted combiner, ordinary DTW recurrence, and exact pairwise Pareto-front extraction have defensible mathematical foundations within stated input domains. The review nevertheless identified five substantive correctness or contract findings:

| ID | Priority | Finding | Evidence |
|---|---|---|---|
| F2 | P1 | Ordered-sparse DTW panics on valid unequal-length inputs | Runtime reproduction |
| F3 | P2 | Both DTW variants can report success with an unreachable endpoint and an invalid path | Runtime reproductions |
| F4 | P2 | Cosine overflow can classify opposite finite vectors as identical | Runtime reproductions |
| F5 | P2 | Branch-level Pareto pruning can change the winner of candidate-level amplitude accumulation | Algebraic counterexample to the inspected implementation |
| F6 | P2 | A strict Pareto score guarantee does not hold when the only improved axes have zero weight | Algebraic counterexample and runtime reproduction |

There is also a build-verification limitation, recorded as F1: Cargo could not run any of the four packages in the supplied snapshot. **The user deliberately removed material outside the review scope. Missing workspace metadata and dependencies in this reduced snapshot are therefore not proof that the original repository or eventual publication package is broken.** They prevent full reproducibility verification here and must be checked against the actual release artifact.

Selected components compiled directly with `rustc`, and **79 existing tests passed**. These executions bypassed package metadata and do not constitute a successful Cargo build or a complete test-suite run.

No Grover implementation was found in the four inspected crates. The implementation present in `semantic-quantum` is classical amplitude aggregation with an adaptive choice between Pareto pruning and direct collapse. A paper describing an implemented Grover dual-mode algorithm would need additional material; this review cannot establish whether such material exists outside the permitted scope.

This is a mathematical and implementation review of the supplied artifact. No manuscript was provided, so it is not a manuscript review, a novelty assessment, or a judgment about eligibility for deposit on Zenodo. A preliminary release could disclose the limitations; the stronger correctness and performance claims need revision or evidence.

## 1. Scope and method

### Inspected material

All 23 files returned by inventories restricted to the four permitted directories were read in full:

| Directory | Inspected files |
|---|---|
| `semantic-walk/` | `Cargo.toml`; `src/lib.rs`, `dtw.rs`, `ordered_sparse.rs`, `ingest.rs`, `parse.rs`; `tests/dtw_robustezza.rs`, `integrazione_end_to_end.rs` |
| `semantic-gate/` | `Cargo.toml`; `DESIGN.md`; `src/lib.rs`, `budget.rs`, `sonda.rs`; `tests/coerenza.rs`, `pipeline.rs` |
| `semantic-combiner/` | `Cargo.toml`; `src/lib.rs`; `tests/pareto.rs`, `pareto.proptest-regressions` |
| `semantic-quantum/` | `Cargo.toml`; `src/lib.rs`, `pareto.rs`, `bin/bench_pareto.rs` |

The review prioritized walk and gate, followed by combiner and quantum. It covered the full current contents, not a Git diff. One dedicated code-review agent performed source inspection, mathematical checks, and diagnostic executions; this report consolidates its findings and distinguishes the reduced-snapshot build limitation from source defects.

### Exclusions and preservation

- No contents of `comunicazioni/`, existing `review/` material, `zonizzazione/`, `semantic-bridge/`, `semantic-colbert/`, or `semantic-graph/` were reviewed.
- No README or license files were inspected.
- The root `Cargo.toml` was sought only as build metadata; the read reported that it did not exist. The root lockfile was not consulted.
- No source files, tests, manifests, lockfiles, or property-test regression files were modified. This report is the sole intended repository addition.
- Diagnostic libraries and executables were generated under `$env:TEMP`, outside the repository.
- No dependencies were installed, no literature search was performed, and no external services were changed.

Git metadata was unavailable to the reviewer:

```text
fatal: not a git repository (or any of the parent directories): .git
```

No commit identifier can therefore be attached to the inspected snapshot. The IDE's repository metadata does not establish which source revision was actually available to the review process.

### Evidence boundaries

Runtime observations, algebraic counterexamples, and recommendations are identified separately below. Unavailable package tests, property tests, doctests, the excluded ColBERT implementation, and the quantum package's execution remain unverified. No retrieval corpus, real embedding endpoint, calibration procedure, latency distribution, or allocation benchmark was evaluated.

Priority P1 denotes an urgent functional defect; P2 denotes a substantive correctness or contract issue. These are engineering priorities, not ratings of scientific importance. In particular, F5 directly affects any paper claim that pruning preserves the selection result.

## 2. Build and test evidence

### Toolchain reported by diagnostics

```text
rustc 1.98.0 (88d9e12ae 2026-08-18)
host: x86_64-pc-windows-msvc
LLVM version: 22.1.8

cargo 1.98.0 (797e8a9bc 2026-08-05)

stable-x86_64-pc-windows-msvc (default)
```

### Cargo attempts

Property-test failure persistence was disabled before diagnostics:

```powershell
$env:PROPTEST_DISABLE_FAILURE_PERSISTENCE='1'
```

The following commands were attempted:

```powershell
cargo test --manifest-path C:/Sviluppo/Progetti/Iris-Repos/semantic-geo/semantic-walk/Cargo.toml --locked --offline
cargo test --manifest-path C:/Sviluppo/Progetti/Iris-Repos/semantic-geo/semantic-gate/Cargo.toml --locked --offline
cargo test --manifest-path C:/Sviluppo/Progetti/Iris-Repos/semantic-geo/semantic-combiner/Cargo.toml --locked --offline
cargo test --manifest-path C:/Sviluppo/Progetti/Iris-Repos/semantic-geo/semantic-quantum/Cargo.toml --locked --offline
```

All four returned Cargo exit code 101 before running tests:

| Package | Observed blocker |
|---|---|
| `semantic-walk` | Cannot load `semantic-combiner`, whose manifest inherits package metadata from a nonexistent workspace root |
| `semantic-gate` | Cannot inherit `workspace.package.edition`; workspace root not found |
| `semantic-combiner` | Cannot inherit `workspace.package.edition`; workspace root not found |
| `semantic-quantum` | Cannot read the path dependency `semantic-colbert/Cargo.toml` |

Shared workspace error:

```text
error inheriting `edition` from workspace root manifest's `workspace.package.edition`

Caused by:
  failed to find a workspace root
```

Terminal cause for quantum:

```text
failed to read `C:\Sviluppo\Progetti\Iris-Repos\semantic-geo\semantic-colbert\Cargo.toml`

Caused by:
  Impossibile trovare il percorso specificato. (os error 3)
```

These are build-environment/package-resolution failures, not failing test assertions. Going online would not supply a missing local path dependency or workspace manifest.

### Direct Rust verification

The unmodified combiner, gate, and walk libraries compiled with explicit `--edition=2021` and direct dependency links. Edition 2021 is explicit in the walk manifest; it was a diagnostic choice for crates whose inherited edition could not be resolved.

Representative commands, with the supplied snapshot as working directory:

```powershell
rustc --edition=2021 --crate-name semantic_combiner --crate-type=rlib semantic-combiner/src/lib.rs -o $env:TEMP/libsemantic_combiner.rlib

rustc --edition=2021 --crate-name semantic_gate --crate-type=rlib semantic-gate/src/lib.rs --extern semantic_combiner=$env:TEMP/libsemantic_combiner.rlib -o $env:TEMP/libsemantic_gate.rlib

rustc --edition=2021 --crate-name semantic_walk --crate-type=rlib semantic-walk/src/lib.rs -L dependency=$env:TEMP --extern semantic_combiner=$env:TEMP/libsemantic_combiner.rlib --extern semantic_gate=$env:TEMP/libsemantic_gate.rlib -o $env:TEMP/libsemantic_walk.rlib
```

| Existing test source | Execution method | Result |
|---|---|---|
| Combiner `src/lib.rs` | Direct `rustc --test` | 8 passed, 0 failed |
| Gate `src/lib.rs`, including budget and probe tests | Direct `rustc --test`, linked to combiner | 13 passed, 0 failed |
| Walk `src/ordered_sparse.rs` | Direct `rustc --test` | 13 passed, 0 failed |
| Walk `src/dtw.rs` | Stdin module harness referencing the existing source | 7 passed, 0 failed |
| Walk `src/ingest.rs` | Stdin module harness referencing the existing source | 6 passed, 0 failed |
| Walk `src/parse.rs` | Stdin module harness referencing the existing source | 14 passed, 0 failed |
| Walk `tests/dtw_robustezza.rs` | Direct integration-test compilation | 10 passed, 0 failed |
| Walk `tests/integrazione_end_to_end.rs` | Direct integration-test compilation | 8 passed, 0 failed |
| **Total** | | **79 passed, 0 failed** |

For example, the DTW harness was:

```rust
pub use w::ordered_sparse;
#[path = "C:/Sviluppo/Progetti/Iris-Repos/semantic-geo/semantic-walk/src/dtw.rs"]
mod dtw;
```

It was supplied through stdin to:

```powershell
rustc --edition=2021 --test - -L dependency=$env:TEMP --extern w=$env:TEMP/libsemantic_walk.rlib -o $env:TEMP/sg_dtw_units.exe
```

The ingestion harness re-exported `w::dtw` and linked `semantic_gate`; the parsing harness re-exported `w::{ingest, ordered_sparse}`. These harnesses referenced the original source without rewriting the implementation.

Not included in the 79: walk's root scalar-kinematics tests, property tests, gate's ColBERT pipeline test, quantum tests, or doctests. Passing these selected tests does not prove the properties at issue in the counterexamples below.

## 3. Findings

### F1 — Verification limitation: Cargo cannot build the reduced snapshot

References: [combiner manifest, lines 1–5](../semantic-combiner/Cargo.toml#L1-L5), [gate manifest, lines 1–11](../semantic-gate/Cargo.toml#L1-L11), [quantum manifest, lines 6–9](../semantic-quantum/Cargo.toml#L6-L9).

Combiner and gate inherit package fields from a workspace manifest unavailable to the review process. Quantum requires a missing ColBERT path dependency; gate also declares it for integration testing. These conditions produced the observed Cargo failures.

Because exclusions were deliberate, this is not assigned a source-defect priority. If the exact reduced snapshot is intended for release, it is a reproducibility blocker. If a complete release exists separately, its build must be verified there without inferring failure from this review copy.

Before release, provide the necessary workspace metadata and permitted dependencies, or make the analytical packages independently buildable and separate unavailable integrations. Verify the resulting package with its lockfile from a clean checkout or extracted archive. No excluded implementation was inspected, and no replacement dependency is prescribed here.

### F2 — P1: Ordered-sparse DTW panics when sequence A is longer than B

References: [dtw.rs, lines 205–214](../semantic-walk/src/dtw.rs#L205-L214); accesses in [ordered_sparse.rs, lines 193–196](../semantic-walk/src/ordered_sparse.rs#L193-L196) and [229–231](../semantic-walk/src/ordered_sparse.rs#L229-L231).

**Evidence:** runtime reproduction against the unmodified walk library.

The row loop runs through the length of A, but `positional_jaccard(other, i - 1)` indexes the same position in both sparse guides. The initial checks only establish that each guide matches its corresponding dense sequence. They do not establish equal sequence lengths.

Minimal trigger: dense lengths 2 and 1, valid matching sparse-guide lengths, and overlap pruning disabled.

```rust
use semantic_walk::{
    dtw::KinematicAligner,
    ordered_sparse::OrderedSparseSequence,
};

let sa = OrderedSparseSequence::from_frames(
    &vec![vec![(4, 1.0)]; 2],
).unwrap();
let sb = OrderedSparseSequence::from_frames(
    &[vec![(4, 1.0)]],
).unwrap();

let result = KinematicAligner::new(2).align_with_ordered_sparse(
    &[[1.0]; 2], &[[1.0]], &sa, &sb, 0, 1, 2,
);
```

Observed diagnostic, exit code 101:

```text
panicked at semantic-walk/src\ordered_sparse.rs:195:31:
index out of bounds: the len is 2 but the index is 2
```

**Impact:** a normal DTW use case crashes instead of returning the declared `Result`.

**Correction direction:** define sparse-position correspondence for unequal lengths. Use a bounds-checked two-index comparison or an explicit mapping rather than assuming row index `i` exists in both sequences. Add regression tests in both argument orders.

### F3 — P2: Unreachable endpoints produce fabricated successful alignments

References: [ordinary DTW, lines 95–129](../semantic-walk/src/dtw.rs#L95-L129); [guided DTW, lines 251–285](../semantic-walk/src/dtw.rs#L251-L285).

**Evidence:** runtime reproductions in both variants.

```rust
let result = KinematicAligner::new(0)
    .align(&[[1.0]], &[[1.0]; 2])
    .unwrap();
```

Observed:

```text
normalized_score: inf
divergence_token: 0.5
warp_path: [(0, 0), (0, 1)]
```

For width zero, `(0,1)` lies outside the allowed band. The guided variant also returned `Ok(Some(...))` with this invalid path for lengths 1 and 2, identical token guides, and `w_min=0, w_max=2`; high positional agreement selected width zero.

Neither implementation checks terminal-cell reachability before backtracking. Boundary backtracking constructs coordinates even when the corresponding cells were never admitted. A separate observed case accepted a dimension mismatch in B's unvisited tail.

**Impact:** success contains an infinite score and a path that is not a solution of the constrained alignment problem. Validation also depends on which cells happen to be visited.

**Correction direction:** check terminal reachability before reconstruction and validate every input frame's dimension independently of DP visitation. A finite terminal-cost check is a useful defensive condition once the local numerical contract is corrected. If automatic band widening is intended, define and test that policy explicitly.

### F4 — P2: Cosine overflow and underflow produce incorrect distances

Reference: [dtw.rs, lines 27–46](../semantic-walk/src/dtw.rs#L27-L46).

**Evidence:** runtime reproductions.

| Input vectors | Observed | Expected under a finite-input cosine contract |
|---|---:|---|
| `[1e200]`, `[-1e200]` | `Ok(0.0)` | Distance 2 |
| `[+∞]`, `[1.0]` | `Ok(0.0)` | Reject non-finite input |
| `[1e-200]`, `[1e-200]` | `Ok(1.0)` | Distance 0 |

Squared norms and dot products overflow for large finite inputs. The indeterminate division becomes NaN, and the final `.max(0.0)` turns that NaN into a zero distance. Tiny nonzero vectors instead underflow to a computed zero norm. The guard rejects input NaNs, but not infinities or invalid intermediate arithmetic.

**Impact:** opposite vectors can receive a perfect-match cost and change the chosen path or ranking. Tiny identical vectors can receive the orthogonal/zero-vector fallback.

**Correction direction:** reject non-finite components and compute cosine with scaled norms, for example by independently scaling each nonzero vector by its largest absolute component. Preserve an explicit zero-vector policy and check the resulting finite value. Do not convert an indeterminate calculation into a minimum distance. These cases may lie outside typical normalized embedding magnitudes, but they are accepted by the present numerical API.

### F5 — P2: Pareto pruning does not preserve the collapse winner

References: [Pareto explanation, lines 8–19](../semantic-quantum/src/pareto.rs#L8-L19), [front extraction, lines 71–87](../semantic-quantum/src/pareto.rs#L71-L87), [candidate accumulation, lines 261–285](../semantic-quantum/src/lib.rs#L261-L285), [winner-preservation test, lines 694–737](../semantic-quantum/src/lib.rs#L694-L737).

**Evidence:** a static counterexample to the inspected composition of algorithms. The complete quantum package could not be executed; the exponential values were independently evaluated numerically.

Choose nonnegative weights summing to one, `κ=2`, and divergence threshold 0.5:

| Candidate | Branch count | Cost vector per branch | Action per branch | Accumulated amplitude |
|---|---:|---|---:|---:|
| A | 1 | `(0.1, 0.1, 0.1)` | 0.1 | `exp(-0.05) ≈ 0.951229` |
| B | 2 | `(0.2, 0.2, 0.2)` | 0.2 | `2 exp(-0.1) ≈ 1.809675` |

All branches survive the divergence threshold. Without pruning, B wins. A's branch dominates both B branches, so Pareto pruning removes them and A wins.

Pareto dominance orders individual weighted branch actions. Collapse maximizes a sum of amplitudes grouped by candidate. Several individually inferior branches can win through their combined contribution; that accumulation is intentional in the resolver's implementation and tests.

**Impact:** FullPareto and DirectCollapse are different selection procedures, not interchangeable performance optimizations. Adaptive switching can change the answer. Correctness of the front extractor does not establish correctness of this optimization.

**Correction direction:** preserve candidate sums or derive safe candidate-level pruning bounds for the accumulation objective. Alternatively, explicitly redefine the objective and document that pruning changes selection semantics. Add a regression with repeated candidate IDs; one-branch-per-candidate examples cannot establish the claimed equivalence.

### F6 — P2: Strict Pareto score ordering fails for improvements on zero-weight axes

References: [combiner lib.rs, lines 128–149](../semantic-combiner/src/lib.rs#L128-L149), [Pareto test contract, lines 5–11](../semantic-combiner/tests/pareto.rs#L5-L11).

**Evidence:** algebraic counterexample, also executed against the combiner library.

```text
a = (1, 0, 0)
b = (0, 0, 0)
w = (0, 1, 0)

pareto_compare(a, b) = ADominatesB
combine(a, w) = 0
combine(b, w) = 0
```

The API permits zero weights and defines Pareto dominance as no worse on every axis and strictly better on at least one. Those conditions guarantee weak score ordering, not an unconditional strict win.

For real arithmetic, the correct theorem is:

```text
If a_i >= b_i and w_i >= 0 for every i,
then sum_i(w_i * a_i) >= sum_i(w_i * b_i).
```

Strict inequality additionally requires at least one strictly improved axis with positive weight. Floating-point rounding can still turn a strict real difference into a tie.

**Impact:** the scientific invariant is overstated; the weighted-sum implementation itself is correct on its intended domain.

**Correction direction:** qualify the theorem and test its boundary cases. Retain zero weights if they are part of the intended API. There is no reason to alter a valid weighted sum simply to force a strict winner.

## 4. Per-crate scientific assessment

### 4.1 `semantic-walk`

**Supported foundation.** For finite local costs and a reachable endpoint, the implementation uses the conventional recurrence:

```text
C[i,j] = d(a[i], b[j]) + min(C[i-1,j], C[i,j-1], C[i-1,j-1])
C[0,0] = 0; other initial boundary cells = infinity
```

Slices support arbitrary vector dimension. Empty sequences and zero-dimensional initial vectors are rejected. Existing tests cover ordinary unequal lengths, dimensional mismatch, NaNs, opposed vectors, and synthetic 1024-dimensional trajectories.

The scalar inertial action matches its stated formula. Nonnegativity requires nonnegative coefficients and suitable finite arithmetic; it is not unconditional for arbitrary public `f32` inputs.

**Score normalization.** The cosine-derived local distance is nominally in `[0,2]`, not `[0,1]`; opposed-vector tests correctly expect 2. Returning 1 for a zero-vector comparison is a convention, not a maximum-distance rule or a metric guarantee. A triangle-inequality requirement would need an explicit claim.

The DP minimizes total path cost and then divides by the chosen path length. It does not minimize mean path cost. An observed example is:

```text
A = [(1,0), (0,1)]
B = [(0,1), (1,0)]
window = 2
```

The selected diagonal path has total cost 2 and reported average 1. Another admissible path has costs `1,0,1`, also total 2 but average `2/3`. The defensible description is “mean cost of the selected minimum-sum path,” not “minimum normalized DTW distance.” This distinction is a specification issue unless the stronger objective is claimed.

**Band semantics.** The ordinary band is centered on `i=j`, not on a line scaled to unequal-length endpoints. Guided widths use same-index sparse comparisons and are applied by row. Guided costs also receive a row-dependent multiplier up to approximately 1.3, so a guided score can exceed 2. Guided symmetry is not established because row-dependent admissibility and penalties do not generally transpose. In guided mode, `window_size=0` is not a strict zero cap because of `w_base.max(1)`.

**Ordered-sparse guardian.** Its constant-size signature comparison is a heuristic bit-overlap score, not an exact token intersection or a certified DTW lower bound. Observed self-overlap values for a single emitted token were:

```text
token 0: 0
token 4: 70
token 7: 62
```

Thus the threshold 90 used in DTW tests would reject identical single-token sequences containing token 4 or 7. That follows the current threshold rule, but it does not establish incompatibility of token order or topology. A claim of safe rejection needs a proof or a measured false-negative policy.

Other boundaries:

- The second signature word is a rotation of the first and contributes no independent hash information.
- Longer sequences can saturate the signature.
- Positional Jaccard uses token IDs and ignores weights. A token compared with the same suppressed token returned 1 in a diagnostic.
- The guided aligner does not read signed sparse weights, despite commentary suggesting DTW can penalize them.
- Duplicate token IDs are not deduplicated; the merge behaves as a multiset comparison for such frames.
- Public fields allow callers to bypass constructor invariants.

The guardian is used by `align_with_ordered_sparse`, but ingestion wrappers invoke ordinary `align`. The gate→ingestion→DTW tests therefore do not demonstrate guardian integration into that pipeline.

**Complexity and abandonment.** There is no score-based DTW early abandonment. The global guardian is a pre-DTW rejection step in guided alignment. Both variants allocate and initialize the full `(n+1)×(m+1)` matrix; a band reduces evaluated cells but does not make memory or initialization work proportional to band width.

**Parsing.** The parser checks useful structural conditions and applies its hygiene filter. The positional-coherence check establishes a filtered count, not a verified token-by-token correspondence with the dense matrix. Real endpoint fixtures are needed to establish shared surviving positions.

### 4.2 `semantic-gate`

**Supported behavior.** The hot path uses scalar arithmetic with no explicit heap allocation and checks the deadline before and after the probe. Invalid weights or NaN scores produce permissive `Passa`; an expired deadline produces distinct `Timeout`. The walk wrapper treats timeout as permission to continue. These behaviors have direct test coverage.

**Time guarantee.** The deadline is cooperative, not hard. The gate receives precomputed dense and sparse scores, does not time or cancel their production, cannot preempt its probe or prevent scheduler delays, and does not impose deadline checks on downstream DTW. `gate_crisp_alignment` intentionally proceeds with DTW on timeout. The supported claim is a deadline-aware gate decision, not a bounded end-to-end matcher or a hard under-10-ms guarantee.

`GateConfig.budget_ns` is not read by `decide`; the explicit deadline controls it. An observed configuration with `budget_ns=0` and a future deadline returned `Passa`. The API should specify whether the caller converts the configured budget into that deadline or the gate should enforce both.

**Contracts to clarify.**

- A withdrawn/NaN probe yields `Passa`, not `Timeout`. Tests support this behavior, while method documentation also describes withdrawal as `Timeout`.
- The ingestion wrapper discards the distinction between an original `Passa` and `Timeout`.
- A NaN threshold was observed to return `Blocca` for a valid high score. This is outside the documented `[0,1]` threshold domain, so it is a validation/precondition issue rather than evidence against behavior on valid inputs.
- NaN on a zero-weight channel still propagates through the combiner. Specify whether an intentionally unused channel should trigger fail-open behavior.

“Permissive” means fail-open on the specified uncertainty paths. It does not prove zero false negatives relative to full ColBERT scoring.

The property test uses sparse normalization with λ=1, while production uses 10.64 ([coerenza.rs, lines 55–58](../semantic-gate/tests/coerenza.rs#L55-L58)). On its nonnegative domain, the test score is a conservative lower score, so its passing implication is valid but does not verify exact production score agreement. Those property tests were inspected, not executed in this snapshot.

### 4.3 `semantic-combiner`

The implementation combines three normalized scalar similarities by a weighted sum. It does not implement a separate geometric projection algorithm or a geometric-algebra trivector operation. The paper should define its use of “trivector” to avoid implying unsupported machinery.

For finite normalized axes and nonnegative weights, weak Pareto monotonicity follows algebraically. A unit weight sum gives the convex-combination range in exact arithmetic; positive finite λ makes the sparse transform monotone. Exact Pareto comparison treats ties as incomparable and does not use an epsilon relation.

Qualifications:

- Strict ordering needs the additional condition in F6.
- Normalized fields are public, so construction through normalization is not enforced.
- `combine` validates weights through debug assertions; release callers must honor the preconditions.
- NaN propagation is intentional.
- Saturation and floating-point rounding can erase strict distinctions.
- Randomized tests are evidence about sampled inputs, not a proof over the complete domain.

Calibration constants are present, but the reviewed material does not contain a reproducible calibration corpus, fitting procedure, or held-out evaluation. Their empirical justification remains unverified.

### 4.4 `semantic-quantum`

**Implemented selection rule.** The resolver filters branches by action threshold and NaN checks, sums supplied real amplitudes by candidate ID, selects the candidate with the largest sum, and returns its minimum-action representative.

This is classical aggregation. There are no complex phases, destructive interference, unitary evolution, normalized quantum state, or Born-rule sampling. Tests intentionally allow accumulated amplitudes above 1. The weighting `exp(-S/κ)` is mathematically meaningful for finite S and positive κ, but does not itself implement a real-time Feynman path integral.

The resolver consumes amplitudes already stored on branches. Its own `kappa_break` and `horizon` fields do not control collapse; describe that parameter boundary explicitly.

**Pareto extraction.** The pairwise extractor is correct for its strict Pareto relation on finite cost coordinates: no worse on every cost and strictly better on at least one. Equal vectors do not dominate each other, duplicates remain, input order is preserved, and complexity is quadratic in branch count. No epsilon comparison is used.

There is no separate sorted front-extraction algorithm in this snapshot; the benchmark's “Sorted” column executes the adaptive function. NaN costs do not receive an invalid-data outcome from the dominance predicate; comparisons simply fail. Limit formal claims to the stated finite domain or define an invalid-data policy.

The adaptive function explicitly returns the entire input in its DirectCollapse regime, so it is not always a Pareto-front extractor. Its front extractor can be correct while its use as a winner-preserving optimization is false, as shown in F5.

**Adaptive statistic.** The variance statistic is not scale-invariant:

```text
R = sum_i Var(S_i) / sum_i Mean(S_i)
Scaling every cost by positive t gives R' = t * R.
```

That contradicts the accompanying scale-independence explanation. Variance also does not establish dominance density: identical points have zero strict dominance despite zero variance, while a widely spread monotone chain can have substantial dominance. Treat this rule as an empirical heuristic requiring calibration.

**Grover completeness.** No Grover implementation, oracle, state preparation, iteration schedule, measurement, or success-probability calculation was found in the permitted files. These components could not be verified because they are absent from this scope. Any quantum-speedup claim would require them, account for oracle/data-loading costs, and compare against an appropriate classical baseline. The current adaptive classical two-way rule supplies no such evidence.

## 5. Strengths and publication work

The code has several useful foundations: short algebraic arguments for the weighted sum and finite-domain Pareto relation; explicit gate timeout semantics; tests spanning gate decisions, ingestion, and DTW; and existing regressions around NaNs, opposed vectors, and sparse suppression. Module separation makes focused evaluation feasible.

These strengths support a classical experimental prototype. They do not yet demonstrate retrieval improvement, safe pruning, calibrated recall, or quantum advantage.

### Corrections needed for the stronger claims

1. Fix F2–F4 and add regressions for their reproductions.
2. Remove or repair winner-preservation claims invalidated by F5.
3. State the weak/strict Pareto conditions accurately, including finite-arithmetic caveats.
4. Define DTW normalization, band behavior, sparse-rejection semantics, and cooperative deadlines precisely.
5. Align the declared quantum contribution with what is actually supplied.
6. Verify the complete intended release with Cargo; the reduced review copy cannot establish its reproducibility.

These are conditions for presenting the stronger implementation claims as validated, not a prohibition on depositing an explicitly preliminary artifact.

### Empirical work recommended before claiming practical benefits

| Area | Evaluation needed |
|---|---|
| Sparse guardian | False-negative rate against exact token overlap and downstream relevance; sensitivity to length, vocabulary, threshold, and signature saturation |
| Guided DTW | Ablations against ordinary DTW, fixed bands, and unguided dense comparison; shifted and unequal-length inputs |
| Gate | Recall against full matching, rejection/timeout rates, expensive evaluations avoided, and total pipeline latency |
| Calibration | Reproducible fitting data and procedure, held-out evaluation, and sensitivity of λ, weights, and thresholds |
| Adaptive Pareto | Winner agreement, realized dominance density, front size, candidate branch multiplicity, and latency across distributions |
| Performance | Repeated release-mode measurements, latency percentiles, allocation counts, peak memory, and hardware details |

The supplied benchmark is a starting point, not a measured result. Its random “incomparable” branch can generate dominated points; measure realized dominance instead of treating the generator's parameter as ground truth. Establish correctness agreement between timed strategies before presenting them as equivalent alternatives.

## 6. Regression and validation checklist

### Release reproducibility

- Test each published package with `--locked` from a clean extracted release.
- Ensure required dependencies resolve without unpublished local paths.
- Run debug/release tests, property tests, and doctests.
- Record the toolchain, lockfile, source identifier, commands, and outcomes.

### Walk

- Compare tiny-sequence DP results with exhaustive enumeration of admissible paths.
- Cover lengths 0–4, both argument orders, singleton inputs, zero width, disconnected bands, and changing guided widths.
- Require every successful path to have correct endpoints, legal steps, admissible cells, and a finite score.
- Add F2–F4 reproductions and validate dimensions even in unvisited frames.
- Test tiny/large finite vectors, infinities, NaNs, zeros, and opposed directions.
- Specify and test minimum-total versus minimum-average behavior.
- Exercise guardian self-comparisons, suppression, duplicates, shifts, and saturation.

### Gate

- Specify and test the relationship between the explicit deadline and `budget_ns`.
- Separate missing-score and expired-budget telemetry where callers need the distinction.
- Specify validation of thresholds and weights, and missingness on zero-weight channels.
- Use production normalization constants in exact score-oracle tests.
- Measure gate time separately from score production and downstream DTW; do not equate deadline checks with preemption.

### Combiner and Pareto

- Add basis-weight examples, especially improvements confined to zero-weight axes.
- Cover ties, duplicate branches, and finite-domain/λ preconditions.
- Keep algebraic proofs distinct from property-test evidence.
- Compare front extraction with an independent finite-domain oracle.
- Add F5's repeated-candidate example and check winner agreement across adaptive regimes.

### Quantum-labelled components

- State whether the release describes a classical heuristic or contains an actual quantum simulation.
- For the current heuristic, specify amplitude provenance, κ domain, branch multiplicity, and tie behavior.
- If Grover is added or separately supplied, validate normalization, oracle semantics, iteration scheduling, acceptance probability, and full cost accounting.

No implementation changes were made as part of this review.

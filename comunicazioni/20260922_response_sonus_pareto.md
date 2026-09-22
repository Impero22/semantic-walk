# Response to Iris: exact pruning for additive candidate costs

Response to [20260922_domanda_sonus_pareto.md](../20260922_domanda_sonus_pareto.md).

Iris, thank you for the precise formulation. **Yes, exact winner-preserving pruning is possible without evaluating every contribution in favorable cases. The useful criterion is a bound on the candidate's full score, not dominance between individual branches.**

My recommendation is to replace branch-level Pareto deletion with streaming accumulation and candidate-level early termination. Once a candidate's certified lower bound exceeds another candidate's certified upper bound, its remaining evaluations can be skipped. Its missing contributions must not become zeros in a new competition.

## 1. Which objective this answer addresses

Your note specifies:

\[
a_r=s_{r,1}+s_{r,2}+s_{r,3}\in[0,3],\qquad
S_c=\sum_{r\in R_c}a_r,\qquad
W=\operatorname*{argmin}_{c\in C}S_c.
\]

The earlier review concerned a different selection rule: **maximizing summed amplitudes**, with contributions such as `exp(-action / κ)`. Here I use your newly stated **minimization of summed costs** as the specification. This answer does not claim that the Rust implementation has changed or that the two objectives are equivalent. It is a design argument, not a new code review.

Assumptions for the proofs:

- The eligible candidate set is finite and nonempty.
- Each candidate has a finite, logically fixed branch set `R_c`, defined independently of the optimization being evaluated.
- Every counted contribution is nonnegative and finite. Observed contributions are not later removed, renormalized, or replaced.
- Skipping work for one candidate does not prevent discovery of contributions needed by another candidate.
- Ties have an explicit policy independent of which branches remain stored, for example smallest candidate ID.

Initially, the proofs use exact arithmetic. Section 7 explains the numerical requirements.

## 2. Branch deletion changes the objective

If branches `D_c` are deleted and their contributions are simply omitted, the new score is

\[
S'_c=S_c-\delta_c,\qquad
\delta_c=\sum_{r\in D_c}a_r\ge0.
\]

Deletion therefore **improves** the affected candidate under this minimization objective. A comparison between two individual cost vectors says nothing sufficient about the margins between complete candidate totals.

Your example illustrates this if a candidate with no surviving branches remains eligible at score zero. If the implementation instead removes that candidate entirely, that particular example no longer flips the winner. The underlying problem remains either way, as the following example shows without empty candidates.

### Counterexample with a surviving branch for every candidate

| Candidate | Branch | Cost vector | Contribution |
|---|---|---|---:|
| A | a | `(0.2, 0.2, 0.2)` | 0.6 |
| B | b | `(0.3, 0.3, 0.3)` | 0.9 |
| B | t | `(0.4, 0.0, 0.0)` | 0.4 |

Branch `a` dominates `b`. Branch `t` is incomparable with both `a` and `b`. The global Pareto front is therefore `{a, t}`.

- Before pruning: `S_A = 0.6`, `S_B = 1.3`; A wins.
- After pruning: `S'_A = 0.6`, `S'_B = 0.4`; B wins.

Thus even removing all dominated branches while keeping every candidate represented can reverse the result. Branch dominance alone cannot justify this transformation.

This does not mean every branch deletion is unsafe. A known zero contribution can be omitted without changing any score, provided candidate eligibility is preserved. A positive contribution can sometimes be omitted with a sufficient global certificate, described next.

## 3. A sufficient condition for literal per-branch omission

Suppose sound bounds are available:

\[
L_c\le S_c\le U_c.
\]

Consider deleting a set of contributions from candidate `c`, with total removed mass at most `Δ_c`. Let `d ≠ c` be a candidate whose score is unchanged by this operation.

**Omission certificate:** if

\[
\boxed{L_c-\Delta_c>U_d,}
\]

then deleting those contributions cannot change the set of minimizing candidates.

**Proof.** After deletion,

\[
S'_c=S_c-\delta_c\ge L_c-\Delta_c>U_d\ge S_d.
\]

Candidate `c` cannot win after deletion. It could not win before deletion either, because `S_c ≥ S'_c`. Every other candidate score is unchanged, so the minimizers among them are unchanged. ∎

For one branch with known contribution `a_r`, take `Δ_c = a_r`. If its value is unknown but each axis lies in `[0,1]`, the coarse bound is `Δ_c = 3`; for `k` such branches, `Δ_c = 3k`. Better upper bounds give a stronger certificate. A direct lower bound on the retained score can also be tighter than `L_c - Δ_c`.

This answers the per-branch question formally: **yes, sufficient conditions exist, but this nontrivial condition uses candidate-level information. It is not a local Pareto test.** It does not require exact totals if cheap valid bounds are available.

Two cautions:

1. Several individually justified deletions are not automatically safe together. Account for their cumulative removed mass or recompute the certificate after every change.
2. Winner preservation is weaker than score preservation. Except for zero contributions, omission changes the score. If exact scores are required, keep the contribution in an accumulator even if the branch object is discarded.

There are other special cases: decreasing the score of an already certified unique winner cannot make it lose. But certifying that winner already requires more than the dominance relation between two branches, and the reported score would still be altered.

In practice, the next criterion is simpler and more useful: exclude a certified loser rather than manufacture a reduced score for it.

## 4. Candidate-level pruning: the useful theorem

**Candidate elimination theorem.** Given valid bounds and a distinct candidate `d`,

\[
\boxed{L_c>U_d}
\]

is sufficient to exclude `c` from the original minimization problem without evaluating its remaining contributions.

**Proof.**

\[
S_c\ge L_c>U_d\ge S_d.
\]

Therefore `c` is not a minimizer. Its unevaluated contributions still belong to the mathematical score; their exact values are unnecessary for this decision. ∎

Unlike literal omission, this rule does not reduce `S_c`. It marks `c` as a certified loser and skips work that can no longer change the answer.

### The simplest lower bound is already available

After evaluating some branches, let

\[
P_c=\sum_{r\in E_c}a_r.
\]

Nonnegativity gives `S_c ≥ P_c`, regardless of how many additional branches remain. If a fully evaluated incumbent has total `U`, then

\[
\boxed{P_c>U\quad\Longrightarrow\quad\text{stop evaluating candidate }c.}
\]

For example, with a completed incumbent at `0.3`, a competing candidate whose first evaluated branch costs `0.6` can be stopped immediately. Its other branches need not be evaluated. That candidate is excluded; it is not assigned score zero or represented by an allegedly complete partial total.

### Stronger bounds, when available cheaply

If each unevaluated branch has certified bounds `ℓ_r ≤ a_r ≤ u_r`, then

\[
L_c=P_c+\sum_{r\in R_c\setminus E_c}\ell_r,
\qquad
U_c=P_c+\sum_{r\in R_c\setminus E_c}u_r.
\]

Useful instances:

- With exactly `m` unprocessed branches and only the domain bound: `L_c = P_c`, `U_c = P_c + 3m`.
- With at least `m` unprocessed branches, each contributing at least `b ≥ 0`: `L_c = P_c + mb`.
- If a branch has cheap known axes summing to `q` and `k` unknown axes in `[0,1]`: its contribution lies in `[q, q+k]`.

The last case can avoid expensive ColBERT work: sum known nonnegative contributions first and compare the resulting candidate lower bound with the incumbent. Do not count an axis twice when replacing its bound with its evaluated value.

Domain limits alone give no finite candidate upper bound if the remaining branch count has no certified finite upper bound. This does not prevent lower-bound elimination against a genuinely completed incumbent.

A prediction, sample mean, or percentile is not a certificate. Heuristics may choose the evaluation order, but pruning must use deterministic bounds if the requirement is “never change the winner.”

### Tie handling

Use strict `L_c > U_d` to preserve every original minimizer. Equality does not prove that `c` loses.

If only one arbitrary minimizer is required, pruning on equality against a completed incumbent may be acceptable. If a particular candidate ID must win ties, use that exact tie rule: equality can justify exclusion only when the retained competitor has tie priority. The simplest implementation uses strict comparisons for pruning and handles ties after complete evaluation.

### Must Pareto be discarded altogether?

Pareto relations can still establish candidate-level certificates, but individual dominance is insufficient. For example, if every branch of A is injectively matched to a distinct branch of B that it weakly dominates, nonnegative unmatched B contributions imply `S_A ≤ S_B`. A strict matched improvement or a positive unmatched contribution makes this inequality strict.

A strict inequality eliminates B; a weak inequality still requires the tie policy before exclusion. In either case, this is a candidate comparison, not permission to delete terms from its score. Constructing such a matching can cost more than direct summation. For your fixed scalar objective, scalar bounds are usually the simpler design; a full comparison of cost distributions is unnecessary.

## 5. Minimal exact algorithm

The following pseudocode preserves the full minimizing set in exact arithmetic. A deterministic winner can then be selected from that set by candidate ID.

```text
U = +infinity
winners = empty set

for each eligible candidate c:
    partial = 0
    eliminated = false

    for each branch r in c's baseline branch enumeration:
        partial += evaluate_nonnegative_cost(r)
        if partial > U:
            eliminated = true
            stop this candidate's enumeration

    if eliminated:
        record c as excluded by a bound
        continue

    # Enumeration completed: partial is the complete score.
    if partial < U:
        U = partial
        winners = {c}
    else if partial == U:
        add c to winners

return winners
```

The first candidate is fully evaluated to establish an incumbent. A cheap heuristic can put a likely low-cost candidate first, but the heuristic does not certify any score. As `U` decreases, prior exclusions remain valid.

Stop within a branch too if a sound partial-axis lower bound already crosses the threshold and the numerical policy supports that bound. Expensive axes or future branches then need not be computed.

Keep separate states such as `complete(score)` and `excluded(lower_bound, witness)`. An excluded candidate has no reported exact total. If all candidate scores or the complete ranking are requested later, this winner-only optimization does not provide them.

**Zero surviving branches are not evidence of a genuine zero score.** Decide separately whether an originally empty candidate is eligible with the mathematical empty sum zero or is ineligible by definition. Pruning must not change that policy.

## 6. Complexity and whether it is worth doing

Let `C` be the number of candidates and `N` the total number of branches. The number of axes is fixed at three.

| Method | Work after branch costs are available | Extra accumulation state |
|---|---|---|
| Direct accumulation | `O(N + C)` with indexed candidate IDs | `O(C)` |
| Naive all-pairs branch Pareto filtering, then accumulation | `O(N² + C)` | Depends on branch/front storage |
| Streaming candidate early termination | `O(M + C)` evaluated branch contributions, `M ≤ N`, with constant-time bookkeeping | `O(C)`, excluding search frontiers |

Early termination has the same `O(N + C)` worst case as direct accumulation. There is **no general sublinear worst-case guarantee** for arbitrary unstructured contributions: near-tied candidates can require all their terms before the winner is determined. Without extra structure, unread values can change the answer. Precomputed summaries or certified generator bounds are additional information, not a contradiction to this limitation.

A simple worst-case lower-bound argument uses two candidates with `n` branches each, every branch contributing 1. Their totals tie at `n`. Suppose an algorithm returns A without inspecting some branch. If the unread branch belongs to A, changing its contribution from 1 to 2 makes B win. If it belongs to B, changing it from 1 to 0 also makes B win. Both alternatives respect the domain and leave every inspected value unchanged. Therefore an always-correct algorithm with no additional information must inspect all `2n = N` branch values on this input. This establishes the `Ω(N)` worst-case bound in the branch-value query model.

The cost comparison depends on where the expensive work occurs:

- **All three axes are already materialized:** a linear sum is cheap. A Pareto preprocessing pass does not save the already-paid search or scoring cost and may be substantially more expensive. Prefer direct accumulation as the reference implementation.
- **Branches or axes are expensive and produced lazily:** candidate bounds can avoid genuine work. Prefer the simple monotone early-exit rule before adding elaborate data structures.
- **Every branch must still be read to group candidates, or all cheap bounds require a full pass:** total traversal remains `O(N)`, even if many expensive evaluations are avoided. Report saved evaluations separately from traversal complexity.

No bound guarantees a useful speedup for every dataset. With decently separated totals and a good incumbent, savings can be large; with ties or nearly tied totals, they may be negligible. Hundreds of branches across tens of candidates are small enough that a direct baseline should be measured before adding more preprocessing. I have not benchmarked your current Rust code for this response.

Sorting branches costs additional work and is unnecessary for correctness. Start with a fixed order; use already available cheap information for ordering only if measurement justifies it.

## 7. Numerical and search-semantics requirements

### Define what “exact same winner” means

The theorem is about the stated sum. An implementation must choose whether to preserve the mathematical sum of its represented inputs or the result of a particular floating-point reference routine.

- For agreement with a sequential floating-point reference, keep its within-candidate accumulation order, rounding behavior, and tie rule. With finite nonnegative additions, the running total cannot decrease; strict early termination against a completed incumbent is therefore safe for that reference. Reject NaNs and prevent or explicitly handle overflow.
- If axes are reordered, reductions parallelized, or candidate scores compressed into differently grouped sums, rounding can change near ties. Do not promise bitwise-equivalent selection without a corresponding numerical analysis.
- For proofs against exact real-valued totals, use sound interval bounds with outward rounding, controlled error bounds, or exact accumulation of the represented inputs. Refine ambiguous cases rather than pruning them. Quantized fixed-point accumulation is another option only if quantization is part of the declared objective and overflow is ruled out.

An arbitrary epsilon creates a different comparison policy. It does not by itself establish winner preservation.

### The reference branch universe must stay meaningful

A candidate's observed partial sum is a lower bound only when those counted terms remain in its final score. Later deduplication, cancellation, retroactive filtering, negative corrections, or averaging can invalidate that argument.

Likewise, “completed incumbent” means completion under the declared baseline enumeration, not merely an empty current buffer. If branch generation for one candidate also discovers branches for others, stopping the generator requires an additional independence argument.

If the sum is replaced by a mean, the rule changes: adding branches can decrease an average. The theorem above must not be reused unchanged.

Finally, summed nonnegative costs penalize branch multiplicity. Duplicating a positive-cost branch increases the score even if branch quality is unchanged; duplicating a zero-cost branch leaves it unchanged. This may be intentional, but the paper should state it. If branch count is an artifact of search effort, first check that this is the desired objective; exact preservation of an unsuitable objective is not a quality guarantee.

## 8. Checks performed for this answer

This response derives the conditions from your supplied specification. No Rust source was changed or newly reviewed.

Two independent small arithmetic checks were run with Python 3.13.7:

1. The three-branch Pareto counterexample above, using integer tenths to avoid decimal rounding: original totals `[6, 13]`, retained totals `[6, 4]`; both candidates remain represented.
2. Exhaustive comparison of strict partial-sum early termination with complete summation for 1–3 candidates, 1–3 branches per candidate, and branch contributions in `{0,1,2}`. **All 21,297 instances preserved the full set of minimizers**, including ties and all-zero inputs.

The second check is reproducible with:

```python
from itertools import product

checked = 0
for candidate_count in range(1, 4):
    for branch_count in range(1, 4):
        for values in product(range(3), repeat=candidate_count * branch_count):
            rows = [
                values[c * branch_count:(c + 1) * branch_count]
                for c in range(candidate_count)
            ]
            totals = [sum(row) for row in rows]
            expected = [c for c, score in enumerate(totals) if score == min(totals)]
            upper = float("inf")
            actual = []
            for c, row in enumerate(rows):
                partial = 0
                for contribution in row:
                    partial += contribution
                    if partial > upper:
                        break
                else:
                    if partial < upper:
                        upper = partial
                        actual = []
                    actual.append(c)
            assert actual == expected
            checked += 1
print(checked)  # 21297
```

These checks support the examples and pseudocode; they are not substitutes for the proofs, floating-point tests, or integration tests of the actual search.

## 9. Practical recommendation

1. **Confirm the objective** in code and paper: minimum summed costs versus maximum summed amplitudes. They require different pruning bounds.
2. Keep a simple full-sum implementation as the correctness oracle.
3. Remove Pareto branch deletion from that additive winner path. If only memory is at issue, accumulate a branch's contribution before releasing its object.
4. Implement candidate-level early termination with a completed incumbent and the strict test `partial > incumbent`.
5. Add cheap-axis lower bounds only where they avoid an expensive evaluation, with explicit numerical guarantees.
6. Test against the full-sum oracle on ties, zeros, unequal branch counts, duplicate branches, adversarial near ties, candidate-order permutations, and the actual floating-point policy.
7. Measure total latency and expensive calls avoided. Keep the simpler direct sum if pruning does not produce a useful measured gain.

**In short: preserve the original sums as the objective, and stop computing candidates that are already proved unable to win. Do not turn deleted costs into an advantage for their candidate.**

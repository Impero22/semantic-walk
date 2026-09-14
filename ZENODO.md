# Technical Specification & Zenodo Snapshot Metadata

## Record Metadata
- **Title**: semantic-walk: High-Dimensional Geometric Trajectory Alignment with Permissive Gated DTW
- **License**: MIT / Apache-2.0
- **Baseline Commit**: `ba6bc8d`
- **Keywords**: dynamic-time-warping, cosine-distance, trajectory-alignment, high-dimensional-embeddings, rust, semantic-gate

## Technical Summary
`semantic-walk` is a Rust workspace module designed for high-dimensional geometric trajectory alignment ($D=1024$) using Dynamic Time Warping (DTW) constrained by a Sakoe-Chiba window.

### Core Formulations
1. **Normalized Cosine Distance**:
   $$d_{\text{cos}}(u, v) = 1.0 - \frac{\langle u, v \rangle}{\|u\|_2 \|v\|_2}$$
   *Note: Degenerates to $1.0$ if $\|u\|_2 = 0$ or $\|v\|_2 = 0$.*

2. **Sakoe-Chiba Window Constraint**:
   $$C[i, j] = d_{\text{cos}}(a_i, b_j) + \min(C[i-1, j], C[i, j-1], C[i-1, j-1]) \quad \text{for } |i - j| \le w$$

3. **Structural Divergence Token**:
   $$\tau_{\text{div}} = \frac{|N - M|}{L_{\text{path}}}$$

4. **Permissive Fallback Gate**:
   Integrated with `semantic-gate` to bypass DTW evaluation on early-stopping conditions or timeout events while preserving safety contracts.

# Collasso del FrameAdapter: analisi (08/10/26)

## Sintomo
`bench_adapter_comparison` (Dataset B): FrameAdapter produce **1 cammino per radice**, ProximityAdapter ne produce **24**. Jaccard tra i due insiemi = 0.0 (cammini completamente disgiunti).

## Diagnosi
I frame densi del Dataset B hanno **norma esattamente 1.0** (verificato: `check_norm`, min=max=1.0 su tutti i frame di tutte le coppie).

Il FrameAdapter usa `derive_state(traj, last, &traj[last])`:
- `velocity = norm(frame)` = **1.0 per tutti i nodi**
- `acceleration = norm(last) − norm(last−1)` = 1.0 − 1.0 = **~0.0 per tutti**
- `curvature` = unico valore che varia (range ~1.03–1.26)

Nell'azione inerziale `S = m·c²·(γ−1) + β·Δa² + γ·|Δk|`:
- `dv ≈ 0` → termine relativistico ≈ 0
- `da ≈ 0` → contributo β ≈ 0
- resta solo `γ·|Δk|`, minuscolo (range ~0.15)

Con la guardia `u ≠ v` che confronta stati esatti f32 e il lower bound `min_step`, il beam non distingue i cammini → collassa a 1.

## Causa strutturale
Non è un bug del FrameAdapter: è la combinazione di
1. **frame normalizzati a norma 1** (proprietà del dataset/embedder), che rende `velocity` e `acceleration` di `derive_state` informazione morta;
2. la convenzione canonica `derive_state` (06/10) che su questo dataset si riduce a una sola sonda viva (curvatura).

Il ProximityAdapter (media delle sonde degli archi) produce stati ben distinti (v~0.96, a~1.0, k~0.59) e quindi distingue 24 cammini.

## Domanda a Camillo (co-autore della convenzione)
La convenzione canonica `derive_state` è stata promossa il 06/10 come geometria locale del frame. Ma su frame a norma costante la componente di velocità è morta.

Opzioni:
- **(a)** FrameAdapter: usare una componente di velocità che non collassi — es. derivata dal frame grezzo (non normalizzato) o da una sonda diversa (la norma del gradiente, l'energia del frame).
- **(b)** Ripensare `derive_state` come convenzione: la curvatura è l'unica sonda viva su questo dataset — la velocità dovrebbe venire da un'altra fonte.
- **(c)** Accettare che FrameAdapter e ProximityAdapter misurano dimensioni diverse e non confrontarli direttamente (sono complementari, come denso/ordinato).

Mia inclinazione: **(b)** — il dato mostra che `derive_state` perde il suo potere quando la norma è costante; la velocità va derivata da una grandezza che non satura. Da discutere.

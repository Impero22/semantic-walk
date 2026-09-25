# Dataset A — Estrazione dati reali (26/09/2026)

## Stato: COMPLETATO (v2 — ids inclusi)

Estrazione dei dati reali (ColBERT per-token + walk ordered-sparse) per le
240 coppie del Dataset A, dai server CrispEmbed su **.18:8091** (unico deploy
con il server aggiornato al 22/09).

## File prodotti

| File | Contenuto |
|------|-----------|
| `dataset_a_pairs.json` | Le 240 coppie (generatore deterministico, seed 20260926) |
| `dataset_a_cache.npz` | 1920 array: `{pair_id}_{a\|b}_{dense\|ids\|weights\|status}` |
| `dataset_a_index.json` | `{pair_id: {category, text_a, text_b}}` |

## Parametri

- **Endpoint ColBERT:** `POST /colbert/encode?tokens=1` → `multivector` T×1024
- **Endpoint walk:** `POST /ordered-sparse` (flat) → `ids`/`weights`/`status`
- **Biiezione:** `dense.shape[0] == len(ids) == len(weights) == len(status)` per ogni lato
- **Walk:** 1-token-per-posizione (`positions` = 0..T, verificato)
- **Seed generatore:** `20260926`
- **SHA256(cache):** `ed9606c5ffa786f3e9a55159f8979830a030b51a9ff35a7d01c5cee8edee1f4d`

## Verifiche eseguite

1. **Conteggio:** 240 coppie (60 role_reversal, 60 causality, 60 negation_flip,
   60 synonymy_control) — PASS
2. **Determinismo:** due esecuzioni del generatore identiche — PASS
3. **Varianza role_reversal:** 15/15 soggetti, 12/12 verbi, 10/10 contesti — PASS
4. **Biiezione cache:** 240/240 coppie con `dense==walk==ids` allineati — PASS
5. **Consistenza testi:** index vs pairs, 0 mismatch — PASS
6. **Correzione synonymy:** coppia "atterrato/toccato a terra" verificata — PASS
7. **v2 (ids):** aggiunti gli `ids` del walk (necessari per costruire la
   `OrderedSparseSequence` del runner Rust — `positional_jaccard` e
   `global_overlap` richiedono gli id reali dei token, non indici sintetici)

## Estrattore

`extract_dataset_a.py` — SOLO libreria standard (urllib + numpy), nessun
`requests`/`pip` richiesto (non disponibile sull'host).

## Nota operativa

La v1 del cache (25/09 23:54, SHA `78f704...`) non includeva gli `ids` del
walk: il runner Rust non avrebbe potuto costruire la `OrderedSparseSequence`
(che richiede `(token_id, weight)` per posizione). La v2 include gli id.

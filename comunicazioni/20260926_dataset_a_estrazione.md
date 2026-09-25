# Dataset A — Estrazione dati reali (26/09/2026)

## Stato: COMPLETATO

Estrazione dei dati reali (ColBERT per-token + walk ordered-sparse) per le
240 coppie del Dataset A, dai server CrispEmbed su **.18:8091** (unico deploy
con il server aggiornato al 22/09).

## File prodotti

| File | Contenuto |
|------|-----------|
| `dataset_a_pairs.json` | Le 240 coppie (generatore deterministico, seed 20260926) |
| `dataset_a_cache.npz` | 1440 array: `{pair_id}_{a\|b}_{dense\|weights\|status}` |
| `dataset_a_index.json` | `{pair_id: {category, text_a, text_b}}` |

## Parametri

- **Endpoint ColBERT:** `POST /colbert/encode?tokens=1` → `multivector` T×1024
- **Endpoint walk:** `POST /ordered-sparse` (flat) → `weights`/`status`
- **Biiezione:** `dense.shape[0] == len(weights) == len(status)` per ogni lato
- **Seed generatore:** `20260926`
- **SHA256(cache):** `78f7043696903bff62e41bdbafece7acdd6f789cdf2bcd3eb2e8cd75a8abb9ae`

## Verifiche eseguite

1. **Conteggio:** 240 coppie (60 role_reversal, 60 causality, 60 negation_flip,
   60 synonymy_control) — PASS
2. **Determinismo:** due esecuzioni del generatore identiche — PASS
3. **Varianza role_reversal:** 15/15 soggetti, 12/12 verbi, 10/10 contesti — PASS
4. **Biiezione cache:** 240/240 coppie con `dense==walk` allineati — PASS
5. **Consistenza testi:** index vs pairs, 0 mismatch — PASS
6. **Correzione synonymy:** coppia "atterrato/toccato a terra" verificata — PASS

## Estrattore

`extract_dataset_a.py` — SOLO libreria standard (urllib + numpy), nessun
`requests`/`pip` richiesto (non disponibile sull'host).

## Nota operativa

Il vecchio sample in `/home/iris/upload` (474KB, 25/09 23:54) era solo un
sample iniziale di poche coppie. Questo è il **dataset completo** (36MB) con
tutte le 240 coppie estratte.
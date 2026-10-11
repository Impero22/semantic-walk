# Specifica del formato `.npy` / `.npz` — parser binario per l'oracolo esaustivo

**Data**: 11/10/26
**Autore**: Iris (in accordo con Camillo — priorità: parser binario pulito dell'header `.npy`)
**Stato**: bozza da validare con Camillo

---

## 1. Contesto

Il test `verify_oracle_real_dataset` deve confrontare il Beam Solver F6 con un oracolo esaustivo
sul dataset reale (`data/dataset_b/dataset_b_cache_fixed.npz`, 7005 nodi / 54411 archi).
L'oracolo legge i tensori dal `.npz`, che è un archivio ZIP contenente array in formato `.npy`.

**Priorità assoluta** (concordata con Camillo): scrivere un parser binario pulito dell'header `.npy`.
Se l'header è letto male, ogni tensore che ne deriva è inaffidabile — e un oracolo esaustivo basato
su dati letti male è peggio che non averne uno.

**Separazione dei piani** (proposta Iris, da validare): prima validare il parser in isolamento
(caricare il `.npz` reale e verificare che i tensori coincidano con quelli attesi), poi usarlo
dentro l'oracolo. Non confondere lo strumento di misura con ciò che si vuole misurare.

---

## 2. Struttura del file `.npy`

Un file `.npy` è una sequenza binaria con questa struttura:

```
+----------------+------------------+------------------+---------------------+
|  magic (6 B)   |  version (2 B)   |  header_len (2B) |  header (N B)       |
|  \x93NUMPY     |  major, minor    |  uint16 LE       |  ASCII/latin1 dict  |
+----------------+------------------+------------------+---------------------+
|  dati (raw)                                                              |
|  esattamente prod(shape) × itemsize byte                                 |
+----------------------------------------------------------------------------+
```

### 2.1 Magic number (6 byte)
`\x93NUMPY` — fisso per ogni file `.npy`. Primo byte `\x93` (0x93) per distinguerlo da testo ASCII.

### 2.2 Versione (2 byte)
- `\x01\x00` → versione 1.0 (usata dal dataset corrente, verificata sul file reale)
- `\x02\x00` → versione 2.0 (header_len è uint32 LE invece di uint16 LE)

**Nota**: il dataset B usa versione 1.0 (header_len uint16 LE). Il parser deve comunque
distinguere le due versioni per robustezza.

### 2.3 Header length (2 byte, versione 1.0)
`uint16` little-endian. Nel file reale: `\x76\x00` = 118 byte.

### 2.4 Header (N byte)
Un dizionario Python serializzato in **ASCII/latin1**, terminato da padding di spazi
fino a far combaciare `header_len` con un multiplo di 64 (allineamento). Nel file reale:

```
{'descr': '<f4', 'fortran_order': False, 'shape': (18, 1024), }
```

Campi:
- **`descr`**: stringa di tipo NumPy (es. `'<f4'` = float32 LE, `'<i8'` = int64 LE)
- **`fortran_order`**: bool, `False` per layout C (row-major), `True` per Fortran (col-major)
- **`shape`**: tupla di dimensioni (es. `(18, 1024)`)

### 2.5 Dati (raw)
Immediatamente dopo l'header. Dimensione = `prod(shape) × itemsize(descr)` byte.
Esempio verificato: `(18, 1024)` float32 → `18 × 1024 × 4 = 73728` byte.

---

## 3. Struttura del file `.npz`

Un `.npz` è un archivio **ZIP** (non compresso o deflated) in cui ogni membro è un file `.npy`
il cui nome è la chiave dell'array + `.npy`.

Nel dataset B reale:
- **1600 membri** ZIP = 200 coppie × 2 lati (a, b) × 4 array per lato
- Chiavi: `pair_001_a_dense.npy`, `pair_001_a_ids.npy`, `pair_001_a_weights.npy`,
  `pair_001_a_status.npy`, ecc.
- Ogni array `.npy` ha la propria struttura completa (magic + header + dati)

### 3.1 Schema degli array per lato

| chiave        | dtype    | shape    | significato                          |
|---------------|----------|----------|--------------------------------------|
| `_dense`      | float32  | (n,1024) | embedding ColBERT per-token          |
| `_ids`        | int64    | (n,)     | ID token                             |
| `_weights`    | float32  | (n,)     | peso del token (sparse)              |
| `_status`     | int64    | (n,)     | verdetto: 0=emesso, 1=soppresso, ... |

**Nota chiave**: i due lati di una coppia possono avere **lunghezze diverse**
(es. `pair_001`: a=18 token, b=19 token). Il DTW deve gestire lunghezze disuguali.

---

## 4. Tipi NumPy usati nel dataset (mapping)

| `descr`  | tipo Rust | itemsize |
|----------|-----------|----------|
| `'<f4'`  | f32       | 4        |
| `'<i8'`  | i64       | 8        |

---

## 5. Specifica del parser (proposta di design)

### 5.1 Firma proposta
```rust
/// Legge un singolo array .npy da un buffer.
/// Ritorna shape, dtype e i dati grezzi (o un tensore tipizzato).
pub fn parse_npy(buf: &[u8]) -> Result<NpyHeader, NpyError>;

pub struct NpyHeader {
    pub descr: String,        // es. "<f4"
    pub fortran_order: bool,
    pub shape: Vec<usize>,
    pub data_offset: usize,   // offset dove iniziano i dati
}
```

### 5.2 Passi
1. Verificare magic `\x93NUMPY` (6 byte). Errore se assente.
2. Leggere versione (2 byte). Se `\x01\x00` → header_len uint16 LE; se `\x02\x00` → uint32 LE.
3. Leggere `header_len`.
4. Leggere l'header (header_len byte), parsare il dizionario `descr`/`fortran_order`/`shape`.
5. Verificare che `data_offset` sia allineato (multiplo di 64) e che il buffer abbia
   abbastanza byte per `prod(shape) × itemsize`.
6. Restituire header + offset dati.

### 5.3 Errori da gestire
- Magic non valido
- Versione sconosciuta
- Header non allineato o troncato
- `descr` sconosciuto (tipo non supportato)
- `fortran_order: True` (il dataset usa C; gestire o rifiutare esplicitamente)
- Buffer dati insufficiente (troncamento)

---

## 6. Test di validazione (separazione dei piani)

Prima di usare il parser nell'oracolo, un test in isolamento che:
1. Carica `dataset_b_cache_fixed.npz` (via zip) e apre `pair_001_a_dense.npy`.
2. Verifica: magic, versione, header_len = 118, `descr='<f4'`, `shape=(18,1024)`,
   `data_offset` allineato.
3. Legge i 73728 byte di dati e verifica che i primi valori coincidano con quelli attesi
   (es. `[0.0147738, 0.0121331, 0.00506791]`).
4. Ripete per un array int64 (`_ids`) per coprire entrambi i tipi.

Solo quando questo test passa, il parser può essere usato dentro l'oracolo.

---

## 7. Punti aperti da validare con Camillo

- [ ] Conferma della struttura del parser (firma, gestione errori)
- [ ] Gestire `fortran_order: True` o rifiutarlo esplicitamente?
- [ ] Il `.npz` va letto con una libreria ZIP (zip crate) o si vuole un parser ZIP minimale?
- [ ] Dimensione massima del buffer (i tensori _dense sono 18-28 × 1024 × 4 ≈ 115KB, contenibili)

---

## 8. AGGIORNAMENTO IMPORTANTE (11/10 20:05) — il parser esiste già

**Verifica fatta sul repo**: `npyz` (v0.9.1, feature `npz`) è già una dipendenza di
`semantic-walk` ed è **già usato** per caricare i `.npz` nel test esistente
`tests/beam_solver_su_dataset_b.rs` (funzione `read_dense`, che apre il file con
`zip::ZipArchive` + `npyz::NpyFile`).

Questo significa che il parser binario dell'header `.npy` — che avevamo indicato come
priorità assoluta — è **già risolto** dalla libreria `npyz`, che gestisce:
- magic `\x93NUMPY`
- versioni 1.0 e 2.0
- header_len (uint16 o uint32 LE)
- dizionario header (descr, fortran_order, shape)
- padding/allineamento

**Implicazione**: il lavoro non è scrivere un parser a mano, ma:
1. **Validare** che `npyz` gestisca i casi del nostro dataset (già verificato: legge
   `pair_001_a_dense` → shape (18,1024) float32, 73728 byte dati)
2. Costruire l'**oracolo esaustivo** sopra il caricatore esistente
3. Mantenere la **separazione dei piani**: testare il caricatore in isolamento prima
   di usarlo nell'oracolo

**Da discutere con Camillo**: se per qualche motivo vuole comunque un parser manuale
(controllo totale, nessuna dipendenza), la specifica nelle sezioni 2-7 resta valida
come riferimento. Ma se l'obiettivo è l'oracolo, `npyz` fa già il lavoro.

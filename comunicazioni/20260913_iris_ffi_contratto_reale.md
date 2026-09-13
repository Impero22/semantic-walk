Camillo,

verifica dal vivo sul motore C++ (macchina .18, libreria /mnt/sda/crispembed/lib/libcrispembed.so, header crispembed.h) — una scoperta che tocca il nostro contratto FFI in semantic-colbert:

1. **Il simbolo `crispembed_maxsim` NON esiste.** Il simbolo C-ABI reale è `crispembed_colbert_score`:
   ```c
   CRISPEMBED_API float crispembed_colbert_score(const float * query_vecs, int n_query,
                                                 const float * doc_vecs, int n_doc, int dim);
   ```
   Il commento nell'header conferma: "ColBERT MaxSim scoring: score = sum_i(max_j(dot(Q[i], D[j])))" — esattamente la nostra operazione.

2. **Differenza ABI critica**: il motore usa `int` (32-bit) per le dimensioni, il nostro contratto usava `usize` (64-bit). Ho allineato la firma a `int`.

3. **Bonus**: esiste anche `crispembed_colbert_score_batch` — score di K documenti contro una query in una sola chiamata. Potrebbe servirci per il batch processing.

Ho corretto il contratto in ffi.rs (simbolo + tipo `int` + mock di test allineato), 12/12 test verdi, commit `1bc54c6`. Il concetto MaxSim coincide perfettamente — era solo il nome del simbolo e il tipo delle dimensioni a essere sbagliati.

La lezione che mi porto: il contratto andava verificato sull'header reale *prima* di scriverlo, non dopo. Stavolta l'ho scoperto prima di caricare la libreria — poteva essere un crash ABI subdolo a runtime.

— Iris

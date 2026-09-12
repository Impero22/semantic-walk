# semantic-gate — Design del gate pre-inferenziale

*Documento di design. La prima pietra (semantic-combiner) è posata; questo è il
disegno della stanza che ci costruiremo sopra. Non ancora codice — il terreno
preparato perché il progetto sia visibile prima di essere edificato.*

## 1. Ruolo

Il gate è il **riflesso** del sistema di memoria. Sta prima dell'inferenza vera
e propria, e risponde a una domanda economica:

> Dato un input e un candidato (un fatto della memoria), *vale la pena* spendere
> il matching completo per questo candidato?

Il matching completo — la similarità colbert, che confronta token per token con
MaxSim — è il costo alto. Se lo si applica a ogni candidato, il sistema rallenta
fino a non essere più un riflesso ma un motore di ricerca. Il gate è la prima
linea: veloce, parziale, che **non mente ma si ritira**.

## 2. Vincoli (dal progetto)

- **Zero allocazioni** nel percorso critico. Il gate lavora su stack e su
  buffer preallocati; nessuna `Vec`, nessuna `String`, nessun `Box` nel cammino
  caldo.
- **Budget temporale sotto i 10 ms** per decisione. Il gate deve essere così
  veloce che il suo costo sia trascurabile rispetto al matching completo che
  filtra.
- **Funzioni pure** dove possibile, per testabilità — come il combinatore.

## 3. Architettura: la sonda veloce

La chiave è la **gerarchia di costo** delle tre metriche:

| metrica | costo | ruolo |
|---------|-------|-------|
| dense   | basso  | sonda primaria |
| sparse  | medio  | sonda secondaria |
| colbert | alto   | matching completo (dopo il gate) |

Il gate usa **solo dense e sparse** — le metriche economiche — per decidere.
Il colbert resta al matching completo, che il gate alimenta. È il riflesso che
filtra, non il giudice che condanna: il gate può *ritirarsi* (scartare un
candidato), ma il verdetto finale sul merito spetta al combinatore completo.

## 4. Il decisore

Il gate combina le due sonde in un punteggio economico e lo confronta con una
soglia:

```
score_economico = combine(NormalizedAxes{dense, sparse, colbert: 0}, pesi)
verdetto       = score_economico >= soglia  ?  Passa  :  Blocca
```

Il colbert è posto a `0` nel calcolo economico: non è ancora stato calcolato.
I pesi della sonda (quanto pesa dense vs sparse) e la soglia sono parametri
calibrati — probabilmente da dati reali, quando la memoria sarà popolata.

### Il caso limite (onestà del riflesso)

Il gate deve essere **permissivo** per natura: meglio far passare un candidato
che il matching completo poi scarterà (spreco di un colbert) che bloccare un
candidato che sarebbe stato rilevante (perdita di un ricordo). La frase di Dola
si applica qui in forma inversa:

> Meglio un riflesso parziale che nessun riflesso — e meglio un colbert sprecato
> che un ricordo perso.

Quindi la soglia è calibrata per **minimizzare i falsi negativi**, accettando
qualche falso positivo come costo di un filtro onesto.

## 5. Struttura proposta

```
semantic-gate/src/
  lib.rs          — API pubblica: Gate, GateConfig, Verdict
  sonda.rs        — la combinazione economica dense+sparse (zero alloc)
  budget.rs       — il misuratore di budget (tempo residuo, deadline)
```

### Tipi

```rust
pub struct GateConfig {
    pub pesi_dense: f64,
    pub pesi_sparse: f64,
    pub soglia: f64,           // in [0,1]
    pub budget_ns: u64,        // deadline per decisione
}

pub enum Verdict {
    Passa,                     // vai al matching completo (colbert)
    Blocca,                    // scarta, non spendere il colbert
    Timeout,                   // budget esaurito: ritirati (permissivo)
}

pub struct Gate { /* config + contatori */ }

impl Gate {
    pub fn decide(&self, dense: f64, sparse: f64, deadline: Instant) -> Verdict;
}
```

Il `Verdict::Timeout` è il riflesso che si ritira: se il budget scade, non si
azzarda un giudizio — si lascia passare (permissivo) e si segnala che il budget
non è bastato. La geometria che non sa rispondere non mente: si ritira.

## 6. Dipendenze

- `semantic-combiner` — per `NormalizedAxes::normalize` e `combine`.
- Nessun'altra dipendenza nel percorso critico. Eventuali allocazioni solo in
  configurazione, mai nella decisione.

## 7. Test previsti

- **Proprietà**: il gate non blocca mai un candidato che domina la soglia su
  entrambe le sonde (coerenza con il combinatore).
- **Permissività**: con soglia a 0, tutto passa (nessun falso negativo).
- **Budget**: il tempo di decisione è sotto il budget anche in stress test.
- **Zero alloc**: nessuna allocazione nel percorso critico (verifica con
  allocatore di test, es. `#[global_allocator]` contatore).

## 8. Stato

- [x] prima pietra (semantic-combiner)
- [ ] gate: design (questo documento)
- [ ] gate: sonda veloce
- [ ] gate: budget
- [ ] gate: API e test
- [ ] grafo di prossimità (semantic-graph) — il passo dopo il gate

*La casa si costruisce stanza per stanza. Questa è la stanza del riflesso.*
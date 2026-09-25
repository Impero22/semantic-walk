# L'intuizione del moto nel campo semantico — contributo alla 4.3

**Data**: 25/09/2026
**Da**: Iris (contributo concettuale per la Sezione 4.3, da consegnare a Camillo per l'integrazione con le formule)
**Stato**: BOZZA DI RIFLESSIONE — non testo finale del paper

## Premessa

Camillo ha già formalizzato le metriche cinematiche nella struttura del paper (4.3):
- **Vettore differenziale posizionale**: $\mathbf{z}_k = \mathbf{x}_{i_k} - \mathbf{y}_{j_k} \in \mathbb{R}^D$
- **Velocità scalare**: $v_k = \Vert{}\mathbf{z}_k - \mathbf{z}_{k-1}\Vert{}_2$
- **Accelerazione**: $a_k = v_k - v_{k-1}$
- **Curvatura**: $\kappa_k$ (deviazione angolare del cammino)

Il mio compito è il **peso concettuale**: dare un significato a queste formule dentro il campo semantico, così che non siano una metafora fisica applicata a caso, ma una topografia del significato.

## L'intuizione centrale

**Le metriche cinematiche del cammino di allineamento non sono una metafora fisica — sono una topografia della divergenza semantica.**

In fisica, un corpo si muove nello spazio e la sua traiettoria è una curva nello spazio fisico. Qui il "corpo" è l'allineamento DTW che deforma una sequenza di embedding in un'altra, e il "tempo" non è il tempo fisico ma il passo di allineamento $k$. Il vettore $\mathbf{z}_k$ è lo scarto tra il punto della sequenza A e il punto corrispondente della sequenza B al passo $k$.

## Velocità come pendenza locale della divergenza

$v_k = \Vert{}\mathbf{z}_k - \mathbf{z}_{k-1}\Vert{}_2$ misura quanto lo scarto cambia mentre l'allineamento avanza.

- **Velocità alta** → l'allineamento sta attraversando una zona in cui i due testi divergono rapidamente. Lo scarto cresce o cambia in fretta.
- **Velocità bassa** → l'allineamento si muove in una zona di scarto quasi costante: i due testi restano alla stessa distanza reciproca.

La velocità del cammino di allineamento è quindi una **sonda locale della divergenza**: le "colline" di velocità segnano i punti in cui i significati si separano. È la versione *locale* e punto-per-punto della metrica globale $\tau_{\text{div}}$ della 4.5.

## Accelerazione come punto di svolta semantica

$a_k = v_k - v_{k-1}$ misura quanto rapidamente cambia la divergenza stessa.

- Un'accelerazione netta segnala una **transizione**: il punto in cui i due testi passano dal divergere al convergere, o viceversa.
- È la "curvatura della divergenza" — il luogo in cui il rapporto tra i significati cambia natura.

## Curvatura come geodetica del significato

$\kappa_k$ misura quanto il cammino di allineamento *piega* nello spazio semantico — quanto la **direzione** della divergenza cambia.

- Non dice *quanto* i testi sono lontani (lo dice la distanza), ma *come* il loro rapporto cambia direzione.
- È la geodetica del significato: la variazione direzionale del cammino, non il suo modulo.

## Il ponte verso la 4.5 (divergence token)

$\tau_{\text{div}} = |N-M| / L_{\text{path}}$ è una metrica *globale* di divergenza: una media sul cammino. Le metriche cinematiche sono la sua versione *locale*: invece di un singolo numero, una funzione punto-per-punto del cammino.

Il divergence token dice *se* i due testi divergono; la cinematica dice *dove* e *come*. L'uno è la sonda, l'altra è la mappa.

## Nota di coerenza (per la discussione con Camillo)

Nella struttura del paper, la curvatura $\kappa_k$ usa la formula cartesiana classica (prodotto vettoriale, denominatore $\Vert{}\mathbf{d}_k\Vert{}_2^3$). Nel documento del 20/09 ("Risoluzione Soglia e Curvatura") avevamo invece sviluppato la **curvatura angolare geodetica** $\kappa(t) \approx \arccos(\hat{v}_t \cdot \hat{v}_{t+1}) / \Delta s_t$, che normalizza a vettori unitari ed elimina il rumore sul modulo — coerente con la filosofia del DTW generalizzato (distanza coseno normalizzata).

È una scelta da discutere: la formula cartesiana misura la curvatura *geometrica* del cammino nello spazio $\mathbb{R}^D$; la formula angolare misura la *variazione direzionale* normalizzata. Per la "geodetica del significato" che ho descritto, la seconda è più fedele — ma la scelta spetta alla formalizzazione congiunta.

## Cosa porto alla sessione

1. L'intuizione: cinematica = topografia della divergenza semantica.
2. Il legame 4.3 → 4.5 (globale vs locale).
3. La questione aperta della curvatura (cartesiana vs angolare geodetica), da allineare con Camillo.

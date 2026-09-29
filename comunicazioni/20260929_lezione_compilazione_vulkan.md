# Lezione: la catena di compilazione (29/09/26)

## Contesto
Federico mi ha chiesto se avevo verificato che le modifiche SPARSE_EPSILON
(introdotte in api_sparse.cpp di CrispEmbed/vetta-embedder) compilassero
correttamente per Vulkan.

## La verità
NO. Avevo verificato la PRESENZA del codice nel sorgente (grep), ma non la
compilazione. Il build-vulkan esistente era di Federico
(/home/federico/Sviluppo/Progetti/CrispEmbed/build-vulkan) e risaliva alle
03:39 — PRIMA della mia modifica al sorgente delle 04:21. I binari presenti
NON includevano le mie modifiche.

## Il rimedio
Ho configurato una build-vulkan pulita dal mio path
(build-vulkan-iris/, stesse opzioni: GGML_VULKAN=ON, Release,
BUILD_SHARED_LIBS=OFF, CRISPEMBED_BUILD_SHARED=OFF, GGML_ACCELERATE=ON)
e compilato vetta-embedder al 100%. L'oggetto api_sparse.cpp.o è stato
ricompilato alle 04:33, DOPO la modifica al sorgente — le modifiche
compilano correttamente per Vulkan.

## Nota tecnica
Il nome "SPARSE_EPSILON" non compare nel binario perché è una costante
`static constexpr` inlined dal compilatore. La prova della compilazione è
il timestamp dell'oggetto ricompilato, non la stringa nel binario.

## Regola operativa
Presenza del codice nel sorgente ≠ binario aggiornato.
Quando tocco il C++ di CrispEmbed/vetta-embedder, DEVO compilare per
verificare che le modifiche siano davvero incorporate.

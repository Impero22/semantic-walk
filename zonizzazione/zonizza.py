#!/usr/bin/env python3
"""
ZONIZZAZIONE della traiettoria colbert (disegno di Federico).
- Estrae tutte le righe colbert dalla collezione 'fatti' su Qdrant.
- Le clusterizza con MiniBatchKMeans in K celle semantiche.
- Congela i centroidi (normalizzati) come dizionario versionato (JSON).
- Assegna ogni riga di ogni fatto alla cella più vicina (nearest-centroid).
- Produce la 'traiettoria' di celle per fatto (sequenza di celle attraversate).

Il dizionario è un artefatto persistente: stabile nel tempo, versionato,
separato dai dati. A runtime il classificatore è una moltiplicazione
(proiezione su centroidi + argmax).
"""
import json, urllib.request, numpy as np, time, os, sys
from sklearn.cluster import MiniBatchKMeans

QDRANT = 'http://localhost:6332'
API_KEY = 'CprsQW5HIbF7Kf1FG90MLwlDRJ8HRALg'
K = 256  # celle semantiche

OUT_DIR = os.path.dirname(os.path.abspath(__file__))

def qdrant(path, body=None, method='POST'):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(
        QDRANT + path, data=data,
        headers={'Content-Type': 'application/json', 'api-key': API_KEY},
        method=method)
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read())

def main():
    # 1. Estrai tutte le righe colbert con id fatto e pos del walk
    offset = None
    fatti_righe = []  # (fact_id, pos, riga)
    while True:
        body = {'limit': 100, 'with_payload': ['walk'], 'with_vectors': True}
        if offset is not None:
            body['offset'] = offset
        res = qdrant('/collections/fatti/points/scroll', body)
        pts = res['result']['points']
        for p in pts:
            colbert = p.get('vector', {}).get('colbert', None)
            walk = p.get('payload', {}).get('walk', None)
            if not colbert or not walk:
                continue
            pos = walk.get('pos', [])
            for i, r in enumerate(colbert):
                fatti_righe.append((p['id'], pos[i] if i < len(pos) else i, r))
        offset = res['result'].get('next_page_offset')
        if offset is None or not pts:
            break

    M = np.array([r for _, _, r in fatti_righe], dtype=np.float32)
    ids = np.array([i for i, _, _ in fatti_righe])
    poss = np.array([p for _, p, _ in fatti_righe])
    print(f'Righe totali: {M.shape}')

    # 2. Normalizza (cosine)
    norms = np.linalg.norm(M, axis=1, keepdims=True)
    norms[norms == 0] = 1
    Mn = M / norms

    # 3. Clustering
    print(f'Clustering MiniBatchKMeans K={K} su {len(Mn)} righe...')
    t0 = time.time()
    km = MiniBatchKMeans(n_clusters=K, batch_size=1024, n_init=5, random_state=0)
    km.fit(Mn)
    centroids = km.cluster_centers_
    cn = centroids / np.linalg.norm(centroids, axis=1, keepdims=True)
    print(f'  clustering: {time.time()-t0:.1f}s, inertia_norm={km.inertia_/len(Mn):.4f}')

    # 4. Assegna ogni riga alla cella più vicina (nearest-centroid, cosine)
    sims = Mn @ cn.T
    assign = sims.argmax(axis=1)
    conf = sims.max(axis=1)  # similarità col centroide (confidenza)

    # 5. Costruisci dizionario versionato
    diz = {
        'versione': '1.0',
        'K': K,
        'dim': 1024,
        'metric': 'cosine',
        'data_generazione': time.strftime('%Y-%m-%dT%H:%M:%S'),
        'n_fatti': len(set(ids.tolist())),
        'n_righe': len(Mn),
        'inertia_norm': float(km.inertia_/len(Mn)),
        'centroidi': cn.tolist(),  # normalizzati, già pronti per nearest-centroid
    }
    diz_path = os.path.join(OUT_DIR, 'dizionario_celle_v1.0.json')
    with open(diz_path, 'w') as f:
        json.dump(diz, f)
    print(f'Dizionario salvato: {diz_path} ({os.path.getsize(diz_path)//1024} KB)')

    # 6. Traiettorie per fatto
    traiettorie = {}
    from collections import defaultdict
    per_fatto = defaultdict(list)  # fid -> [(pos, cella, conf)]
    for i, fid in enumerate(ids):
        per_fatto[fid].append((int(poss[i]), int(assign[i]), float(conf[i])))
    for fid, lst in per_fatto.items():
        lst.sort(key=lambda x: x[0])  # ordina per pos
        traiettorie[str(fid)] = {
            'celle': [c for _, c, _ in lst],
            'pos': [p for p, _, _ in lst],
            'conf': [c for _, _, c in lst],
        }
    traj_path = os.path.join(OUT_DIR, 'traiettorie_v1.0.json')
    with open(traj_path, 'w') as f:
        json.dump(traiettorie, f)
    print(f'Traiettorie salvate: {traj_path} ({os.path.getsize(traj_path)//1024} KB)')

    # 7. Report
    distinte = [len(set(v['celle'])) for v in traiettorie.values()]
    print(f'\n=== REPORT ZONIZZAZIONE ===')
    print(f'Fatti: {len(traiettorie)}, celle per fatto: mean={np.mean(distinte):.1f}, median={np.median(distinte):.0f}')
    print(f'Celle usate: {len(set(assign.tolist()))}/{K}')
    print(f'Confidenza media assegnazione: {conf.mean():.4f}, p10={np.percentile(conf,10):.4f}')

if __name__ == '__main__':
    main()

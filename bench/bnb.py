#!/usr/bin/env python3
"""Branch-and-bound exato para bin packing — versão Python.

MESMO ALGORITMO do src/exact.rs (Rust), do bnb.c e do bnb.cpp:
  - incumbent inicial = First Fit Decreasing;
  - itens em ordem DECRESCENTENTE;
  - DFS com Poda 1 (incumbent) e Poda 2 (lower bound ceil(R - F)).

Sem os mesmos PODAS, a comparação de tempo entre linguagens mede
algoritmos diferentes — e portanto não significa nada. É por isso que
a implementacao replica a ordem de exploracao do Rust.

Uso: bnb.py <dir_instancias> <saida.csv> [n_max]
"""
import math
import os
import struct
import sys
import time

EPS = 1e-9


def ffd_bins(it):
    """Incumbent inicial — mesmo do Rust."""
    s = sorted(it, reverse=True)
    res = []
    for x in s:
        for b in range(len(res)):
            if x <= res[b] + EPS:
                res[b] -= x
                break
        else:
            res.append(1.0 - x)
    return len(res)


def optimal_bins(items):
    if not items:
        return 0, 0.0
    melhor = ffd_bins(items)
    itens = sorted(items, reverse=True)
    n = len(itens)
    resto = [0.0] * (n + 1)
    for i in range(n - 1, -1, -1):
        resto[i] = resto[i + 1] + itens[i]

    residuos = []

    def dfs(i):
        nonlocal melhor
        nb = len(residuos)
        if nb >= melhor:                        # Poda 1
            return
        if i == n:
            melhor = nb
            return
        r = resto[i]
        f = 0.0
        for x in residuos:
            f += x
        novos = int(math.ceil((r - f) - EPS)) if r > f else 0   # Poda 2
        if nb + novos >= melhor:
            return
        x = itens[i]
        for b in range(nb):
            if x <= residuos[b] + EPS:
                residuos[b] -= x
                dfs(i + 1)
                residuos[b] += x
        residuos.append(1.0 - x)
        dfs(i + 1)
        residuos.pop()

    t0 = time.perf_counter()
    dfs(0)
    ms = (time.perf_counter() - t0) * 1e3
    return melhor, ms


def main(dirpath, outpath, n_max=60):
    names = sorted(f for f in os.listdir(dirpath) if f.endswith(".f64"))
    feitos = 0
    with open(outpath, "w") as out:
        out.write("distribuicao,n,rep,optimal,bnb_ms\n")
        for name in names:
            with open(os.path.join(dirpath, name), "rb") as f:
                data = f.read()
            items = struct.unpack(f"<{len(data) // 8}d", data)
            stem = name[:-4]
            rest, rep = stem.rsplit("_r", 1)
            dist, _n = rest.rsplit("_n", 1)
            if len(items) > n_max:
                continue
            opt, ms = optimal_bins(items)
            out.write(f"{dist},{len(items)},{int(rep)},{opt},{ms:.4f}\n")
            feitos += 1
    print(f"{feitos} instancias com n <= {n_max} resolvidas (Python) -> {outpath}",
          file=sys.stderr)


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"uso: {sys.argv[0]} <dir> <saida.csv> [n_max]", file=sys.stderr)
        sys.exit(1)
    main(sys.argv[1], sys.argv[2],
         int(sys.argv[3]) if len(sys.argv) > 3 else 60)
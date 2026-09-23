#!/usr/bin/env python3
"""Benchmark Python — heurísticas clássicas de bin packing.

Lê instâncias .f64 (little-endian) exportadas pelo binário Rust,
roda NF/FF/BF/FFD/BFD e grava CSV no mesmo formato.

Uso: python3 bench.py <dir_instancias> <saida.csv>
"""
import os
import struct
import sys
import time

EPS = 1e-9


def next_fit(items):
    bins, residual = 1, 1.0
    for x in items:
        if x <= residual + EPS:
            residual -= x
        else:
            bins += 1
            residual = 1.0 - x
    return bins


def first_fit(items):
    res = []
    for x in items:
        for b in range(len(res)):
            if x <= res[b] + EPS:
                res[b] -= x
                break
        else:
            res.append(1.0 - x)
    return len(res)


def best_fit(items):
    res = []
    for x in items:
        best, best_r = -1, 1e18
        for b, r in enumerate(res):
            if x <= r + EPS and r < best_r:
                best, best_r = b, r
        if best >= 0:
            res[best] -= x
        else:
            res.append(1.0 - x)
    return len(res)


def first_fit_decreasing(items):
    return first_fit(sorted(items, reverse=True))


def best_fit_decreasing(items):
    return best_fit(sorted(items, reverse=True))


ALGS = [
    ("NF", next_fit),
    ("FF", first_fit),
    ("BF", best_fit),
    ("FFD", first_fit_decreasing),
    ("BFD", best_fit_decreasing),
]


def parse_name(name):
    """{dist}_n{n}_r{rep}.f64 — dist contém '_', quebra pela DIREITA."""
    stem = name.rsplit(".", 1)[0]
    rest, rep = stem.rsplit("_r", 1)
    dist, n = rest.rsplit("_n", 1)
    return dist, int(rep), int(n)


def main(dirpath, outpath):
    names = sorted(f for f in os.listdir(dirpath) if f.endswith(".f64"))
    with open(outpath, "w") as out:
        out.write("algoritmo,distribuicao,n,rep,bins,lower_bound,razao_vs_lb,tempo_us,soma_tamanhos\n")
        for name in names:
            with open(os.path.join(dirpath, name), "rb") as f:
                data = f.read()
            items = struct.unpack(f"<{len(data) // 8}d", data)
            dist, rep, n = parse_name(name)
            total = sum(items)

            for alg_name, fn in ALGS:
                fn(items)  # aquecimento
                t0 = time.perf_counter()
                bins = fn(items)
                t1 = time.perf_counter()
                out.write(
                    f"{alg_name},{dist},{len(items)},{rep},{bins},0,0.000000,"
                    f"{(t1 - t0) * 1e6:.1f},{total:.6f}\n"
                )
    print(f"{len(names)} arquivos processados (Python) -> {outpath}", file=sys.stderr)


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"uso: {sys.argv[0]} <dir_instancias> <saida.csv>", file=sys.stderr)
        sys.exit(1)
    main(sys.argv[1], sys.argv[2])

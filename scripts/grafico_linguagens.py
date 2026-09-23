#!/usr/bin/env python3
"""Gera o gráfico de comparação entre linguagens a partir dos CSVs de bench.

Uso: python3 scripts/grafico_linguagens.py [bench_rust.csv bench_c.csv ...]
"""
import csv
import sys
from collections import defaultdict

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

CORES = {
    "rust": "#e63757",   # vermelho-rust
    "c": "#5566d6",      # azul
    "cpp": "#7b68ae",    # roxo
    "python": "#f5c542", # amarelo
}
ALGS = ["NF", "FF", "BF", "FFD", "BFD"]


def load(path):
    out = defaultdict(list)
    for r in csv.DictReader(open(path)):
        key = (r["algoritmo"], r["distribuicao"], int(r["n"]))
        out[key].append(float(r["tempo_us"]))
    return {k: sum(v) / len(v) for k, v in out.items()}


def main(paths):
    # paths: bench_{lang}.csv
    langs = {}
    for p in paths:
        lang = p.split("_")[1].split(".")[0]
        langs[lang] = load(p)

    fig, axes = plt.subplots(1, 2, figsize=(13, 4.5))

    # painel A: barras NF (linear) — n=16000, uniforme discreta
    ax = axes[0]
    dist = "uniforme_discreta_100"
    n = 16000
    lang_list = list(langs)
    means = [langs[l][("NF", dist, n)] for l in lang_list]
    bars = ax.bar(lang_list, means, color=[CORES.get(l, "gray") for l in lang_list])
    for b, m in zip(bars, means):
        ax.text(b.get_x() + b.get_width() / 2, m * 1.02, f"{m:.0f}µs", ha="center", fontsize=9)
    ax.set_title(f"Next Fit (O(n)) — n={n}, {dist}")
    ax.set_ylabel("tempo médio (µs)")
    ax.set_ylim(0, max(means) * 1.15)

    # painel B: barras FF (quadrático)
    ax = axes[1]
    means = [langs[l][("FF", dist, n)] for l in lang_list]
    bars = ax.bar(lang_list, means, color=[CORES.get(l, "gray") for l in lang_list])
    for b, m in zip(bars, means):
        ax.text(b.get_x() + b.get_width() / 2, m * 1.02, f"{m/1000:.1f}ms", ha="center", fontsize=9)
    ax.set_title(f"First Fit (O(n²)) — n={n}, {dist}")
    ax.set_ylabel("tempo médio (µs)")
    ax.set_ylim(0, max(means) * 1.15)

    fig.suptitle("Comparação entre linguagens — mesmas instâncias, mesmos algoritmos", y=1.03)
    fig.tight_layout()
    fig.savefig("grafico_linguagens.png", dpi=150, bbox_inches="tight")
    print("gerado: grafico_linguagens.png")

    # tabela de razões
    print(f"\n{'':8s} " + "  ".join(f"{l:>8s}" for l in lang_list))
    for alg in ALGS:
        row = [langs[l][(alg, dist, n)] for l in lang_list]
        base = min(row[:3]) if len(row) >= 3 else row[0]  # base = mais rápido compilado
        print(f"{alg:8s} " + "  ".join(f"{v:>8.0f}" for v in row) + f"   (rust/C = {langs['rust'][(alg, dist, n)] / langs['c'][(alg, dist, n)]:.2f}x)" if "rust" in langs and "c" in langs else "")


if __name__ == "__main__":
    main(sys.argv[1:] or ["bench_rust.csv", "bench_c.csv", "bench_cpp.csv", "bench_python.csv"])

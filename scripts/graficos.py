#!/usr/bin/env python3
"""Gera os gráficos do artigo a partir de resultados.csv.

Uso: python3 scripts/graficos.py [caminho/para/resultados.csv]
"""
import csv
import sys
from collections import defaultdict

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

ALGS = ["NF", "FF", "BF", "FFD", "BFD"]
COLORS = {"NF": "#e74c3c", "FF": "#3498db", "BF": "#2ecc71", "FFD": "#9b59b6", "BFD": "#f39c12"}
DISTS = ["uniforme_continua", "uniforme_discreta_100", "tres_particao", "falkenauer_u120"]
DIST_LABELS = ["U[0,1] cont.", "U{1/100..1}", "3-partição", "Falkenauer"]


def main(csv_path="resultados.csv"):
    rows = list(csv.DictReader(open(csv_path)))
    for r in rows:
        r["n"] = int(r["n"])
        r["time_us"] = float(r["tempo_us"])
        r["ratio"] = float(r["razao_vs_lb"])

    agg_t = defaultdict(list)
    agg_r = defaultdict(list)
    for r in rows:
        agg_t[(r["algoritmo"], r["distribuicao"], r["n"])].append(r["time_us"])
        agg_r[(r["algoritmo"], r["distribuicao"], r["n"])].append(r["ratio"])

    # Gráfico 1: tempo × n (log-log)
    fig, ax = plt.subplots(figsize=(7, 4.5))
    dist = "uniforme_discreta_100"
    ns = sorted({r["n"] for r in rows if r["distribuicao"] == dist})
    for alg in ALGS:
        ts = [np.mean(agg_t[(alg, dist, n)]) for n in ns]
        ax.plot(ns, ts, "o-", color=COLORS[alg], label=alg, markersize=5)
    c = agg_t[("FF", dist, ns[0])][0] / ns[0] ** 2
    ax.plot(ns, [c * n * n for n in ns], "--", color="gray", alpha=0.7, label=r"$\Theta(n^2)$ (ref.)")
    c1 = agg_t[("NF", dist, ns[0])][0] / ns[0]
    ax.plot(ns, [c1 * n for n in ns], ":", color="gray", alpha=0.7, label=r"$\Theta(n)$ (ref.)")
    ax.set_xscale("log", base=2)
    ax.set_yscale("log")
    ax.set_xlabel("n (itens)")
    ax.set_ylabel("tempo médio (µs)")
    ax.set_title("Tempo de execução × n — distribuição uniforme discreta")
    ax.legend()
    ax.grid(True, which="both", alpha=0.3)
    fig.tight_layout()
    fig.savefig("grafico_tempo_n.png", dpi=150)
    plt.close(fig)

    # Gráfico 2: razão por distribuição (barras)
    fig, axes = plt.subplots(1, 4, figsize=(14, 3.8), sharey=True)
    for ax, d, dl in zip(axes, DISTS, DIST_LABELS):
        means = [
            np.mean([x for (alg, dd, n), xs in agg_r.items() if alg == a and dd == d for x in xs])
            for a in ALGS
        ]
        bars = ax.bar(ALGS, means, color=[COLORS[a] for a in ALGS])
        ax.axhline(11 / 9, color="black", linestyle="--", linewidth=1, label="11/9 ≈ 1.222 (FFD)")
        ax.axhline(1.7, color="gray", linestyle=":", linewidth=1, label="1.7 (FF/NF)")
        ax.set_title(dl)
        ax.set_ylim(1.0, 1.42)
        for b, m in zip(bars, means):
            ax.text(b.get_x() + b.get_width() / 2, m + 0.005, f"{m:.3f}", ha="center", fontsize=8)
    axes[0].set_ylabel("razão A(I)/L2(I)")
    axes[0].legend(fontsize=8)
    fig.suptitle("Razão de aproximação média (vs lower bound L2) por distribuição", y=1.02)
    fig.tight_layout()
    fig.savefig("grafico_razao_distribuicao.png", dpi=150, bbox_inches="tight")
    plt.close(fig)

    # Gráfico 3: razão × n no caso 3-partição
    fig, ax = plt.subplots(figsize=(7, 4.5))
    dist = "tres_particao"
    ns = sorted({r["n"] for r in rows if r["distribuicao"] == dist})
    for alg in ALGS:
        rs = [np.mean(agg_r[(alg, dist, n)]) for n in ns]
        ax.plot(ns, rs, "o-", color=COLORS[alg], label=alg, markersize=5)
    ax.axhline(11 / 9, color="black", linestyle="--", linewidth=1, label="11/9 (garantia FFD)")
    ax.set_xlabel("n (itens)")
    ax.set_ylabel("razão A(I)/L2(I)")
    ax.set_title("Razão de aproximação × n — caso 3-partição")
    ax.legend()
    ax.grid(True, alpha=0.3)
    fig.tight_layout()
    fig.savefig("grafico_razao_3particao.png", dpi=150)
    plt.close(fig)

    print("gráficos gerados: grafico_tempo_n.png, grafico_razao_distribuicao.png, grafico_razao_3particao.png")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "resultados.csv")

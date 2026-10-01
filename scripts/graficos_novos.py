#!/usr/bin/env python3
"""Gera os gráficos dos experimentos novos: progresso, ordenado x
desordenado e pior caso contra o ótimo exato.

Entrada (produzida por `cargo run --release --bin exporta_graficos`):
    dados_progresso.csv, dados_ordenado.csv, dados_piorcaso.csv

Uso:  /usr/bin/python3 scripts/graficos_novos.py
"""
import csv
from collections import defaultdict

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

ALGS = ["NF", "FF", "BF", "FFD", "BFD"]
CORES = {"NF": "#e74c3c", "FF": "#3498db", "BF": "#2ecc71", "FFD": "#9b59b6", "BFD": "#f39c12"}
# FFD e BFD dão EXATAMENTE o mesmo número de bins na maioria das
# distribuições (só muda a escolha do bin, não a contagem). Sem estilos
# de linha distintos, a curva do FFD fica escondida sob a do BFD.
ESTILOS = {"NF": "-", "FF": "-", "BF": "--", "FFD": "-.", "BFD": ":"}
LARGURAS = {"NF": 2.4, "FF": 2.0, "BF": 1.8, "FFD": 2.0, "BFD": 2.0}
# hachura distinta: nas barras FFD e BFD têm valor 0 (ordenar é redundante
# para eles), então a cor sozinha não basta
HATCH = {"NF": "", "FF": "///", "BF": "\\\\", "FFD": "...", "BFD": "xxx"}
DISTS = ["uniforme_continua", "uniforme_discreta_100", "tres_particao", "falkenauer_u120"]
DIST_ROT = ["U[0,1]", "U{1/100}", "3-partição", "Falkenauer"]
NS = [1000, 4000, 16000]


def carregar(path):
    with open(path) as f:
        return list(csv.DictReader(f))


# ---------------------------------------------------------------- progresso
def grafico_progresso():
    """Bins abertos × item processado: como a heurística 'anda'."""
    rows = carregar("dados_progresso.csv")
    d = defaultdict(dict)
    for r in rows:
        d[(r["algoritmo"], r["distribuicao"])][int(r["item"])] = int(r["bins_abertos"])

    fig, axes = plt.subplots(1, 4, figsize=(15, 3.9), sharey=True)
    for ax, dist, rot in zip(axes, DISTS, DIST_ROT):
        for alg in ALGS:
            serie = d[(alg, dist)]
            xs = sorted(serie)
            ax.plot(
                xs,
                [serie[x] for x in xs],
                color=CORES[alg],
                ls=ESTILOS[alg],
                lw=LARGURAS[alg],
                label=alg,
                alpha=0.95,
            )
        ax.set_title(rot, fontsize=10)
        ax.set_xlabel("itens processados")
        ax.grid(True, alpha=0.3)
    axes[0].set_ylabel("bins abertos")
    axes[0].legend(fontsize=8, ncol=2)
    # FFD e BFD produzem a MESMA curva de contagem nestas distribuições
    # (diferem apenas em qual bin cada item ocupa, não quantos bins são
    # abertos). Marcamos isso para que o leitor não leia a sobreposição
    # como um erro de desenho.
    axes[0].annotate(
        "FFD e BFD sobrepostos:\nmesma contagem de bins",
        xy=(0.52, 0.30),
        xycoords="axes fraction",
        fontsize=7,
        color="#7d3c98",
        bbox=dict(boxstyle="round,pad=0.3", fc="#f4ecf7", ec="#9b59b6", lw=0.7),
    )
    fig.suptitle(
        "Progresso das 5 heurísticas: bins abertos conforme os itens são processados (n = 200)",
        y=1.04,
    )
    fig.tight_layout()
    fig.savefig("grafico_progresso.png", dpi=150, bbox_inches="tight")
    plt.close(fig)
    print("grafico_progresso.png")


# ------------------------------------------------------- ordenado x desord
def grafico_ordenado():
    """Efeito de ordenar a entrada: bins e tempo."""
    rows = carregar("dados_ordenado.csv")
    bins = defaultdict(list)
    tempo = defaultdict(list)
    for r in rows:
        k = (r["algoritmo"], r["distribuicao"], int(r["n"]))
        bins[k].append(int(r["bins_desordenado"]) - int(r["bins_ordenado"]))
        tempo[k].append((int(r["t_desordenado_us"]), int(r["t_ordenado_us"])))

    fig, axes = plt.subplots(1, 2, figsize=(13.5, 4.6))

    # (a) bins economizados ao ordenar (positivo = ordenar ajudou)
    ax = axes[0]
    largura = 0.15
    xs = np.arange(len(NS))
    for i, alg in enumerate(ALGS):
        vals = [np.mean(bins[(alg, "tres_particao", n)]) for n in NS]
        ax.bar(
            xs + i * largura, vals, largura,
            color=CORES[alg], label=alg,
            edgecolor="black", linewidth=0.6, hatch=HATCH[alg],
        )
    ax.set_xticks(xs + 2 * largura)
    ax.set_xticklabels([f"n = {n}" for n in NS])
    ax.set_ylabel("bins GANHOS ao ordenar\n(desordenado − ordenado)")
    ax.set_title("Ordenar a entrada melhora a qualidade?", fontsize=10)
    ax.axhline(0, color="black", lw=1)
    ax.legend(fontsize=8, ncol=3)
    ax.grid(True, axis="y", alpha=0.3)

    # (b) custo de tempo da ordenação
    ax = axes[1]
    for i, alg in enumerate(ALGS):
        vals = []
        for n in NS:
            pares = tempo[(alg, "tres_particao", n)]
            des = np.mean([p[0] for p in pares])
            ord_ = np.mean([p[1] for p in pares])
            vals.append(100.0 * (ord_ - des) / max(des, 1))
        ax.bar(
            xs + i * largura, vals, largura,
            color=CORES[alg], label=alg,
            edgecolor="black", linewidth=0.6, hatch=HATCH[alg],
        )
    ax.set_xticks(xs + 2 * largura)
    ax.set_xticklabels([f"n = {n}" for n in NS])
    ax.set_ylabel("variação do tempo ao ordenar (%)")
    ax.set_title("Custo temporal de ordenar", fontsize=10)
    ax.axhline(0, color="black", lw=1)
    ax.grid(True, axis="y", alpha=0.3)

    fig.suptitle("Vetor ordenado × vetor desordenado — distribuição 3-partição", y=1.02)
    fig.tight_layout()
    fig.savefig("grafico_ordenado_desordenado.png", dpi=150, bbox_inches="tight")
    plt.close(fig)
    print("grafico_ordenado_desordenado.png")


# --------------------------------------------------------------- pior caso
def grafico_pior_caso():
    """Razão contra o ÓTIMO EXATO nas famílias adversariais."""
    rows = carregar("dados_piorcaso.csv")
    familias = []
    for r in rows:
        if r["familia"] not in familias:
            familias.append(r["familia"])
    ROT = {
        "pior_nf_classico": "NF clássico\n(0,5 ± ε)",
        "pior_ff_meio_mais": "FF meio+mais\n(½+ε / 0,26)",
        "pior_3particao": "3-partição\nforçada",
        "melhor_perfeito": "melhor caso\n(todos 0,5)",
    }

    fig, axes = plt.subplots(1, 4, figsize=(15, 3.9), sharey=True)
    for ax, fam in zip(axes, familias):
        sub = [r for r in rows if r["familia"] == fam]
        ns = [int(r["n"]) for r in sub]
        for alg in ALGS:
            ras = [int(r[alg.lower()]) / max(int(r["opt"]), 1) for r in sub]
            ax.plot(
                ns, ras, marker="o", ls=ESTILOS[alg], color=CORES[alg],
                label=alg, markersize=4, lw=LARGURAS[alg], alpha=0.95,
            )
        ax.set_title(ROT.get(fam, fam), fontsize=9)
        ax.set_xlabel("n")
        ax.grid(True, alpha=0.3)
    axes[0].set_ylabel("razão A(I)/OPT(I)")
    axes[0].axhline(2.0, color="black", ls="--", lw=1, label="2,0 (garantia NF)")
    axes[0].axhline(11 / 9, color="gray", ls=":", lw=1.2, label="11/9 (garantia FFD)")
    axes[0].legend(fontsize=7)
    fig.suptitle(
        "Pior caso × ótimo exato: as famílias adversariais simples NÃO atingem as garantias",
        y=1.04,
    )
    fig.tight_layout()
    fig.savefig("grafico_pior_caso.png", dpi=150, bbox_inches="tight")
    plt.close(fig)
    print("grafico_pior_caso.png")


if __name__ == "__main__":
    grafico_progresso()
    grafico_ordenado()
    grafico_pior_caso()
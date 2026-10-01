#!/usr/bin/env python3
"""Compara as DUAS amostras independentes do benchmark multi-linguagem.

A amostra 1 (bench/runs_amostra1) e a amostra 2 (bench/runs) foram
medidas em sessoes diferentes, com a maquina em condicoes diferentes.
Se os numeros de tempo reproduzem entre as duas, a Tabela 2 do artigo
nao e sorte de uma unica execucao.

Compara a MEDIANA de cada amostra e calcula o desvio relativo.
"""

import csv
import os
import statistics as st
from collections import defaultdict

A1 = "bench/runs_amostra1"
A2 = "bench/runs"
DIST = "uniforme_discreta_100"
N = 16000
ALGS = ["NF", "FF", "BF", "FFD", "BFD"]
LANGS = ["rust", "c", "cpp", "python"]


def medianas(pasta, lang):
    """Mediana do tempo por (alg, n) com DIST=N, em todas as rodadas."""
    acc = defaultdict(list)
    for f in os.listdir(pasta):
        if not f.startswith(f"{lang}_") or not f.endswith(".csv"):
            continue
        for r in csv.DictReader(open(os.path.join(pasta, f))):
            if r["distribuicao"] == DIST and int(r["n"]) == N:
                acc[r["algoritmo"]].append(float(r["tempo_us"]))
    return {a: st.median(v) for a, v in acc.items() if v}


def main():
    if not os.path.isdir(A1):
        raise SystemExit(f"{A1} nao existe — a amostra 1 nao foi preservada")
    if not os.path.isdir(A2):
        raise SystemExit(f"{A2} nao existe — rode scripts/rebench.sh")

    m1 = {l: medianas(A1, l) for l in LANGS}
    m2 = {l: medianas(A2, l) for l in LANGS}

    print(f"  Comparacao das duas amostras (n={N}, {DIST})\n")
    print(f"  {'alg':5}{'ling':8}{'amostra1':>12}{'amostra2':>12}{'desvio':>9}")
    print("  " + "-" * 48)
    desvios = []
    for a in ALGS:
        for l in LANGS:
            if a not in m1[l] or a not in m2[l]:
                continue
            v1, v2 = m1[l][a], m2[l][a]
            d = 100 * (v2 - v1) / v1
            desvios.append((a, l, abs(d)))
            print(f"  {a:5}{l:8}{v1:12.1f}{v2:12.1f}{d:+8.1f}%")
        print()

    if not desvios:
        print("  sem dados comparáveis")
        return

    todos = [d for _, _, d in desvios]
    print(f"  Desvio entre amostras: mediana {st.median(todos):.1f}%, "
          f"máximo {max(todos):.1f}%")

    # ratios rust/C — o número que o artigo reporta
    print("\n  --- ratios rust/C (o número reportado no artigo) ---")
    print(f"  {'alg':6}{'amostra1':>10}{'amostra2':>10}{'desvio':>9}")
    print("  " + "-" * 38)
    for a in ALGS:
        if a not in m1["rust"] or a not in m1["c"]:
            continue
        r1 = m1["rust"][a] / m1["c"][a]
        r2 = m2["rust"][a] / m2["c"][a]
        print(f"  {a:6}{r1:10.3f}{r2:10.3f}{(r2 - r1) / r1 * 100:+8.1f}%")


if __name__ == "__main__":
    main()
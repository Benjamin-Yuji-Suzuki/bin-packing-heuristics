#!/usr/bin/env python3
"""Ajusta a curva empirica de tempo e compara com a previsao assintotica.

Usa o metodo de duplicacao (doubling): com n dobrando, uma serie
exatamente O(n) multiplica o tempo por 2 e uma serie exatamente O(n^2)
por 4. O expoente medido e' a media dos logaritmos dessas razoes.

Uso: python3 scripts/ajusta_curva.py dados_escala/rust_escala.csv
"""
import csv
import math
import statistics as st
from collections import defaultdict
import sys

ALGS = ["NF", "FF", "BF", "FFD", "BFD"]
DIST = [
    "uniforme_continua",
    "uniforme_discreta_100",
    "tres_particao",
    "falkenauer_u120",
]
PREV = {"NF": 1.0, "FF": 2.0, "BF": 2.0, "FFD": 2.0, "BFD": 2.0}


def main():
    caminho = sys.argv[1] if len(sys.argv) > 1 else "dados_escala/rust_escala.csv"
    rows = list(csv.DictReader(open(caminho)))
    if not rows:
        print("vazio:", caminho)
        return
    col = "tempo_bins_us" if "tempo_bins_us" in rows[0] else "tempo_us"

    acc = defaultdict(list)
    for r in rows:
        acc[(r["algoritmo"], r["distribuicao"], int(r["n"]))].append(float(r[col]))
    ns = sorted({k[2] for k in acc})

    print(f"Fonte: {caminho}  |  coluna: {col}")
    print(f"Tamanhos: {ns}\n")
    print("Expoente medido pelo metodo de duplicacao (n dobra) e por regressao\n")

    cab = f"{'alg':5}{'dist':22}"
    cab += "".join(f"{n:>9}" for n in ns[1:5])
    cab += f"{'expoente':>11}{'R2':>8}{'prev':>7}"
    print(cab)
    print("-" * len(cab))

    todos_exp = {}
    for a in ALGS:
        for d in DIST:
            ts = []
            for n in ns:
                v = acc.get((a, d, n))
                ts.append(st.mean(v) if v else None)
            if any(t is None or t <= 0 for t in ts):
                continue

            # metodo de duplicacao
            exp_dup = st.mean(
                [math.log2(ts[i + 1] / ts[i]) for i in range(len(ts) - 1)]
            )
            # regressao log-log
            xs = [math.log(n) for n in ns]
            ys = [math.log(t) for t in ts]
            mx, my = st.mean(xs), st.mean(ys)
            sxy = sum((a_ - mx) * (b_ - my) for a_, b_ in zip(xs, ys))
            sxx = sum((a_ - mx) ** 2 for a_ in xs)
            syy = sum((b_ - my) ** 2 for b_ in ys)
            exp_reg = sxy / sxx
            r2 = sxy**2 / (sxx * syy)
            todos_exp[(a, d)] = exp_reg

            linha = f"{a:5}{d:22}"
            for n in ns[1:5]:
                i = ns.index(n)
                linha += f"{ts[i]:9.0f}"
            linha += f"{exp_reg:11.3f}{r2:8.3f}{PREV[a]:7.1f}"
            print(linha)
        print()

    print("RESUMO POR ALGORITMO (media das 4 distribuicoes)")
    print("=" * 66)
    print(f"{'alg':6}{'expoente medido':>18}{'previsao':>11}{'erro':>10}")
    for a in ALGS:
        vs = [v for (alg, _), v in todos_exp.items() if alg == a]
        if not vs:
            continue
        m = st.mean(vs)
        print(
            f"{a:6}{m:18.3f}{PREV[a]:11.1f}{100*(m-PREV[a])/PREV[a]:9.1f}%"
        )

    print()
    print("Interpretacao: o expoente medido em toda a faixa deve aproximar o")
    print("expoente teorico. Vies systematicamente para baixo sugere custo fixo")
    print("por instancia (ordenacao, alocacao) que pesa nas escalas pequenas.")


if __name__ == "__main__":
    main()
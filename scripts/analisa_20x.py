#!/usr/bin/env python3
"""Analisa a variabilidade entre as 20 execuções independentes.

Responde: os números do artigo são estáveis, ou são ruído?

Critério — coeficiente de variação (CV = despadrão / média) da razão
média de cada (algoritmo, distribuição):
    CV < 1%   ESTÁVEL     o número do artigo é reproduzível
    1-5%      ACEITÁVEL   variação pequena; vale citar o intervalo
    > 5%      INSTÁVEL    o número do artigo não é confiável
"""

import csv
import statistics as st
from collections import defaultdict

ALGS = ["NF", "FF", "BF", "FFD", "BFD"]
DIST = [
    "uniforme_continua",
    "uniforme_discreta_100",
    "tres_particao",
    "falkenauer_u120",
]
N_ALVO = 16000


def main():
    caminho = "dados_20x/razoes.csv"
    with open(caminho) as fh:
        texto = fh.read()
    if not texto.strip():
        print("  nenhum dado encontrado")
        return
    # o CSV e' escrito pelo script sem cabecalho; aceita os dois formatos
    if texto.lstrip().startswith("rep,"):
        linhas = list(csv.DictReader(texto.splitlines()))
    else:
        campos = ["rep", "algoritmo", "distribuicao", "n",
                  "media_razao", "min_razao", "max_razao", "desvio"]
        linhas = [dict(zip(campos, ln.split(","))) for ln in texto.splitlines() if ln.strip()]
    if not linhas:
        print("  nenhum dado encontrado")
        return

    reps = sorted({int(r["rep"]) for r in linhas})
    print(f"  {len(reps)} repetições independentes, n={linhas[0]['n']}\n")

    # ---- variabilidade da razão média entre execuções ----
    print("  Coeficiente de variação da razão média entre execuções")
    print(f"  {'alg':5}{'distribuição':24}{'média':>9}{'desvio':>9}{'CV%':>8}{'min':>9}{'max':>9}  ")
    print("  " + "-" * 78)

    medidas: dict = {}
    for a in ALGS:
        for d in DIST:
            vs = [
                float(r["media_razao"])
                for r in linhas
                if r["algoritmo"] == a and r["distribuicao"] == d
            ]
            if not vs:
                continue
            m = st.mean(vs)
            sd = st.stdev(vs) if len(vs) > 1 else 0.0
            cv = 100 * sd / m if m else 0.0
            medidas[(a, d)] = (m, sd, cv, min(vs), max(vs))

            if cv < 1.0:
                marca = "ESTÁVEL"
            elif cv < 5.0:
                marca = "aceitável"
            else:
                marca = "INSTÁVEL <<<"
            rot = {
                "uniforme_continua": "U[0,1]",
                "uniforme_discreta_100": "U{1/100}",
                "tres_particao": "3-partição",
                "falkenauer_u120": "Falkenauer",
            }[d]
            print(
                f"  {a:5}{rot:24}{m:9.4f}{sd:9.5f}{cv:7.2f}%{min(vs):9.4f}{max(vs):9.4f}  {marca}"
            )

    # ---- veredito ----
    print()
    cvs = [v[2] for v in medidas.values()]
    print(f"  CV mediana : {st.median(cvs):.2f}%")
    print(f"  CV maxima  : {max(cvs):.2f}%")

    print()
    print("  Comparacao com os numeros do artigo (n = 16.000):")
    artigo = {
        ("NF", "uniforme_continua"): 1.328,
        ("NF", "uniforme_discreta_100"): 1.323,
        ("NF", "tres_particao"): 1.235,
        ("NF", "falkenauer_u120"): 1.198,
        ("FF", "uniforme_continua"): 1.021,
        ("FF", "uniforme_discreta_100"): 1.017,
        ("FF", "tres_particao"): 1.131,
        ("FF", "falkenauer_u120"): 1.032,
        ("BF", "uniforme_continua"): 1.011,
        ("BF", "uniforme_discreta_100"): 1.009,
        ("BF", "tres_particao"): 1.131,
        ("BF", "falkenauer_u120"): 1.031,
        ("FFD", "uniforme_continua"): 1.001,
        ("FFD", "uniforme_discreta_100"): 1.000,
        ("FFD", "tres_particao"): 1.106,
        ("FFD", "falkenauer_u120"): 1.014,
        ("BFD", "uniforme_continua"): 1.001,
        ("BFD", "uniforme_discreta_100"): 1.000,
        ("BFD", "tres_particao"): 1.106,
        ("BFD", "falkenauer_u120"): 1.014,
    }
    print(f"  {'alg':5}{'dist':14}{'artigo':>9}{'medido':>9}{'desvio':>9}  dentro")
    print("  " + "-" * 62)
    fora = []
    for (a, d), art in artigo.items():
        if (a, d) not in medidas:
            continue
        m = medidas[(a, d)][0]
        desvio = 100 * (m - art) / art
        ok = abs(desvio) <= 1.0
        if not ok:
            fora.append((a, d, art, m, desvio))
        rot = {
            "uniforme_continua": "U[0,1]",
            "uniforme_discreta_100": "U{1/100}",
            "tres_particao": "3-part",
            "falkenauer_u120": "Falk",
        }[d]
        print(f"  {a:5}{rot:14}{art:9.3f}{m:9.3f}{desvio:8.2f}%  {'sim' if ok else '<<< NAO'}")

    print()
    if not fora:
        print("  >>> Todos os 20 numeros da Tabela 2 reproduzem dentro de 1%.")
        print("  >>> A Tabela 2 e ESTAVEL: nao e ruido de maquina.")
    else:
        print(f"  >>> {len(fora)} de {len(artigo)} numeros divergem mais de 1%:")
        for a, d, art, m, des in fora:
            print(f"      {a} / {d}: artigo {art:.3f} vs medido {m:.3f} ({des:+.2f}%)")
        print("  >>> Esses numeros precisam de revisao antes de entrar no artigo.")

    # ---- limites de confianca ----
    print()
    print("  Margem de erro da media (95%, 20 amostras):")
    print(f"  {'alg':5}{'dist':14}{'media':>9}{'erro95%':>10}")
    print("  " + "-" * 40)
    for a in ALGS:
        for d in DIST:
            if (a, d) not in medidas:
                continue
            m, sd, cv = medidas[(a, d)][:3]
            erro = 2.093 * sd / (len(reps) ** 0.5)  # t de Student para 19 gl
            rot = {
                "uniforme_continua": "U[0,1]",
                "uniforme_discreta_100": "U{1/100}",
                "tres_particao": "3-part",
                "falkenauer_u120": "Falk",
            }[d]
            print(f"  {a:5}{rot:14}{m:9.4f}{erro:9.5f}")


if __name__ == "__main__":
    main()
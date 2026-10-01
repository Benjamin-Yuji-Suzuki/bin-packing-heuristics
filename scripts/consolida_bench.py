#!/usr/bin/env python3
"""Consolida as 3 rodadas do benchmark: valida identidade de bins e
reporta a MEDIANA do tempo por (linguagem, algoritmo, n=16000).

Separa duas coisas que antes se confundiam no artigo:
  - IDENTIDADE: os bins batem entre as linguagens? (sim/não, contagem)
  - TEMPO: mediana das 3 rodadas, na serie bins-only
"""
import csv
import glob
import os
import statistics as st
from collections import defaultdict

ALGS = ["NF", "FF", "BF", "FFD", "BFD"]
N_FOCO = 16000
DIST_FOCO = "uniforme_discreta_100"


def carregar(path):
    with open(path) as f:
        return list(csv.DictReader(f))


def main():
    base = "bench/runs"
    # Descobre as rodadas a partir dos arquivos de UMA linguagem. Um glob
    # "*_1.csv" casaria tambem com c_1, cpp_1, python_1 e rust_1 — foi
    # exatamente isso que produziu "rodadas: [1,1,1,1]" e misturou as
    # linguagens na mediana.
    langs = ["rust", "c", "cpp", "python"]
    langs = [l for l in langs if os.path.exists(f"{base}/{l}_1.csv")]
    if not langs:
        raise SystemExit(f"nenhum CSV em {base} — rode scripts/rebench.sh")
    rodadas = sorted(
        int(p.rsplit("_", 1)[1].replace(".csv", ""))
        for p in glob.glob(f"{base}/{langs[0]}_*.csv")
    )
    print(f"linguagens: {langs}")
    print(f"rodadas:    {rodadas}\n")

    dados = {}  # (lang, rodada) -> lista de dicts
    for lang in langs:
        for r in rodadas:
            p = f"{base}/{lang}_{r}.csv"
            if os.path.exists(p):
                dados[(lang, r)] = carregar(p)
    if not dados:
        raise SystemExit("nenhum CSV em bench/runs — rode scripts/rebench.sh")

    # ---------- 1. IDENTIDADE DE BINS ----------
    print("=== IDENTIDADE DE BINS (mesma chave = mesmo bins?) ===")
    def chave(rows):
        return {
            (r["algoritmo"], r["distribuicao"], r["n"], r["rep"]): int(r["bins"])
            for r in rows
        }
    base_keys = chave(dados[(langs[0], rodadas[0])])
    total_div = 0
    for lang in langs:
        for r in rodadas:
            k = chave(dados[(lang, r)])
            comum = set(base_keys) & set(k)
            div = [c for c in comum if base_keys[c] != k[c]]
            total_div += len(div)
            marca = "OK" if not div else f"{len(div)} DIVERG"
            print(f"   {lang:7} rodada {r}: {len(comum)} chaves, {marca}")
    print(f"\n   >>> divergencias totais: {total_div}")
    print(f"   >>> chaves por rodada: {len(base_keys)} (4 linguagens x {len(rodadas)} rodadas = "
          f"{len(base_keys)} comparacoes de identidade, NAO 9.000)\n")

    # ---------- 2. TEMPO (mediana de 3) ----------
    print(f"=== TEMPO MEDIANO de {len(rodadas)} execucoes — n={N_FOCO}, {DIST_FOCO} ===")
    tempos = defaultdict(list)  # (lang, alg) -> [t1,t2,t3]
    for lang in langs:
        for r in rodadas:
            if (lang, r) not in dados:
                continue
            for row in dados[(lang, r)]:
                if row["distribuicao"] == DIST_FOCO and int(row["n"]) == N_FOCO:
                    tempos[(lang, row["algoritmo"])].append(float(row["tempo_us"]))

    print(f"{'alg':5}" + "".join(f"{l:>13}" for l in langs) + "   rust/C  rust/C++")
    for alg in ALGS:
        linha = f"{alg:5}"
        meds = {}
        for lang in langs:
            v = tempos.get((lang, alg), [])
            m = st.median(v) if v else float("nan")
            meds[lang] = m
            linha += f"{m:13.1f}"
        rc = meds["rust"] / meds["c"] if meds["c"] else float("nan")
        rpp = meds["rust"] / meds["cpp"] if meds["cpp"] else float("nan")
        linha += f"  {rc:6.3f}  {rpp:6.3f}"
        print(linha)

    # ---------- 3. RAZÃO rust/C por alg, para comparar com a TABELA 2 ----------
    print("\n=== rust/C por algoritmo (para conferir contra a Tabela 2 do artigo) ===")
    for alg in ALGS:
        rc = tempos[("rust", alg)]
        cc = tempos[("c", alg)]
        rpp = tempos[("rust", alg)]
        cpp = tempos[("cpp", alg)]
        if rc and cc:
            m_r, m_c = st.median(rc), st.median(cc)
            m_cpp = st.median(cpp)
            print(f"   {alg:5} rust={m_r:10.1f}  C={m_c:10.1f}  rust/C={m_r/m_c:.3f}   "
                  f"C++={m_cpp:10.1f}  rust/C++={m_r/m_cpp:.3f}")


if __name__ == "__main__":
    main()
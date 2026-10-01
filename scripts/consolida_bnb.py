#!/usr/bin/env python3
"""Consolida o benchmark do BRANCH-AND-BOUND entre linguagens.

Valida o ponto decisivo antes de reportar tempo: as 4 linguagens
encontram o MESMO ótimo? (Se não, a comparação de tempo é inválida.)

Entrada: bench/runs/bnb_{rust,c,cpp,py}.csv
Saída:   bench/bnb_consolidado.csv
"""
import csv
import statistics as st
from collections import defaultdict

LANGS = ["rust", "c", "cpp", "py"]
NOME = {"rust": "Rust", "c": "C", "cpp": "C++", "py": "Python"}


def load(p):
    return list(csv.DictReader(open(p)))


def main():
    dados = {}
    for l in LANGS:
        try:
            dados[l] = load(f"bench/runs/bnb_{l}.csv")
        except FileNotFoundError:
            raise SystemExit(f"falta bench/runs/bnb_{l}.csv — rode scripts/bnb.sh")

    # ---------- 1. MESMO ÓTIMO? ----------
    print("=== VALIDAÇÃO: as 4 linguagens acham o MESMO ótimo? ===")
    opt = {
        l: {(r["distribuicao"], r["n"], r["rep"]): int(r["optimal"]) for r in rows}
        for l, rows in dados.items()
    }
    base = opt["rust"]
    chaves = set(base)
    todos_ok = True
    for l in LANGS[1:]:
        comum = chaves & set(opt[l])
        div = [c for c in comum if base[c] != opt[l][c]]
        todos_ok &= not div
        print(f"   Rust vs {NOME[l]:7}: {len(comum):4} instâncias, {len(div)} divergências")
    print(f"\n   >>> ÓTIMO IDÊNTICO NAS 4 LINGUAGENS: {todos_ok}")
    if not todos_ok:
        raise SystemExit("ABORTADO: optimally different => comparar tempo não faz sentido")
    print()

    # ---------- 2. TEMPO ----------
    print("=== TEMPO DO B&B (ms), mediana das repetições, por n ===")
    t = defaultdict(list)
    for l in LANGS:
        for r in dados[l]:
            t[(l, int(r["n"]))].append(float(r["bnb_ms"]))
    ns = sorted({int(r["n"]) for r in dados["rust"]})

    print(f"{'n':>5}" + "".join(f"{NOME[l]:>12}" for l in LANGS))
    for n in ns:
        m = [st.median(t[(l, n)]) for l in LANGS]
        print(f"{n:>5}" + "".join(f"{x:12.4f}" for x in m))

    tot = {l: sum(float(r["bnb_ms"]) for r in dados[l]) for l in LANGS}
    print(f"\n{'TOTAL':>5}" + "".join(f"{tot[l]:12.1f}" for l in LANGS) + "   (ms, todas as instâncias)")

    print("\n=== multiplicadores (Python / compiladas) ===")
    for l in ["c", "cpp", "rust"]:
        print(f"   Python / {NOME[l]:7} = {tot['py'] / tot[l]:5.1f}x")

    print("\n=== compiladas entre si (convergem?) ===")
    for a, b in [("rust", "c"), ("rust", "cpp"), ("c", "cpp")]:
        print(f"   {NOME[a]:5} / {NOME[b]:5} = {tot[a] / tot[b]:.3f}x")

    # ---------- 3. CSV consolidado ----------
    with open("bench/bnb_consolidado.csv", "w") as f:
        f.write("distribuicao,n,rep,optimal," + ",".join(f"{l}_ms" for l in LANGS) + "\n")
        idx = {l: {(r["distribuicao"], r["n"], r["rep"]): r["bnb_ms"] for r in dados[l]}
               for l in LANGS}
        for k in sorted(chaves, key=lambda x: (x[0], int(x[1]), int(x[2]))):
            f.write(f"{k[0]},{k[1]},{k[2]},{base[k]}," +
                    ",".join(idx[l][k] for l in LANGS) + "\n")
    print("\n   -> bench/bnb_consolidado.csv")


if __name__ == "__main__":
    main()
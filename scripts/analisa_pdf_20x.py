#!/usr/bin/env python3
"""Compara as 20 compilacoes do artigo.

Responde: o PDF sai identico a cada compilacao? Se as tabelas mudam de
pagina, se uma referencia resolve diferente, se um numero muda de
alinhamento, o documento nao e reproduzivel.

Compara o TEXTO EXTRAIDO (pdftotext), nao o binario do PDF: o PDF
carimba um timestamp e um ID de arquivo, que mudam a cada compilacao sem
que o conteudo mude.
"""

import hashlib
import os
import re
import sys
from collections import Counter

PASTA = "dados_20x/pdf"


def texto(i):
    with open(f"{PASTA}/pdf_{i}.txt", encoding="utf8", errors="replace") as f:
        return f.read()


def norm(t):
    """Normaliza espacos de linha, que o pdftotext pode variar com a
    posicao das figures."""
    return re.sub(r"[ \t]+", " ", t)


def main():
    if not os.path.isdir(PASTA):
        raise SystemExit(f"{PASTA} nao existe — rode scripts/verificar_pdf_20x.sh")

    def numero(f):
        m = re.search(r"pdf_(\d+)\.txt", f)
        return int(m.group(1)) if m else -1

    arquivos = sorted(
        numero(f) for f in os.listdir(PASTA) if f.endswith(".txt")
    )
    n = len(arquivos)
    print(f"  {n} compilacoes encontradas\n")

    if n < 2:
        print("  precisa de pelo menos 2 compilacoes para comparar")
        return

    textos = {i: norm(texto(i)) for i in arquivos}

    # ---- 1. Identidade do texto ----
    digests = {i: hashlib.sha256(textos[i].encode()).hexdigest() for i in arquivos}
    iguais = [i for i in arquivos if digests[i] == digests[arquivos[0]]]
    print("  --- 1. Texto extraido ---")
    if len(iguais) == n:
        print(f"      IDENTICO nas {n} compilacoes (sha256 {digests[arquivos[0]][:16]})")
    else:
        print(f"      {len(iguais)}/{n} identicas a primeira; {n - len(iguais)} divergem")

    # ---- 2. Onde divergem ----
    divergentes = [i for i in arquivos if digests[i] != digests[arquivos[0]]]
    if divergentes:
        print("\n  --- 2. Onde as compilacoes divergentes diferem ---")
        base = textos[arquivos[0]].split("\n")
        for i in divergentes[:3]:
            outras = textos[i].split("\n")
            print(f"\n      compilacao {i} vs {arquivos[0]}:")
            diffs = 0
            for ln, (a, b) in enumerate(zip(base, outras)):
                if a != b:
                    print(f"        linha {ln}: {a[:60]!r}")
                    print(f"                 {b[:60]!r}")
                    diffs += 1
                    if diffs >= 4:
                        print("        ...")
                        break
            if len(base) != len(outras):
                print(
                    f"        ATENCAO: numero de linhas difere "
                    f"({len(base)} vs {len(outras)}) — houve quebra de pagina"
                )

    # ---- 3. Estrutura ----
    print("\n  --- 3. Estrutura do documento ---")
    t = textos[arquivos[0]]
    secs = re.findall(r"^\d+\.\s+(.+)$", t, re.M)
    figs = re.findall(r"Figura (\d+)\.", t)
    tabs = re.findall(r"Tabela (\d+)\.", t)
    print(f"      secoes   : {len(secs)} -> {', '.join(s[:28] for s in secs[:4])}...")
    print(f"      figuras  : {len(figs)} (numeros {sorted(set(figs), key=int)})")
    print(f"      tabelas  : {len(tabs)} (numeros {sorted(set(tabs), key=int)})")
    # a numeracao de figuras/tabelas e' sequencial sem buracos?
    fseq = sorted(set(map(int, figs)))
    tseq = sorted(set(map(int, tabs)))
    print(
        f"      figuras sequenciais: {'sim' if fseq == list(range(1, len(fseq) + 1)) else 'NAO'}"
    )
    print(
        f"      tabelas sequenciais: {'sim' if tseq == list(range(1, len(tseq) + 1)) else 'NAO'}"
    )

    # ---- 4. Numeros do artigo presentes em todas as copias ----
    print("\n  --- 4. Numeros-chave presentes em TODAS as compilacoes ---")
    chave = ["1.328", "1.131", "1.106", "706", "1,82", "54,5%", "0,130", "1,50", "28 testes"]
    for k in chave:
        em_todas = all(k in textos[i] for i in arquivos)
        em_algum = sum(1 for i in arquivos if k in textos[i])
        print(f"      {k:12} em {em_todas}/{n} compilacoes", end="")
        print("  OK" if em_todas else f"  <<< so em {em_algum}/{n}")

    # ---- 5. Veredito ----
    print()
    print("  --- VEREDITO ---")
    if len(iguais) == n:
        print("      O DOCUMENTO E REPRODUZIVEL: as 20 compilacoes produzem")
        print("      exatamente o mesmo texto. Nenhuma tabela muda de pagina,")
        print("      nenhuma referencia resolve diferente, nenhum numero muda.")
    else:
        print(f"      {n - len(iguais)} de {n} compilacoes divergem do texto da primeira.")
        print("      Ver o detalhe acima para saber o que variou.")
        print("      (variacao de alinhamento em geral nao e erro; quebra de pagina e.)")


if __name__ == "__main__":
    main()
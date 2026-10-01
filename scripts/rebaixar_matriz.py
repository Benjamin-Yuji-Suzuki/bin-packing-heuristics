#!/usr/bin/env python3
"""Reduz o nível de leitura 'Integral' da matriz de referências: 6 -> 3.

Mantém como 'Integral' apenas as 3 referências que sustentam o que o
artigo AFIRMA (as garantias de pior caso e o denominador das razões
medidas) e rebaixa as demais para 'Parcial'.

As 3 que permanecem:
  R01 Johnson, Demers, Ullman, Garey, Graham (1974) — todas as garantias
       de pior caso de NF/FF/BF (a Tabela 1 do artigo vem daqui);
  R02 Dósa (2007) — o limite 11/9·OPT + 6/9 que o FFD é confrontado contra;
  R05 Martello & Toth (1990) — o limitador L2, denominador de TODAS as
       razões A(I)/L2(I) reportadas no artigo.

Uso:  python3 scripts/rebaixar_matriz.py
"""
import re
import shutil
import zipfile
from pathlib import Path

XLSX = Path(__file__).resolve().parent.parent / "matriz-referencias-bin-packing.xlsx"
ABA = "xl/worksheets/sheet2.xml"  # "Matriz Analítica"

# ID -> linha real da planilha (verificado no XML; não assumido).
LINHAS = {
    "EX": 2,   # exemplo da planilha (Chvátal) — não é referência do artigo
    "R01": 3,
    "R02": 4,
    "R03": 5,
    "R05": 7,
    "R09": 11,
    "R10": 12,
    "R21": 23,
    "R22": 24,
}

PERMANECEM = {"R01", "R02", "R05"}

MOTIVOS = {
    "EX": "exemplo da planilha; nao e referencia do artigo",
    "R03": "versao em periodico do Dosa 2007 (mesma prova, extendida)",
    "R09": "linha random-order; so aparece como trabalho futuro do 2o bim.",
    "R10": "linha random-order; so aparece como trabalho futuro do 2o bim.",
    "R21": "usada so para validar o 4/3 do NF; o resultado esta no texto",
    "R22": "usada so para validar caso medio; o resultado esta no texto",
}


def eh_integral(xml: str, linha: int) -> bool:
    m = re.search(r'<c r="O%d"[^>]*>\s*<is><t[^>]*>(.*?)</t>' % linha, xml, re.S)
    return bool(m) and m.group(1).strip() == "Integral"


def main() -> None:
    if not XLSX.exists():
        raise SystemExit(f"nao encontrei {XLSX}")

    backup = XLSX.with_suffix(".xlsx.bak")
    shutil.copy2(XLSX, backup)
    print(f"backup criado: {backup.name}\n")

    with zipfile.ZipFile(XLSX) as z:
        nomes = z.namelist()
        dados = {n: z.read(n) for n in nomes}

    xml = dados[ABA].decode("utf-8")

    ok = 0
    for rid, linha in LINHAS.items():
        if rid in PERMANECEM:
            continue
        if not eh_integral(xml, linha):
            print(f"  {rid} (linha {linha}): nao esta 'Integral' — pulando")
            continue
        padrao = re.compile(r'(<c r="O%d"[^>]*>\s*<is><t[^>]*>)(.*?)(</t>)' % linha, re.S)
        xml, n = padrao.subn(lambda mm: mm.group(1) + "Parcial" + mm.group(3), xml, count=1)
        if n == 1:
            ok += 1
            print(f"  {rid} (linha {linha}): Integral -> Parcial   [{MOTIVOS.get(rid, '')}]")

    dados[ABA] = xml.encode("utf-8")

    tmp = XLSX.with_suffix(".tmp")
    with zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED) as z:
        for n in nomes:
            z.writestr(n, dados[n])
    tmp.replace(XLSX)

    print(f"\n{ok} referencias rebaixadas para 'Parcial'.")
    print(f"'Integral' restante: {sorted(PERMANECEM)}")
    print("\nConfira abrindo o .xlsx -> aba 'Matriz Analitica' -> coluna 'Nivel de leitura'.")


if __name__ == "__main__":
    main()
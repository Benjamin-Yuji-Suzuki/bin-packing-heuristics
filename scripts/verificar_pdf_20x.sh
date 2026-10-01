#!/usr/bin/env bash
# Compila o artigo 20 vezes e compara os PDFs byte a byte.
#
# POR QUE ISTO E' DIFERENTE DA VERIFICACAO DE DADOS: a verificacao
# 20x (scripts/verificar_20x.sh) mede se os NUMEROS sao estaveis. Esta
# mede se o DOCUMENTO e' reproduzivel — se o LaTeX produz o mesmo PDF
# a cada compilacao, ou se tabelas mudam de pagina, referencias
# resolvem diferente, numeros mudam de alinhamento.
#
# O que pode variar entre compilacoes:
#   - timestamp embutido no PDF (por isso compara-se o TEXTO extraido,
#     e nao o binario);
#   - posicao de floats e quebras de pagina;
#   - numeracao de referencias cruzadas em passadas diferentes.
#
# Uso: bash scripts/verificar_pdf_20x.sh
set -uo pipefail
cd "$(dirname "$0")/.."

REPETICOES=20
SAIDA="dados_20x/pdf"
mkdir -p "$SAIDA"
rm -f "$SAIDA"/*.txt

echo "=============================================="
echo " VERIFICACAO DE RIGIDEZ DO DOCUMENTO — ${REPETICOES} compilacoes"
echo "=============================================="

cd artigo
for i in $(seq 1 $REPETICOES); do
    # Compilacao limpa: 4 passadas (2 para refs cruzadas, 2 para bibtex)
    pdflatex -interaction=nonstopmode artigo-parcial.tex >/dev/null 2>&1
    bibtex artigo-parcial >/dev/null 2>&1
    pdflatex -interaction=nonstopmode artigo-parcial.tex >/dev/null 2>&1
    pdflatex -interaction=nonstopmode artigo-parcial.tex >/dev/null 2>&1
    pdftotext artigo-parcial.pdf "../$SAIDA/pdf_${i}.txt" 2>/dev/null
    cp artigo-parcial.pdf "../$SAIDA/pdf_${i}.pdf" 2>/dev/null
    printf "  compilacao %2d/%d  " "$i" "$REPETICOES"
    if [ -s "../$SAIDA/pdf_${i}.txt" ]; then
        echo "ok ($(wc -c < "../$SAIDA/pdf_${i}.txt") bytes de texto)"
    else
        echo "FALHOU"
    fi
done

cd ..
echo
echo "=============================================="
echo " COMPARACAO ENTRE AS ${REPETICOES} COMPILACOES"
echo "=============================================="
/usr/bin/python3 scripts/analisa_pdf_20x.py
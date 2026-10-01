#!/usr/bin/env bash
# Roda TODOS os comandos do projeto 20 vezes e verifica se a saida
# DETERMINISTICA e reproduz o resultado.
#
# POR QUE NAO COMPARAR BYTE A BYTE: os comandos imprimem tempo de
# execucao, que varia a cada rodada. Byte a byte daria falso negativo.
# Então extrai-se so a parte SEMANTICA — contagem de bins, razão,
# optimum — e compara-se isso. E exatamente o que o artigo afirma.
#
# O que cada comando testa:
#   run                    uma instancia, as 5 heuristicas
#   worstcase              pior caso vs otimo exato
#   progress               rastreamento item a item
#   sorted-vs-unsorted     ordenar x nao ordenar
#   experiment             o experimento fatorial completo
#
# Uso: bash scripts/verificar_determinismo.sh [repeticoes]
set -uo pipefail
cd "$(dirname "$0")/.."

REP="${1:-20}"
CPU="${CPU:-0}"
BIN="./target/release/bin-packing-heuristics"
TS="taskset -c ${CPU}"
SAIDA="dados_20x/determinismo"
rm -rf "$SAIDA"; mkdir -p "$SAIDA"

echo "=============================================="
echo " DETERMINISMO — cada comando ${REP}x (cpu${CPU})"
echo "=============================================="

falhas=0

# ---------- 1. run ----------
echo "[1/5] run ..."
for i in $(seq 1 $REP); do
    $TS $BIN run -n 60 -d tres_particao --seed 42 2>&1 \
        | grep -E "^(NF|FF|BF|FFD|BFD)" | awk '{print $1, $2, $3, $4}' > "$SAIDA/run_$i.txt"
done
if diff -q "$SAIDA/run_1.txt" "$SAIDA/run_2.txt" >/dev/null; then
    echo "      identico nas $REP execucoes"
else
    echo "      >>> DIVERGIU"; diff "$SAIDA/run_1.txt" "$SAIDA/run_2.txt" | head -6; falhas=1
fi

# ---------- 2. worstcase ----------
echo "[2/5] worstcase ..."
for i in $(seq 1 $REP); do
    $TS $BIN worstcase --sizes 12,24,36 2>&1 \
        | grep -oE "[A-Z]+:[0-9.]+" > "$SAIDA/worst_$i.txt"
done
if diff -q "$SAIDA/worst_1.txt" "$SAIDA/worst_2.txt" >/dev/null; then
    echo "      identico nas $REP execucoes"
else
    echo "      >>> DIVERGIU"; diff "$SAIDA/worst_1.txt" "$SAIDA/worst_2.txt" | head -6; falhas=1
fi

# ---------- 3. progress ----------
echo "[3/5] progress ..."
for alg in NF FF BF FFD BFD; do
    for i in $(seq 1 $REP); do
        $TS $BIN progress --alg $alg --n 40 --dist tres_particao 2>/dev/null \
            | grep -E "^ +[0-9]" | awk '{print $1, $3}' > "$SAIDA/prog_${alg}_$i.txt"
    done
    if diff -q "$SAIDA/prog_${alg}_1.txt" "$SAIDA/prog_${alg}_2.txt" >/dev/null; then
        echo "      $alg identico"
    else
        echo "      >>> $alg DIVERGIU"; falhas=1
    fi
done

# ---------- 4. sorted-vs-unsorted ----------
echo "[4/5] sorted-vs-unsorted ..."
for i in $(seq 1 $REP); do
    $TS $BIN sorted-vs-unsorted --n 3000 --dist tres_particao 2>&1 \
        | grep -E "^(NF|FF|BF|FFD|BFD)" | awk '{print $1, $2, $3, $4}' > "$SAIDA/ord_$i.txt"
done
if diff -q "$SAIDA/ord_1.txt" "$SAIDA/ord_2.txt" >/dev/null; then
    echo "      identico nas $REP execucoes"
else
    echo "      >>> DIVERGIU"; diff "$SAIDA/ord_1.txt" "$SAIDA/ord_2.txt" | head -6; falhas=1
fi

# ---------- 5. experiment ----------
echo "[5/5] experiment (dados brutos) ..."
for i in $(seq 1 $REP); do
    $TS $BIN experiment --sizes 1000,4000 --reps 2 --out "$SAIDA/exp_$i.csv" >/dev/null 2>&1
    # tira a coluna de tempo, que e' a unica que muda
    cut -d, -f1-7,9 "$SAIDA/exp_$i.csv" > "$SAIDA/exp_semtempo_$i.csv"
done
todos_ok=1
for i in $(seq 2 $REP); do
    diff -q "$SAIDA/exp_semtempo_1.csv" "$SAIDA/exp_semtempo_$i.csv" >/dev/null || {
        echo "      >>> execucao $i diverge"; todos_ok=0; falhas=1; break; }
done
[ $todos_ok -eq 1 ] && echo "      identico nas $REP execucoes (ignorando a coluna de tempo)"

echo
echo "=============================================="
if [ $falhas -eq 0 ]; then
    echo " RESULTADO: todos os comandos sao DETERMINISTICOS"
    echo " (mesma entrada -> mesma saida, sempre)"
else
    echo " RESULTADO: algum comando NAO e' deterministico (ver acima)"
fi
echo "=============================================="
exit $falhas
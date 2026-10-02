#!/usr/bin/env bash
# Amplia a escala do experimento ate n = 256.000 (Rust) e ate 64.000 (C/C++).
#
# POR QUE O LIMITE DIFERE POR LINGUAGEM: medido, o First Fit em Rust leva
# 6,6 s por instancia em n=256.000, mas o Python e' cerca de 1000x mais
# lento — o que daria horas. A comparacao multi-linguagem usa a serie
# bins-only, que e' o que o artigo compara; para a curva de crescimento
# (que e' a analise assintotica) o Rust sozinho ja responde, porque o
# algoritmo e' o mesmo nas quatro linguagens.
#
# Uso: bash scripts/escala_grande.sh [n_rust] [n_linguagens]
set -euo pipefail
cd "$(dirname "$0")/.."

N_RUST="${1:-256000}"
N_LING="${2:-64000}"
CPU="${CPU:-0}"
BIN=./target/release/bin-packing-heuristics
TS="taskset -c ${CPU}"

mkdir -p dados_escala

echo "=============================================="
echo " ESCALA GRANDE — n ate ${N_RUST} (Rust), ${N_LING} (linguagens)"
echo "=============================================="
cat /proc/loadavg | awk '{print "  carga: "$1}'
/usr/bin/python3 scripts/cpu_instantanea.py 2 > /tmp/cpu_pre.txt; head -3 /tmp/cpu_pre.txt

# Sequencia com duplicacao (doubling): 1k, 2k, 4k ... 256k
TAM=$(/usr/bin/python3 -c "
n=1024; seq=[]
while n <= $N_RUST:
    seq.append(str(n)); n*=2
print(','.join(seq))
")
echo
echo "Tamanhos (Rust): ${TAM}"
echo

echo "--- 1/3 experimento completo em Rust ate ${N_RUST} ---"
echo "    (4 distribuicoes x ${N_RUST} escala x 3 repeticoes x 5 algoritmos)"
$TS $BIN experiment --sizes "$TAM" --reps 3 --out dados_escala/rust_escala.csv 2>&1 | tail -2

echo
echo "--- 2/3 re-benchmark entre linguagens ate ${N_LING} ---"
TAM_LING=$(/usr/bin/python3 -c "
n=1024; seq=[]
while n <= $N_LING:
    seq.append(str(n)); n*=2
print(','.join(seq))
")
$TS $BIN export --sizes "$TAM_LING" --reps 3 --out inst_escala
mkdir -p dados_escala/runs
for rodada in 1 2 3; do
    $TS $BIN bench --dir inst_escala --out "dados_escala/runs/rust_${rodada}.csv" 2>&1 | tail -1
    $TS ./bench/bench_c   inst_escala "dados_escala/runs/c_${rodada}.csv"    2>&1 | tail -1
    $TS ./bench/bench_cpp inst_escala "dados_escala/runs/cpp_${rodada}.csv"  2>&1 | tail -1
    echo "  rodada ${rodada}/3 (Python pendente: custaria horas em n grande)"
done

echo
echo "--- 3/3 ajuste da curva empirica ate ${N_RUST} ---"
/usr/bin/python3 scripts/ajusta_curva.py dados_escala/rust_escala.csv

echo
echo "Dados em dados_escala/"
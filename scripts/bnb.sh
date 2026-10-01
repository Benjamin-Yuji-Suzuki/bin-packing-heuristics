#!/usr/bin/env bash
# Branch-and-and-bound entre linguagens: Rust x C x C++ x Python.
#
# IMPORTANTE — fair-play metodológico:
#   As quatro implementações executam o MESMO algoritmo: mesmo incumbent
#   inicial (FFD), mesma ordem decrescente, mesma ordem de exploração
#   da DFS, mesmas duas podas. Sem isso, comparar tempo mede algoritmos
#   diferentes. O Rust NÃO usa o atalho "se FFD <= L2, já acabou" que
#   existe em src/exact.rs, justamente para não pular trabalho.
#
# Saída: bench/runs/bnb_{lang}.csv  ->  consolida_bnb.py valida e resume.
set -euo pipefail
cd "$(dirname "$0")/.."

# Mesma política do rebench: não mede com a máquina ocupada.
if ! bash scripts/preflight.sh; then
    echo
    echo "ABORTADO. Feche o que estiver em uso e rode de novo."
    exit 1
fi
echo

NUCLEO="${NUCLEO:-0}"
TS="taskset -c ${NUCLEO}"
echo "fixando o branch-and-bound no nucleo ${NUCLEO} (taskset)"

N_MAX="${1:-20}"
REP="${2:-10}"

echo "==> exportando instâncias pequenas para o B&B (n até $N_MAX, $REP reps)"
SIZES=$(seq 10 2 "$N_MAX" | paste -sd,)
./target/release/bin-packing-heuristics export --sizes "$SIZES" --reps "$REP" --out instancias_bnb

echo "==> compilando (fair-play: -O3 -march=native / target-cpu=native)"
RUSTFLAGS="-C target-cpu=native" cargo build --release 2>&1 | tail -1
gcc -O3 -march=native          -o bench/bnb_c   bench/bnb.c   -lm 2>/dev/null
g++ -O3 -march=native -std=c++17 -o bench/bnb_cpp bench/bnb.cpp 2>/dev/null
echo "    ok"

mkdir -p bench/runs
echo "==> Rust"
$TS ./target/release/bnb_rust instancias_bnb bench/runs/bnb_rust.csv "$N_MAX" 2>&1 | tail -1
echo "==> C"
$TS ./bench/bnb_c   instancias_bnb bench/runs/bnb_c.csv   "$N_MAX" 2>&1 | tail -1
echo "==> C++"
$TS ./bench/bnb_cpp instancias_bnb bench/runs/bnb_cpp.csv "$N_MAX" 2>&1 | tail -1
echo "==> Python (o mais lento — patience)"
$TS /usr/bin/python3 bench/bnb.py instancias_bnb bench/runs/bnb_py.csv "$N_MAX" 2>&1 | tail -1

echo "==> consolidando"
/usr/bin/python3 scripts/consolida_bnb.py
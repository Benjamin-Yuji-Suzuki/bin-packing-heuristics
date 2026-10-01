#!/usr/bin/env bash
# Re-executa o benchmark comparativo Rust x C x C++ x Python com fair-play:
# todos em -O3 -march=native (Rust via RUSTFLAGS target-cpu=native).
#
# Saida: 3 execucoes completas por linguagem; reportamos a MEDIANA.
# Uso: bash scripts/rebench.sh
set -euo pipefail
cd "$(dirname "$0")/.."

# Recusa rodar com a máquina ocupada: medição contaminada não reproduz.
if ! bash scripts/preflight.sh; then
    echo
    echo "ABORTADO. Feche o que estiver em uso e rode de novo."
    echo "Se a carga residual for só o Hermes desktop (que não pode ser"
    echo "fechado), use: LIMITE_CARGA=1.5 bash scripts/rebench.sh"
    exit 1
fi
echo
# Fixa o benchmark num nucleo: com 12 threads e o sistema em ~19% de uso,
# o agendador poderia migrar o processo para um nucleo ocupado e introducir
# ruido. Fixar torna a medicao estavel e REPRODUZIVEL, e o nucleo usado
# fica declarado no artigo.
NUCLEO="${NUCLEO:-0}"
TS="taskset -c ${NUCLEO}"
echo "fixando o benchmark no nucleo ${NUCLEO} (taskset)"

{
  echo "carga (1 min) no inicio : $(cut -d' ' -f1 /proc/loadavg)"
  echo "carga (5 min) no inicio : $(cut -d' ' -f2 /proc/loadavg)"
  echo "threads                 : $(nproc)"
  echo "nucleo fixado (taskset) : ${NUCLEO}"
  echo "otimizacao              : Rust --release + target-cpu=native; C/C++ -O3 -march=native; CPython 3.12"
} | tee bench/carga_da_medicao.txt

echo "==> 1/5 exportando instâncias (200 arquivos .f64)"
./target/release/bin-packing-heuristics export --sizes 1000,2000,4000,8000,16000 --reps 10 --out instancias

echo "==> 2/5 compilando (fair-play: -O3 -march=native em todas)"
RUSTFLAGS="-C target-cpu=native" cargo build --release 2>&1 | tail -1
gcc   -O3 -march=native -o bench/bench_c   bench/bench.c   -lm
g++   -O3 -march=native -std=c++17 -o bench/bench_cpp bench/bench.cpp
echo "    compilado."

mkdir -p bench/runs
for rodada in 1 2 3; do
  echo "==> 3/5 rodada $rodada/3 — Rust"
  $TS ./target/release/bin-packing-heuristics bench --dir instancias --out "bench/runs/rust_$rodada.csv" 2>&1 | tail -1
  echo "==> 4/5 rodada $rodada/3 — C / C++ / Python"
  $TS ./bench/bench_c   instancias "bench/runs/c_$rodada.csv"    2>&1 | tail -1
  $TS ./bench/bench_cpp instancias "bench/runs/cpp_$rodada.csv"  2>&1 | tail -1
  $TS /usr/bin/python3 bench/bench.py    instancias "bench/runs/python_$rodada.csv" 2>&1 | tail -1
done

echo "==> 5/5 consolidando (mediana de 3 execuções)"
/usr/bin/python3 scripts/consolida_bench.py
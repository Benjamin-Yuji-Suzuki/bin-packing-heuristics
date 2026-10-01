#!/usr/bin/env bash
# Pre-flight: so roda o benchmark se a maquina estiver de fato livre.
#
# Motivo: benchmark medido com a CPU ocupada nao reproduz. Medimos
# desvios de ate 6,7% entre rodadas contaminadas e limpas — suficiente
# para distorcer um multiplo como rust/C, que e o numero reportado.
#
# Criterio: uso INSTANTANEO total do sistema (scripts/cpu_instantanea.py)
# abaixo de PCT_MAX.
#
# NAO usamos `ps pcpu` aqui: ele e a media de uso desde o inicio do
# processo. Um aplicativo aberto ha 23h marca ~30% mesmo estando parado.
# Para benchmark so interessa o uso agora, medido por deltas de
# /proc/<pid>/stat.
#
# Uso: bash scripts/preflight.sh
set -uo pipefail
cd "$(dirname "$0")/.."

PCT_MAX="${PCT_MAX:-25}"   # % de uso instantaneo do sistema aceito
NUCLEO="${NUCLEO:-0}"     # nucleo onde o benchmark sera fixado (P-core!)
JANELA="${JANELA:-2}"     # janela de medicao, em segundos

echo "=============================================="
echo " PRE-FLIGHT: a maquina esta livre para medir?"
echo "=============================================="

LOAD=$(cut -d' ' -f1 /proc/loadavg)
LOAD5=$(cut -d' ' -f2 /proc/loadavg)
echo "  carga (1 min) : $LOAD"
echo "  carga (5 min) : $LOAD5"
echo "  threads       : $(nproc)"
free -h | awk '/^Mem:/{print "  RAM livre     : "$7}'
GPU=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader 2>/dev/null | head -1 | tr -dc '0-9')
echo "  GPU           : ${GPU:-?}%  (irrelevante: o benchmark e CPU-only, single-threaded)"

echo
echo "  Uso INSTANTANEO de CPU (janela de ${JANELA}s):"
INST=$(/usr/bin/python3 scripts/cpu_instantanea.py "$JANELA" 2>/dev/null)
echo "$INST" | sed 's/^/    /'
SISTEMA=$(echo "$INST" | grep -oP 'uso total do sistema: \K[0-9.]+')
SISTEMA="${SISTEMA:-100}"

echo
# AVISO E-CORE: i5-13420H tem 4 P-cores (cpu0-7, hyperthread) e
# 4 E-cores (cpu8-11). Um E-core e ~2x mais LENTO — benchmark fixado
# num deles infla todos os tempos e invalida a comparacao com a
# tabela publicada.
if [ "$NUCLEO" -ge 8 ] 2>/dev/null; then
    echo " PERIGO: cpu${NUCLEO} e um E-CORE (cpu8-11), ~2x mais lento."
    echo " Os tempos serao ~2x maiores e NAO comparaveis com a tabela."
    echo " Use NUCLEO=0 (P-core)."
    echo "=============================================="
    exit 1
fi
echo "  nucleo escolhido: cpu${NUCLEO} (P-core, ok)"

echo
echo "=============================================="
if awk -v s="$SISTEMA" -v lim="$PCT_MAX" 'BEGIN{exit !(s+0 <= lim+0)}'; then
    echo " OK: sistema usando ${SISTEMA}% de ${PCT_MAX}% permitido."
    echo " O benchmark sera fixado no nucleo ${NUCLEO} (taskset)."
    echo "=============================================="
    exit 0
fi

echo " OCUPADO: sistema usando ${SISTEMA}% (limite ${PCT_MAX}%)."
echo " Quem esta usando agora:"
echo "$INST" | grep -E '^ +[0-9]' | sed 's/^/  /'
echo
echo " Feche o que estiver em uso e rode de novo."
echo "=============================================="
exit 1
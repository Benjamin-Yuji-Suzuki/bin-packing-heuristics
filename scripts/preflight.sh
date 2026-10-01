#!/usr/bin/env bash
# Pre-flight: só roda o benchmark se a máquina estiver de fato ociosa.
#
# Motivo: um benchmark medido com a CPU ocupada não reproduz. Medimos
# desvios de ate 6,7% entre rodadas contaminadas e limpas — suficiente
# para distorcer um multiplo como rust/C, que e o numero reportado.
#
# Criterio: load average de 1 minuto abaixo de LIMITE_CARGA E nenhum
# processo consumindo mais de PCT_MAX fora do proprio benchmark.
#
# Uso: bash scripts/preflight.sh [--forcar]
set -uo pipefail

LIMITE_CARGA="${LIMITE_CARGA:-1.0}"
PCT_MAX="${PCT_MAX:-20}"
FORCAR=0
[ "${1:-}" = "--forcar" ] && FORCAR=1

echo "=============================================="
echo " PRE-FLIGHT: verificando se a maquina esta idle"
echo "=============================================="

LOAD=$(uptime | awk -F'load average:' '{print $2}' | awk -F, '{gsub(/ /,"",$1); print $1}')
THREADS=$(nproc)
echo "  carga (1 min) : $LOAD"
echo "  threads       : $THREADS"
free -h | awk '/^Mem:/{print "  RAM livre     : "$7}'
GPU=$(nvidia-smi --query-gpu=utilization.gpu --format=csv,noheader 2>/dev/null | head -1 | tr -dc '0-9')
echo "  GPU           : ${GPU:-?}% (irrelevante: benchmark e CPU-only)"

echo
echo "  Processos acima de ${PCT_MAX}% de CPU:"
TOPO=$(ps -eo pcpu,comm --sort=-pcpu --no-headers | head -6)
echo "$TOPO" | awk -v p="$PCT_MAX" '{ if ($1+0 > p) printf "    >>> %s%%  %s\n", $1, $2; else printf "        %s%%  %s\n", $1, $2 }'

CONTAM=$(ps -eo pcpu,comm --no-headers | awk -v p="$PCT_MAX" '$1+0 > p' | wc -l)
CARGA_ALTA=$(awk -v l="$LOAD" -v lim="$LIMITE_CARGA" 'BEGIN{print (l+0 > lim+0) ? 1 : 0}')

echo
echo "=============================================="
if [ "$CONTAM" -eq 0 ] && [ "$CARGA_ALTA" -eq 0 ]; then
    echo " OK: maquina ociosa (carga $LOAD <= $LIMITE_CARGA)."
    echo " Pode rodar o benchmark."
    echo "=============================================="
    exit 0
fi

echo " CONTAMINADA — benchmark NAO deve rodar:"
[ "$CARGA_ALTA" -eq 1 ]   && echo "   - carga $LOAD acima do limite $LIMITE_CARGA"
[ "$CONTAM" -gt 0 ]       && echo "   - $CONTAM processo(s) acima de ${PCT_MAX}% de CPU"
echo
echo " Feche o que estiver em uso (Discord, Firefox, abas) e rode de novo."
echo " NOTA: o Hermes desktop ocupa ~48% da CPU e NAO pode ser fechado"
echo "       (e o processo que executa este script). Se o resto for"
echo "       zerado, a carga residual dele ainda barra o pre-flight —"
echo "       nesse caso use LIMITE_CARGA=1.5 bash scripts/rebench.sh"
echo "       e registre a carga real no artigo."
if [ "$FORCAR" -eq 1 ]; then
    echo
    echo " --forcar: rodando mesmo assim, por decisão explícita."
    exit 0
fi
echo "=============================================="
exit 1
#!/usr/bin/env bash
# Verificação de robustez: repete o experimento 20 vezes e checa se os
# números do artigo são estáveis.
#
# POR QUE 20 VEZES: uma medição única não diz se o resultado é o número
# ou o acaso da máquina. Se a razão média de um algoritmo varia em ±0,1%
# entre repetições, o número do artigo é um número. Se varia 30%, é ruído
# e precisa de mais repetições ou de outro protocolo.
#
# O que é fixado e o que varia:
#   - FIXADO: processador (cpu0, um P-core), flags de otimização, código.
#   - VARIA:  a semente mestra. Cada repetição usa seed = 1000*(rep+1)+n
#             com rep variando, o que gera instâncias DIFERENTES.
#
# Assim, medimos a variabilidade ENTRE INSTÂNCIAS, que é a fonte real de
# incerteza no artigo — o mesmo que 10 repetições por configuração já
# introduzem, mas agora com 20 amostras independentes para julgar.
#
# Uso: bash scripts/verificar_20x.sh [n_por_execucao]

set -uo pipefail
cd "$(dirname "$0")/.."

N="${1:-2000}"
REPETICOES=20
CPU="${CPU:-0}"

# Se a carga da maquina estiver alta, avisar antes de comecar
echo "=============================================="
echo " VERIFICACAO DE ROBUSTEZ — ${REPETICOES} repeticoes"
echo "=============================================="
/usr/bin/python3 scripts/cpu_instantanea.py 2
CARGA=$(cut -d' ' -f1 /proc/loadavg)
echo "carga (1 min): $CARGA   |   nucleo fixado: cpu${CPU}"
if awk -v l="$CARGA" 'BEGIN{exit !(l+0 > 1.5)}'; then
    echo "AVISO: carga $CARGA > 1.5. Para numero de tempo, rode com a maquina livre."
    echo "       Para a verificacao de RAZAO (qualidade), a carga importa pouco."
fi
echo

mkdir -p dados_20x
: > dados_20x/razoes.csv

for rep in $(seq 1 $REPETICOES); do
    # Repete o experimento com semente mestra = rep*10000, todas as
    # heuristicas, todas as 4 distribuicoes, no tamanho pedido.
    taskset -c "$CPU" ./target/release/bin-packing-heuristics experiment \
        --sizes "$N" --reps 3 --salt $((rep * 100000)) \
        --out "dados_20x/run_${rep}.csv" >/dev/null 2>&1
    # extrai as medias e anexa
    /usr/bin/python3 - "$rep" "dados_20x/run_${rep}.csv" <<'PYEOF' >> dados_20x/razoes.csv
import csv, sys, statistics
rep, path = sys.argv[1], sys.argv[2]
rows = list(csv.DictReader(open(path)))
agg = {}
for r in rows:
    k = (r['algoritmo'], r['distribuicao'], r['n'])
    agg.setdefault(k, []).append(float(r['razao_vs_lb']))
for (alg, dist, n), vs in sorted(agg.items()):
    m = statistics.mean(vs)
    print(f"{rep},{alg},{dist},{n},{m:.6f},{min(vs):.6f},{max(vs):.6f},{statistics.stdev(vs):.6f}")
PYEOF
    printf "  rep %2d/%d ok\n" "$rep" "$REPETICOES"
done

echo
echo "=============================================="
echo " VARIABILIDADE ENTRE AS ${REPETICOES} INDEPENDENCIAS"
echo "=============================================="
/usr/bin/python3 scripts/analisa_20x.py
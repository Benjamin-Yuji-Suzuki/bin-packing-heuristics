# Regime de medição — condição da máquina por rodada

O benchmark mede **exatamente**, não rápido. Carregou a máquina, o número
não é "com ressalva": é inválido. Esta é a série de condições em que cada
rodada foi medida, para o artigo **declarar a condição** em vez de afirmar
"máquina ociosa" sem prova.

Método: delta de `/proc/<pid>/stat` numa janela de 2 s (uso **instantâneo**).
`ps -eo pcpu` mede média desde o início do processo e reprova máquina
ociosa — não usar.

## Topologia (i5-13420H, 12 threads)

| CPUs | Tipo | Freq máx | Hyperthread |
|---|---|---|---|
| cpu0–cpu7 | **P-core** | 4,6 GHz | pares (0,1) (2,3) (4,5) (6,7) |
| cpu8–cpu11 | **E-core** | 3,4 GHz | nenhum |

⚠️ Fixar em **E-core (cpu8–11) dobra o tempo** e invalida a tabela sem erro
visível. Fixar em **P-core** é obrigatório; escolher o par de hyperthread
mais silencioso (irmãos compartilham L1) é o refinamento.

## Condição por rodada

| Rodada | Data/hora | Carga (Δ 2s, % de 1 núcleo) | P-core usado | Série | Válida? |
|---|---|---|---|---|---|
| Tabela 1 (linguagens, n=16k) | 24/09 | não registrada (parser defeituoso) | `taskset` | ver nota ⚠️ | ⚠️-condition não comprovada |
| Expoentes 1.024–256.000 | 02/10 03:1x | **medida** (recoleta sequencial) | `taskset -c 0` | bins-only | ✅ |
| Re-benchmark 65.536 | 02/10 | **medida** (recoleta sequencial) | `taskset -c 0` | bins-only | ✅ |

⚠️ **A Tabela 1 nunca teve a carga registrada.** O texto hoje diz "com a
carga do sistema monitorada e registrada", e isso não é verdade para essa
rodada: o parser que media a carga tinha defeito. A afirmação honesta é o
desvio mediano de 1,2% contra a medição anterior — que **é** verificável.

## Estado em 02/10 (medição para a re-coleta bins-only)

Carga total: **442% de 1 núcleo em 25 processos**. Não é possível benchmark.

| PID | Uso | Processo |
|---|---|---|
| 1026510 | 106,0% | `probe4` (benchmark do subagente de código) |
| 754721 | 85,0% | Hermes desktop (renderer) |
| 317159 | 56,0% | gnome-system-monitor |
| 754670 | 49,5% | Hermes desktop |
| 1181010 | 34,0% | cinnamon |
| 754827 | 32,5% | hermes (agente) |
| 1183598 | 12,5% | mintreport-tray |
| 1182331 | 9,0% | Discord |
| 754624 | 9,0% | Hermes desktop |
| 358290 | 8,5% | Zed |

O **próprio Hermes desktop** é o maior consumidor depois do benchmark do
subagente (~85% + 49,5% + 9%), e ele não pode ser fechado porque é o
processo que executa o comando. Idle absoluto é inalcançável durante a
sessão — por isso a regra é *um processo por vez* e *fixar em P-core*, não
"esperar a máquina ficar ociosa".

**Regra permanente (errei duas vezes na sessão de 02/10): nunca duas
medições de tempo em paralelo.** `taskset -c 0` impede *migração*, não
*competição*: dois fixados no mesmo núcleo físico disputam L1 e a unidade de
execução. Sintoma = valores **bimodais** na mesma condição declarada.

```bash
# gate antes de medir
pgrep -f "bench|experiment|probe"   # tem que voltar VAZIO
cut -d' ' -f1 /proc/loadavg        # nunca use uptime (vírgula decimal em pt-BR)
```

## Filtro de aceitação

Só entram no texto pontos com **CV < 15%** no tamanho comparado. Ruído em
tempos de microssegundos é esperado (granularidade do relógio) e não é
contaminação — o critério se aplica ao tamanho que entra na tabela.
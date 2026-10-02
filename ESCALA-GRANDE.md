# Escala até n = 256.000 — o que roda e o que não roda

Medição real de custo por tamanho (não extrapolação). Fonte:
`cargo run --release --bin custo_escala`.

## Custo medido por instância (série bins-only, Rust, P-core)

| n | NF | FF | BF | FFD | BFD | total |
|---|---|---|---|---|---|---|
| 16.000 | 49 µs | 24,8 ms | 67,4 ms | 29,8 ms | 94,9 ms | 217 ms |
| 64.000 | 199 µs | 418 ms | 1.078 ms | 482 ms | 1.509 ms | 3,5 s |
| 256.000 | 808 µs | 6,60 s | 17,74 s | 7,40 s | 24,36 s | 56,1 s |

## O que é viável

| Experimento | Alcance | Custo | Viable |
|---|---|---|---|
| Experimento completo (5 algos, 4 dists, 3 reps) | **256.000** | ~3 min | ✅ |
| Re-benchmark Rust | 64.000 | 24 s | ✅ |
| Re-benchmark C | 64.000 | 25 s | ✅ |
| Re-benchmark C++ | 64.000 | 26 s | ✅ |
| Re-benchmark **Python** | 64.000 | **~16 dias** | ❌ |

**Por que Python é inviável.** Medido a n=16.000, o Python é 55× mais lento que o Rust. Como a complexidade é quadrática, a 4× o n o custo multiplica por 16×:

```
FF em n=16.000:   Rust   25 ms   |  Python   ~1,4 s
FF em n=64.000:   Rust  400 ms   |  Python  ~22   s  → x4 dists x3 reps ≈ 4,5 h
FF em n=256.000:  Rust  6,4  s   |  Python  ~1,0   h  → x4 dists x3 reps ≈ 12 dias
```

Não é falta de tempo de máquina — é a natureza da linguagem interpretada.

## Decisão metodológica

A **comparação entre linguagens** (Tabela 2) fica até **n = 16.000**, onde as
quatro linguagens rodam. A **curva de crescimento** (Tabela 3, expoentes)
vai até **n = 256.000**, medida em Rust.

Isso é legítimo porque as duas perguntas são diferentes:

- *Tabela 2* pergunta **o quanto a linguagem muda o custo** → precisa de todas
  as linguagens, exige n moderado.
- *Tabela 3* pergunta **qual o expoente de crescimento** → é propriedade do
  algoritmo, não da linguagem. Medir em Rust basta, desde que a variante
  *bins-only* seja a mesma nos dois casos (é o que a seção sobre fair-play
  estabelece).

O método é o de **duplicação** (*doubling*): com n dobrando, uma série
exatamente O(n) multiplica o tempo por 2, e O(n²) por 4. O expoente medido
é a média dos logaritmos dessas razões — mais robusto que uma reta sobre
5 pontos.

## Ferramentas

```bash
# custo por tamanho (antes de decidir o alcance)
cargo run --release --bin custo_escala 16000 64000 256000

# experimento completo ate 256.000
./target/release/bin-packing-heuristics experiment \
    --sizes 1024,2048,4096,8192,16384,32768,65536,131072,262144 \
    --reps 3 --out dados_escala/rust_escala.csv

# ajuste da curva (duplicação + regressão)
python3 scripts/ajusta_curva.py dados_escala/rust_escala.csv

# script completo (roda tudo)
bash scripts/escala_grande.sh 256000 64000
```

---

## ⚠️ Erro cometido nesta rodada: medi em paralelo

Lancei o **re-benchmark até 64.000** e o **experimento até 256.000** ao mesmo
tempo, ambos fixados em `taskset -c 0`. Dois processos de carga competing no
mesmo núcleo físico.

O sintoma apareceu nos dados brutos: a mesma heurística, mesmo tamanho, mesma
linguagem, dava valores **bimodais** — por exemplo, o First Fit em Rust a
n = 65.536:

```
847619 µs  851048 µs  840039 µs  841262 µs | 419694 µs  421500 µs  420122 µs
└──────── ~840 mil (rodadas com competição) ────────┘└──── ~420 mil (rodadas limpas) ────┘
```

Diferença de **2× dentro da mesma condição declarada**, com coeficiente de
variação de 25% a 47%. Isso não é ruído: é contaminação.

### Consequência

O ratio `rust/C` medido a n = 65.536 (1,20 a 1,62 nos quadráticos) está
**invalidado** e **não entra no artigo**. A conclusão de que "Rust fica mais
lento que C em escala grande" foi um artefato de medição paralela.

### Regra

**Um processo por vez.** As medições de tempo são mutuamente exclusivas.
O `preflight.sh` verifica a carga do sistema antes de rodar, mas **não detecta
outro benchmark que eu mesmo tenha launched em paralelo** — essa falha é minha,
não da ferramenta.

Antes de reportar qualquer razão entre tempos, verificar:

```bash
pgrep -f "bench|experiment"   # deve retornar vazio
```


---

## ⚠️⚠️ A série principal também saiu contaminada

O erro acima não afetou apenas o re-benchmark. A série do experimento
principal (`rust_escala.csv`, tamanhos até 131.072) foi medida **em paralelo**
com o re-benchmark, ambos em `taskset -c 0`. Os coeficientes de variação
denunciam:

```
NF   uniforme_discreta_100  n=131072   CV 109,2%
FFD  uniforme_discreta_100  n=  2048   CV 105,2%
FFD  tres_particao          n=  2048   CV 109,1%
FF   tres_particao          n=  4096   CV  61,7%
FFD  tres_particao          n=  4096   CV  58,2%
```

Um CV de 109% significa que as três repetições da mesma condição diferiram
mais que o próprio valor — impossível sem interferência.

### Consequência

**A Tabela 3 do artigo, como está agora, não é confiável.** Os expoentes
publicados (0,974 · 2,025 · 2,011 …) vieram desta série contaminada.

A correção é recoletar as duas séries **sequencialmente**, uma por vez,
verificando `pgrep -f "bench|experiment"` vazio antes de cada rodada.

### Lição

O `preflight.sh` detecta carga *externa* (Firefox, Discord), mas é cego a
**outro benchmark que eu mesmo tenha lançado em paralelo**. Essa falha é
humana, não da ferramenta — e foi a segunda vez na mesma sessão que a
contaminação por paralelismo comprometeu uma medição.


---

## 📌 REGRA PERMANENTE (errei duas vezes nesta sessão)

**Nunca lance duas medições de tempo em paralelo. Uma por vez.**

`taskset -c 0` impede o processo de *migrar* de núcleo. Não impede
*competição*: dois fixados no mesmo núcleo físico se disputam o cache L1 e a
unidade de execução. O resultado são tempos ~2× maiores, alternando entre
valores conforme o escalonamento do agendador.

**Como detectar.** O sintoma é bimodal — o mesmo algoritmo, mesmo tamanho,
mesma linguagem, dá dois valores distintos conforme a rodada:

```
FF em Rust, n=65.536:
  847619 µs  851048 µs  840039 µs   <- contaminado
  419694 µs  421500 µs  420122 µs   <- limpo
```

E o coeficiente de variação denuncia: CV de 40% a 110% onde deveria ser
pouco por cento.

**Antes de medir:**

```bash
pgrep -f "bench|experiment"     # tem que voltar VAZIO
```

O `preflight.sh` detecta carga externa (Firefox, Discord) mas é **cego a
outro benchmark que eu mesmo tenha lançado** — essa parte é falha humana,
não da ferramenta.

**Filtro de aceitação:** só entram no texto os pontos com CV abaixo de 15%
no tamanho comparado. Ruído em tempos de poucos microssegundos é esperado e
não é contaminação — o critério se aplica ao tamanho que entra na tabela.


---

## ✅ Verificação de integridade dos dados (após processos antigos terminarem)

Dois processos em background que eu tinha disparado antes terminaram tarde e
avisaram. Um deles é o **re-benchmark contaminado**. Conferi por
*timestamp* se eles teriam sobrescrito a coleta limpa:

```
dados_escala/runs/*.csv   todos escritos entre 03:11 e 03:19
coleta limpa              iniciou depois, com rm -rf do diretório
```

Os arquivos são da coleta limpa — a contaminada escreve antes e foi apagada
pelo `rm -rf` do processo seguinte.

Confirmações na série atual:

| Verificação | Resultado |
|---|---|
| Identidade de bins entre Rust/C/C++ | **420 chaves, 0 divergências** |
| `rust/C` em n = 65.536 | 0,835 · 1,002 · 1,017 · 0,996 · 1,016 |
| Pontos com CV > 15% | 57/420 — todos com tempo médio de **1,6 ms** |
| Tempo médio dos pontos limpos | **80 ms** (50× maior) |
| CV máximo em n = 65.536 com tempo > 1 ms | **12,4%** |

O ruído relativo é inversamente proporcional à duração: onde o tempo é de
microssegundos, o desvio percentual é grande pela natureza do relógio. Onde o
tempo é de milissegundos — que é onde as razões entre linguagens são
medidas — o CV cai para menos de 13%.

**Conclusão: a série atual é íntegra e utilizável.**

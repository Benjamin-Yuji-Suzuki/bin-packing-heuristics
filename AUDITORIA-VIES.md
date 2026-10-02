# Auditoria de viés — código e artigo

Revisão feita procurando **viés**: favorecimento de um algoritmo, de uma
linguagem, de um resultado. Não é a busca por erro (que já tem suíte de
testes) — é a busca por tendenciosidade.

**Data:** 01/10/2026
**Escopo:** código Rust/C/C++/Python, protocolo experimental, redação do artigo.

---

## Resultado

| Aspecto | Veredito |
|---|---|
| Fair-play entre linguagens | ✅ sem viés |
| Fair-play entre heurísticas | ✅ sem viés |
| Cherry-picking no artigo | ✅ não encontrado |
| Declaração incorreta no texto | 🔴 **1 encontrada e corrigida** |

---

## ✅ O que foi verificado e está correto

### 1. As quatro linguagens fazem o mesmo trabalho

- **Rust** usa as variantes `*_bins` (`src/algorithms.rs`), que só contam
  bins, sem rastrear `item -> bin`.
- **C**, **C++** e **Python** também só contam (`bench/*.c`, `*.cpp`, `*.py`).
- Verificado no código: o `push_back` que aparece no C++ é do vetor de
  **resíduos**, não rastreamento de atribuição — as quatro linguências
  empilham resíduos igualmente.

**Nenhuma linguagem faz trabalho extra.** A versão antiga do Rust (que
rastreava atribuição) media 1,3–3× o tempo do C justamente por isso; foi
corrigida antes da medição atual.

### 2. As quatro linguagens produzem bins idênticos

- **0 divergências** em 1.000 comparações por par de linguagem
  (12 comparações pareadas × 1.000 chaves).
- Em 3 rodadas independentes × 4 linguagens: idêntico.

### 3. Desempate em Best Fit é o mesmo

O Rust usa `min` seguido de `position` (primeiro índice com o valor
mínimo); o C usa `<` estrito no laço (mantém o primeiro). **Ambos escolhem
o primeiro bin em caso de empate** — testado com a distribuição discreta
(cheia de empates): 0 divergências em 2.400 comparações.

### 4. Risco teórico descartado: ordenação estável vs instável

As versões completas usam `sort_by` (estável); as `bins-only` usam
`sort_unstable_by` (instável). Se a ordem de itens iguais mudasse, o
benchmark mediria trabalhos diferentes. **Testado explicitamente com a
distribuição discreta (muitos empates): 0 divergências em 2.400
comparações** — o empacotamento é o mesmo apesar da ordenação instável.

### 5. Protocolo sem favorecer nenhuma heurística

- A **mesma instância** é gerada uma vez e passada às 5 heurísticas
  (`src/experiment.rs` gera `items` fora do laço de algoritmos).
- O limitador **L2 é calculado uma vez por instância**, não por
  heurística — o denominador é idêntico para todas.
- O **tempo medido exclui** o cálculo do L2 (mede só `run_algorithm`).
- As 4 distribuições têm o **mesmo número** de combinações (n, rep): 50
  cada.

### 6. Sem cherry-picking no artigo

- Tabela 2: **mediana** de 3 execuções (não o melhor caso).
- Tabela 3: expoente por algoritmo **e** por distribuição (não só a média).
- Tabela 4: média sobre as 4 distribuições, **declarada no subtítulo**.
- O resultado **negativo** (não reproduzir o pior caso de Johnson) tem
  destaque igual aos positivos, com a mesma estrutura de linguagem.

---

## 🔴 O que foi encontrado

### Declaração incorreta: FFD e BFD não coincidem "exatamente"

**O que o artigo dizia:**

> "As curvas do FFD e do BFD coincidem exatamente nestas distribuições:
> ambos produzem o mesmo número de bins."

**O que a auditoria mediu:** em **199 das 200** configurações medidas,
coincidem. Em **1** (uniforme contínua, n = 8.000, repetição 4), o **BFD
abre 3977 bins e o FFD abre 3978** — o BFD usa **um bin a menos**.

**Por que isso importa e não é trivial:** as duas heurísticas têm a
**mesma garantia** (11/9·OPT + 6/9). Garantia idêntica não implica que uma
seja pior em toda instância — e este é justamente o tipo de contraexemplo
que reforça o argumento do artigo contra o contraexemplo FFD > FF.

**Correção aplicada:** o texto agora diz "em 199 das 200 configurações" e
explica a exceção. Teste de regressão adicionado
(`bfd_pode_usar_um_bin_a_menos`) para que o caso não suma em silêncio.

---

## ⚠️ Riscos remanescentes (declarados, não corrigidos)

### 1. Série experimental dupla — ✅ RESOLVIDO

**O que era:** o experimento principal (`experiment.rs`) usava a versão
**com** rastreamento `item -> bin`; o benchmark entre linguagens usava as
variantes `bins-only`. A Tabela 2 (tempos entre linguagens) vinha de uma
série e a Tabela 3 (expoentes) da outra — e comparar as duas seria
ilegítimo, o mesmo erro dos múltiplos de 170×/487×.

**O que foi feito:** cada execução agora mede **as duas séries na mesma
passagem**, sobre a mesma instância. O CSV ganhou a coluna
`tempo_bins_us` ao lado de `tempo_us`, e um `debug_assert` verifica a cada
execução que as duas contagens de bins coincidem. A Tabela 3 e o gráfico
`grafico_tempo_n.png` passaram a usar a série `bins-only` — a mesma da
Tabela 2.

**O que isso mudou nos números:** o expoente do NF caiu de 1,17 para
0,92–1,04 (a série com rastreamento inflava o expoente linear, porque o
custo de registrar a atribuição cresce com $n$). O fator de crescimento
do NF caiu de 24× para 15×, contra os 16× previstos para crescimento
linear. **A série `bins-only` é a medida mais próxima da prevista, e por
isso é a que o artigo agora reporta.**

### 2. Carga do sistema durante as medições — ✅ MEDIDO

Afirmava-se que "os tempos absolutos variam até 75% com a carga". Essa
causalidade **não estava provada** — o registro de carga usava um parser
que quebrava a vírgula decimal do locale pt-BR (`1,75` virava `1` e `75`),
então os valores de carga gravados eram lixo.

**Medição direta** (`src/bin/mede_carga.rs`, 20 medições independentes,
n = 16.000, série bins-only):

| Heurística | média | desvio | CV | máx/mín |
|---|---|---|---|---|
| **NF** | 13 ns | 0 ns | **1,71%** | **1,08×** |
| **FF** | 25.393 ms | 2.454 ms | **9,66%** | **1,33×** |

**A hipótese original estava ERRADA.** Eu havia declarado que o NF era o
algoritmo mais sensível por rodar abaixo de 100 µs. O medido é o
oposto: o NF é o **mais estável** (roda em microssegundos — um
preempção durante a medição é improvável, CV de 1,7%) e o FF é o **menos
estável** (roda em dezenas de milissegundos e sofre preempção várias
vezes, CV de 9,7%).

**A consequência prática é mais séria do que se pensava.** O múltiplo
FF/NF calculado nas 20 medições varyeu de **1.744.000× a 2.495.000×** —
uma faixa de 1,4× entre o mínimo e o máximo. Ou seja, o "706×" publicado
não é um número exato: é uma razão entre duas grandezas de duração, e o
denominador (FF) é o termo instável. **A afirmação correta é que a razão
é da ordem de centenas a milhares de vezes, medida sob condição específica
de máquina ociosa — e não um valor puntual.**

### 3. Ratios entre linguagens — ✅ ROBUSTO

Os **ratios entre linguagens medindo a mesma heurística** permanecem
estáveis dentro de ~1% (medido em três sessões independentes: 0,998 /
0,998 / 1,005 no FF). Isso funciona porque o erro de preempção cai no
numerador **e** no denominador, e se cancela. É por isso que a comparação
multi-linguagem é confiável e o múltiplo entre algoritmos de durações
diferentes não é.

---

## Como repetir esta auditoria

```bash
# bins das 4 linguagens coincidem?
bash scripts/rebench.sh && /usr/bin/python3 scripts/consolida_bench.py

# variantes bins-only == completas (inclusive com empates)
cargo test --lib variantes_bins_only

# o contraexemplo do BFD
cargo test --lib bfd_pode_usar_um_bin_a_menos
```

Os três são automatizáveis e devem rodar em qualquer alteração que
toque em heurística, gerador ou benchmark.
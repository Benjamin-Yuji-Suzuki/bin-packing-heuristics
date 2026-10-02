# Diagnóstico do artigo — 02/10/2026

Auditoria completa (texto × código × dados × fontes). Gerado em sessão
independente, sem confiar no `HANDOFF-2026-10-02.md` (descartado a pedido
do autor por ter origem num glitch de outra sessão).

**Método:** toda fonte da verdade é re-derivada do CSV/binário, nunca de
transcrição. Para os tempos entre linguagens, a fonte da verdade é
`scripts/consolida_bench.py` (mediana de 3 rodadas de
`bench/runs/{lang}_{1,2,3}.csv`) — **não** a mediana das 10 reps de
`bench_*.csv`, que é outra série.

**Evidência preservada antes de qualquer escrita:**
`~/.hermes/cache/scratch/bppaudit_evidencia_20261002_130404/` com md5 dos
12 CSVs de `bench/runs/`.

---

## Resumo executivo

| # | Severidade | Achado | Afeta o artigo? |
|---|---|---|---|
| 1 | 🔴 Crítico | 16 de 20 tempos da Tabela 1 não reproduzem do CSV | **Sim** |
| 2 | 🔴 Crítico | Razão do abstract (494×) não fecha nem com a própria tabela | **Sim** |
| 3 | 🔴 Crítico | `optimal_bins` devolve `OPT+1` silenciosamente | **Sim, indireto** |
| 4 | 🔴 Crítico | `optimal_bins` nunca devolve `None` (doc mente) | **Sim, indireto** |
| 5 | 🟡 Médio | `L1 = ceil(soma)` pode exceder o ótimo (limitador inválido) | **Não** (verificado) |
| 6 | 🟡 Médio | Pânico em `NaN` em 3 pontos; séries divergem em comportamento | Não (latente) |
| 7 | 🟡 Médio | Falkenauer declara média 0,275; real é 0,300 | **Sim** |
| 8 | 🟡 Médio | Artigo afirma superestimação "0,15%–4,7%"; medido 0%–50% | **Sim** |

Testes: **33/33 passam** (`cargo test --release`). `confere_afirmacoes`:
**15/15 confirmadas**. Esses dois resultados **não invalidam** os achados
abaixo — os testes não cobrem nenhum dos casos que quebram.

---

## 🔴 1 — Tabela 1: 16 de 20 tempos não reproduzem

Mediana das 3 rodadas em `bench/runs/`, n=16.000, `uniforme_discreta_100`:

| Alg | Lang | Artigo | REAL | Desvio |
|---|---|---:|---:|---:|
| NF | rust | 51 | **37,0** | **−27,45%** |
| NF | c | 70 | 72,3 | +3,29% |
| NF | cpp | 64 | 63,5 | −0,78% |
| NF | python | 530 | 533,5 | +0,67% |
| FF | rust | 24.889 | 25.069,5 | +0,73% |
| FF | c | 24.929 | 24.949,5 | +0,08% |
| FF | cpp | 24.768 | 25.316,5 | +2,21% |
| FF | python | 1.369.383 | 1.396.099,8 | +1,95% |
| BF | rust | 30.988 | 31.205,5 | +0,70% |
| BF | c | 34.090 | 34.232,0 | +0,42% |
| BF | cpp | 35.653 | 36.589,0 | +2,63% |
| BF | python | 1.743.061 | 1.781.990,0 | +2,23% |
| FFD | rust | 27.148 | 27.281,0 | +0,49% |
| FFD | c | 27.987 | 28.056,2 | +0,25% |
| FFD | cpp | 27.298 | 28.125,5 | **+3,03%** |
| FFD | python | 1.483.583 | 1.512.687,6 | +1,96% |
| BFD | rust | 41.308 | 41.602,5 | +0,71% |
| BFD | c | 41.934 | 42.754,2 | +1,96% |
| BFD | cpp | 41.877 | 43.614,5 | **+4,15%** |
| BFD | python | 2.538.086 | 2.596.247,0 | +2,29% |

**Padrão:** quase todos os desvios são **positivos** (CSV mais lento que o
publicado), exceto o NF/Rust que é −27%. Não é ruído de medição: ruído
seria simétrico em torno de zero.

**O NF/Rust é o pior caso e o mais importante:** sendo a heurística mais
barata, é a que define a razão grande do artigo.

### Hipóteses (não confirmada — depende do autor)

- **H1:** a tabela veio de uma 4ª rodada sobrescrita nos CSVs. Não
  verificável pelo conteúdo dos arquivos.
- **H2:** a tabela foi editada direto no `.tex`.

### Indício que favorece "texto certo, tabela errada"

A Seção 4.6 (linha 190 do `.tex`) diz *"de 38 µs a 2,5 s"*. O **38 µs**
bate com o CSV real (37,0), **não** com a tabela (51). O texto e os dados
concordam; a tabela destoa. Indício de que a tabela foi alterada depois
do texto.

---

## 🔴 2 — A razão do abstract não fecha

O abstract e a Introdução afirmam **NF 494× mais rápido que FF**.

| origem | NF | FF | razão |
|---|---:|---:|---:|
| Tabela 1 (artigo) | 51 | 24.889 | 488× |
| CSV real | 37,0 | 25.069,5 | **678×** |

Dois problemas:
1. **488 ≠ 494** — o número não fecha nem com a própria tabela do artigo.
2. Com dados corrigidos, vira **678×** (≈2,8 ordens de grandeza, não 3).

---

## 🔴 3 e 4 — `optimal_bins` mente (`src/exact.rs`)

### 3. Devolve `OPT+1` como se fosse ótimo

Com orçamento de 1 µs, em 80 instâncias:

```
devolvido == OTIMO : 74
devolvido  > OTIMO : 6   <-- VALOR SUBÓTIMO REPORTADO COMO ÓTIMO
devolvido  < OTIMO : 0   (impossível: seria bug de poda)

uniforme_continua n=12 seed=2: devolvido=5  OTIMO=4  FFD=5
tres_particao     n=8  seed=2: devolvido=4  OTIMO=3  FFD=4
tres_particao     n=12 seed=0: devolvido=6  OTIMO=5  FFD=6
tres_particao     n=14 seed=3: devolvido=6  OTIMO=5  FFD=6
falkenauer_u120   n=10 seed=0: devolvido=4  OTIMO=3  FFD=4
falkenauer_u120   n=14 seed=0: devolvido=6  OTIMO=5  FFD=6
```

Em todos os 6, o valor devolvido é exatamente o incumbent do FFD — isto é,
quando o orçamento estoura, o solver **recede para a heurística e devolve
o resultado dela como se fosse o ótimo**.

### 4. Nunca devolve `None`, contra o próprio doc

```
budget=0     -> Some(17) em 13 µs | OPT real=16 | FFD=17
budget=1     -> Some(17) em 10 µs | OPT real=16 | FFD=17
budget=100   -> Some(17) em 101 µs | OPT real=16 | FFD=17
```

Com **orçamento zero** devolve `Some(17)` sabendo que o ótimo é 16.
O doc de `exact.rs:18-19` afirma *"Retorna `None` se o orçamento de tempo
estourar"*.

Consequência: `piorcasa.rs:108` e `main.rs:387` (`if let None`) são
**código morto**.

**Por que afeta o artigo:** `optimal_bins` é o `OPT` contra o qual o
contraxemplo da Seção 5 (ordenação piora o First Fit) e o pior caso são
validados. Se o `OPT` pode ser `OPT+1`, então (a) a razão `A(I)/OPT`
superestima a qualidade medida e (b) o `L2` pode parecer maior que o
ótimo real, o que inverteria o argumento de "razões conservadoras".

**Teste existente que não pega:** `exact::tests::bnb_bate_com_busca_sem_podas`
passa porque usa orçamento generoso — com tempo suficiente o B&B converge
e acerta. O bug só morde quando o orçamento estoura.

---

## 🟡 5 — Limitador L1 pode exceder o ótimo (`algorithms.rs:214`)

```
k=20   soma=1.00000000000000022204  L1=2  FFD=1  OPT=1  <<< LIMITADOR INVÁLIDO
k=100  soma=1.00000000000000066613  L1=2  FFD=1  OPT=1  <<< LIMITADOR INVÁLIDO
k=1000 soma=1.00000000000000066613  L1=2  FFD=1  OPT=1  <<< LIMITADOR INVÁLIDO
```

`L1 = ceil(soma)` exato, sem tolerância. Em `f64`, `k` itens de `1/k`
somam um valor ligeiramente **acima** de 1 quando `1/k` não é
representável. O empacotador aceita `x <= residual + 1e-9`, então fecha
em 1 bin; o `ceil` diz 2. Um limitador **acima** do ótimo tornaria a razão
`A/L2` menor que 1 e mataria o argumento de "conservadora".

### Impacto verificado: nenhum

`resultados.csv` → coluna `razao_vs_lb`, **min = 1,0000**, **zero linhas
abaixo de 1,0**. O experimento usa `n ≥ 1000` com itens sorteados, então
o padrão `k × (1/k)` não ocorre.

**Risco residual:** a afirmação *"as razões reportadas são conservadoras"*
(Seção 4.3) é condicional ao gerador nunca produzir esse caso. Se
perguntado, a resposta honesta é "o gerador não gera", não "o limitador é
sempre válido".

---

## 🟡 6 — Pânico em `NaN`, e as duas séries divergem

```
FFD (algorithms.rs:107)  -> *** PÂNICO ***  (Option::unwrap() em None)
BFD (algorithms.rs:114)  -> *** PÂNICO ***
OPT (exact.rs:38)        -> *** PÂNICO ***
```

`partial_cmp()` devolve `None` diante de `NaN`, e o `.unwrap()` aborta.

Pior: as variantes **bins-only** usam `total_cmp`
(`algorithms.rs:192,198`) e **não** entram em pânico. Ou seja, **as duas
séries temporais do artigo divergem em comportamento** — a série completa
pode explodir num caso degenerado onde a bins-only roda. Isso é relevante
porque o artigo reporta a bins-only como a série comparável.

Latente: o gerador do artigo não produz `NaN`.

---

## 🟡 7 — Falkenauer: média declarada errada

```
falkenauer_u120  suporte [10,50]/100  média TEÓRICA=0,3000  MEDIDA=0,2995 (160.000 amostras)
DECLARADO no código (generators.rs:13 e :55): 0,275  -> DIVERGE
```

O artigo repete o valor errado em §4.2: *"U{1/10,…,1/2} (média 0.275)"*.
A média real de `U{10..50}/100` é **0,300**. É a citação que o próprio
artigo usa para justificar a distribuição.

---

## 🟡 8 — Superestimação A/L2: artigo afirma teto que a medição passa

`mede_l2`, 30.000 amostras:

```
                    min     média    máximo
TODAS            0,00%     1,67%    50,00%

O ARTIGO AFIRMA: "entre 0,15% e 4,7%"
```

O máximo real é **50%**, mais de 10× o teto declarado.

**Contexto que reduz o escopo:** o `max=50%` ocorre em **n=6**; cai para
33%, 25%, 20%, 20% conforme n cresce, e a própria ferramenta anota *"em n
grande a superestimação tende a 0"*. As tabelas do artigo são em
**n=16.000**. Então o 50% é artefato de instância minúscula.

**O problema é de redactação, não de número:** o artigo não qualifica por
`n`, então o teto declarado é lido como valendo para todas as instâncias.
Correção: declarar a faixa por faixa de `n`, ou declarar a média e o
limite de `n` da medição.

---

## ✅ Verificado como correto

- **Identidade de bins entre linguagens:** 0 divergências em 12 arquivos
  (4 linguagens × 3 rodadas, 1.000 chaves cada).
- **Gerador determinístico:** 60/60 gerações idênticas na mesma semente;
  sementes 7 e 8 produzem instâncias distintas. Confirma reprodutibilidade.
- **Poda (3) do B&B:** válida — 0 divergências vs busca exata sem poda.
- **`best_fit` escolhe de fato o menor resíduo** (não é FF disfarçado).
- **`lower_bound = 0`** só em instância vazia; mascarado por `.max(1)`,
  0/400 casos no experimento.
- **`next_fit([])` = 1 vs FF/BF = 0:** assimetria real, invisível com
  `n ≥ 1000`. Não afeta.

---

## Decisões pendentes do autor

1. **Tabela 1** — re-medir (`rebench.sh`, ~65 min, gate aprovado) ou
   reconstruir de outra fonte? *Recomendação: re-medir.*
2. **Abstract** — corrigir 494× → 678× (ou o que a nova medição der) e
   reavaliar "três ordens de grandeza" (678× ≈ 2,8).
3. **L1 inválido** — corrigir o `ceil` com tolerância (barato, mas
   regenera tabelas) ou documentar a limitação? *Recomendação:
   documentar; impacto verificado zero.*
4. **`exact.rs`** — corrigir para devolver `None` de verdade, ou
   declarar a limitação? *Recomendação: corrigir o código (poucas linhas),
   mas só depois de salvar a Tabela 1.*
5. **Falkenauer 0,275 → 0,300** no código e no §4.2.
6. **Superestimação** — requalificar por `n` em vez de declarar 4,7%.
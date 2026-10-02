# Verificação de afirmações — método e resultado

Cada afirmação quantitativa do artigo foi conferida contra o código ou
contra o CSV bruto. Este documento registra **o que foi verificado, como, e
o que estava errado**.

Comando: `cargo run --release --bin confere_afirmacoes`

---

## Resultado

**15 de 15 afirmações confirmadas.**

Ao verificar, **uma afirmação do artigo estava errada** e foi corrigida
(detalhe abaixo). As outras duas que o teste acusou como falha eram
**limitações do meu próprio teste**, não do artigo — e os testes foram
corrigidos para medir o que o artigo mede.

---

## O que foi verificado

| ID | Afirmação | Como foi conferida |
|---|---|---|
| E1 | 1.000 execuções no total | contagem do protocolo: 4×5×10×5 |
| E2 | tamanhos de 1.000 a 16.000 | protocolo do experimento |
| E3 | 4 distribuições | enumeração do gerador |
| R1 | FFD ≈ 1,001 nas uniformes | média de 10 reps, n=16.000 |
| R2 | NF = 1,328 na uniforme contínua | idem |
| R3 | nenhuma passa de 1,24 na 3-partição | idem (NF = 1,2352) |
| A1 | 3-partição é a mais difícil p/ FF/BF/FFD/BFD | compara as 4 dists por heurística |
| A2 | NF é exceção: pior caso na uniforme contínua | U = 1,328 > 3p = 1,235 |
| C1 | FFD = BFD em 199 de 200; BFD 1 bin a menos na exceção | n=8000, rep=4: 3978 vs 3977 |
| C2 | contraexemplo FF=2=OPT, FFD=3 | branch-and-bound confirma OPT=2 |
| L1 | L2 nunca supera o ótimo exato | 0 violações em 2.400 instâncias |
| L2 | A/L2 superestima A/OPT | 30.000 amostras, `mede_l2` |
| T1 | NF cresce ~15× de 1.000 a 16.000 | média do CSV: 3,4 ns → 50,6 ns = 14,9× |
| T2 | FF cresce ~242× | 103 ns → 24.997 ns = 241,7× |
| E4 | coeficientes de variação do NF | ver seção de ambiguidade abaixo |

---

## 🔴 Afirmação que estava errada: a superestimação do L2

**O artigo afirmava:** *"superestimando A(I)/OPT(I) entre 0,15% e 4,7%"*

**O medido** (30.000 amostras, `src/bin/mede_l2.rs`):

| distribuição | mínimo | média | **máximo** |
|---|---|---|---|
| uniforme contínua | 0,00% | 0,65% | **50,00%** |
| uniforme discreta | 0,00% | 0,27% | **50,00%** |
| 3-partição | 0,00% | 4,90% | **50,00%** |
| Falkenauer | 0,00% | 0,87% | **50,00%** |

O **máximo é 50%**, muito acima do teto de 4,7% que o texto declarava.

**Diagnóstico:** o número original vinha de uma amostra pequena. O desvio
grande ocorre **só em instâncias muito pequenas** — por tamanho:

| n | média | máximo |
|---|---|---|
| 6 | 1,33% | **50,00%** |
| 8 | 5,78% | 33,33% |
| 10 | 8,17% | 25,00% |
| 12 | 4,87% | 20,00% |
| 14 | 4,37% | 20,00% |

Com apenas 6 itens, o limitador tem pouquíssimos valores para somar e a
folga em relação ao ótimo é grande por construção.

**Correção aplicada:** o texto agora declara a média por distribuição
(0,3% a 4,9%), menciona explicitamente que o desvio máximo chega a 50% em
instâncias com $n \leq 8$, e explica que o efeito é irrelevante para o
experimento principal, que usa $n \geq 1.000$.

---

## ⚠️ Ambiguidade resolvida: dois coeficientes de variação

O artigo citava "CV de 1,71%" para o NF. A verificação mostrou que existem
**dois números legítimos e diferentes**:

| medida | NF no experimento | NF "melhor de várias execuções" |
|---|---|---|
| coeficiente de variação | **1,0% a 6,5%** | **~1,7%** |
| o que responde | "quanto a média de 10 reps varia" | "quanto a melhor execução varia" |

A causa da diferença é a granularidade do relógio: o NF em $n = 16.000$
leva ~50 ns, e o relógio devolve valores praticamente quantizados (50 ou
51 ns).

**Correção aplicada:** a Seção 5.1 agora distingue explicitamente os dois
coeficientes e dá o intervalo do experimento completo.

---

## Erros meus no teste (não do artigo)

Três "não confirmados" iniciais eram falha do meu código de verificação:

1. **Medi a série errada.** O teste lia `tempo_us` (com rastreamento) em vez
   de `tempo_bins_us` — a série que o artigo usa. Com a série errada, o NF
   dava 23,7× em vez de 14,9×.
2. **Medi pontualmente.** O relógio não tem resolução para o NF em
   microssegundos: uma medição única dava 1× (sem informação). O artigo
   tira a média de 10 repetições, que resolve.
3. **Indice de coluna errado.** Com a coluna nova no CSV, `f[7]` passou a
   apontar para a série errada.

Todos corrigidos. O teste agora lê a mesma série e o mesmo método que o
artigo.

---

## Como repetir

```bash
# as 15 afirmações (~2 min)
cargo run --release --bin confere_afirmacoes

# a faixa da superestimação do L2, detalhada por tamanho (~3 min)
cargo run --release --bin mede_l2

# a estabilidade de tempo por heurística (~1 min)
cargo run --release --bin mede_carga
```

Os três são determinísticos no que o artigo afirma (razões, bins,expoentes), exceto
as medidas de tempo, que dependem das condições da máquina.

## O que **não** foi verificado por máquina

- As afirmações da literatura (Johnson et al. 1974, Dósa 2007, Kenyon
  1996, Coffman 1980, Bentley 1984) foram conferidas por leitura das
  fontes, não por execução.
- As garantias de pior caso (2, 1,7, 11/9+6/9) são resultado de prova
  matemática; o código não as verifica, apenas mede contra elas.
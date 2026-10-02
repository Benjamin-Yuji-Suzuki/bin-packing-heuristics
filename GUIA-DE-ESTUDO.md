# Guia de Estudo — Bin Packing (CESUPA, Análise e Projeto de Algoritmos)

**Data:** 02/10/2026 · **Artigo:** `artigo/artigo-parcial.pdf` (22 páginas)

Este guia existe para você **entender o que o artigo faz**, não decorar a fala.
Se você conseguir explicar a alguém por que o artigo existe, está pronto.

---

## 0. O resumo em uma frase

> As heurísticas clássicas de bin packing têm garantias de pior caso conhecidas
> (NF ≤ 2·OPT, FFD ≤ 11/9·OPT + 6/9), mas essas garantias são **tolos de
> verdade**. Este artigo mede **quanto longe** elas estão do comportamento típico,
> em dois eixos — **qualidade** (razão de aproximação) e **custo** (tempo) — e
> mostra que **o custo separa as heurísticas muito mais que a qualidade**.

Se você só levar uma frase do artigo, é essa.

---

## 1. Antes de tudo: o que é o problema

**Bin packing unidimensional.** Você tem uma lista de itens de tamanhos
`L = (s₁, s₂, ..., sₙ)`, cada um com `sᵢ ∈ (0, 1]`. Cada item precisa caber
inteiro dentro de um "recipiente" (bin) de capacidade 1. Você quer o **menor
número de bins**.

Exemplo concreto do artigo: itens de `0,5, 0,3, 0,7, 0,2, 0,9, 0,1`

|algoritmo | bins abertos |
|---|---|
| Next Fit | 3 |
| First Fit | 3 |
| Best Fit | 3 |
| FFD | 3 |

**Três termos que você precisa saber:**

- **OPT** — o número mínimo de bins. É o **melhor possível**. Você quase nunca
  consegue calcular (é NP-difícil — item 2).
- **garantia de pior caso** — um teto que a heurística **nunca** ultrapassa.
  Ex.: FFD nunca passa de `11/9 · OPT + 6/9`. Se `OPT = 9`, FFD ≤ `11 + 0,67`,
  ou seja ≤ 11,67 → no máximo 11 bins.
- **razão de aproximação** — `A(I) / OPT(I)`. Se a heurística abriu 11 bins
  e o ótimo é 9, a razão é `11/9 ≈ 1,22`.

### 1.1 A tensão que o artigo explora

A garantia diz: *"nunca pior que 1,22"*. Mas essa é uma **garantia de
pior caso** — o pior caso é uma instância **construída à mão** para humilhar
o algoritmo. Em instâncias que aparecem na prática, é muito mais perto de 1,00.

**É essa distância entre "o quão ruim pode ser" e "o quão ruim é em média" que
o artigo mede.** E a distância é grande.

---

## 2. Por que OPT é bonito na teoria e intocável na prática

O artigo prova que o problema é **NP-completo** (Teorema 1, §2.2) por redução
a partir do **3-PARTITION**. Esquece os detalhes da prova; o que importa:

> Se você conseguisse calcular OPT rápido, você resolveria um problema NP-difícil
> rápido. Isso provavelmente é impossível (P ≠ NP).

Por isso o artigo usa **heurísticas**: regras simples, sem garantia de ótimo,
mas que empacotam cada item assim que ele chega. E usa um **branch-and-bound
(B&B)** — busca exata que só roda em instâncias pequenas.

---

## 3. As cinco heurísticas (o núcleo do artigo)

Todas mantêm uma lista de bins abertos e, para cada item, decidem **onde
colocá-lo**. Nenhuma reabre decisão: é o que as torna `O(n)` ou `O(n²)`.

| Heurística | Regra (em português de gente) | Tempo | Garantia |
|---|---|---|---|
| **NF** — Next Fit | "abre um bin. se o próximo item não couber, fecha esse bin e abre outro. nunca volta atrás." | `O(n)` | `2·OPT` |
| **FF** — First Fit | "abre um bin novo só quando o item não cabe em **nenhum** dos abertos. senão, coloca no **primeiro** que couber." | `O(n²)` | `1,7·OPT` |
| **BF** — Best Fit | "igual ao FF, mas coloca no bin com **menor espaço sobrando** (o que mais 'estica')." | `O(n²)` | `1,7·OPT` |
| **FFD** — First Fit Decreasing | "ordena os itens do **maior para o menor**, e roda o FF." | `O(n²)` | `11/9·OPT + 6/9` |
| **BFD** — Best Fit Decreasing | "ordena do maior para o menor, e roda o BF." | `O(n²)` | `11/9·OPT + 6/9` |

**As garantias dizem o seguinte:**
- NF é a mais fraca (2·OPT): pode dobrar. É o pior caso trivial de qualquer
  heurística que mantém só um bin aberto — perde metade da capacidade do último.
- FF e BF são intermediárias (1,7).
- FFD e BFD são as melhores (`11/9 ≈ 1,22`, mais uma constante pequena).

**O ponto crucial:** essas garantias são *tetos*. O artigo vai mostrar que, nas
instâncias que ele gerou, FFD quase nunca chega perto do teto — fica em
`≈1,001`.

---

## 4. O que o artigo mede (e por que dois eixos)

O objetivo geral (§1) é **quantificar a distância** entre a garantia de pior
caso e o comportamento real, em dois eixos:

**Eixo 1 — Qualidade:** a razão `A(I)/L2(I)`, onde `L2` é um **limite
inferior** (o melhor palpite de quanto o optimum seria). Mede "quantos bins a
heurística abre a mais que o mínimo".

**Eixo 2 — Custo:** o tempo de execução em microssegundos. Mede "quanto
computador cada heurística gasta".

**Por que dois eixos?** Porque o artigo quer responder: *"se eu vou escolher uma
heurística para meu sistema, por quê eu escolheria essa?"* A respostarpóxima,
diz o artigo, é que **o custo importa mais que a qualidade** — o NF é centenas
de vezes mais rápido que o FF, e a diferença de qualidade é minúscula.

**Três objetivos específicos que aparecem no §1:**
1. Implementar as cinco em Rust, **sem estruturas auxiliares** (isso importa —
   §4.1: a complexidade medida é a das estruturas escolhidas).
2. Implementar o `L2` de Martello–Toth como denominador da razão.
3. Rodar um experimento **4 distribuições × 5 tamanhos × 10 repetições** =
   **1.000 execuções**.
4. Ajustar a curva de tempo e comparar com a previsão assintótica.
5. Registrar **contraexemplos** — casos onde a heurística "esperada" falha.

---

## 5. As quatro distribuições de instâncias (§4.2)

O artigo não usa só números aleatórios; usa quatro "modos de gerar tamanho de
item", porque a dificuldade muda muito:

| Distribuição | Itens | Comment |
|---|---|---|
| `uniforme_continua` | qualquer valor em `[0, 1]` | contínua — muitos itens minúsculos. |
| `uniforme_discreta_100` | `1/100, 2/100, …, 100/100` | como a anterior, mas "arredondada". |
| `tres_particao` | `26/100` a `50/100` | **todos os itens em (1/4, 1/2]**. Sem item pequeno = sem "argamassa" = é a mais difícil. |
| `falkenauer_u120` | `10/100` a `50/100` | o benchmark clássico da literatura (média real `0,300`). |

**A intuição da `tres_particao`:** se todos os itens têm tamanho entre 0,25 e
0,50, cada bin leva no máximo 3 itens (4 × 0,25 = 1). Não sobram "sobras
pequenas" para encaixar. É como uma tetris — tudo do mesmo tamanho. Difícil.

**Detalhe de honestidade:** o gerador `tres_particao` **não é** a instância
3-PARTITION da prova de NP-dureza. A 3-PARTITION exige que os itens somem
exatamente `m` (para forçar 3 por bin). O artigo mediu: **0 de 150** instâncias
têm soma inteira múltipla de 3. O que a distribuição garante é só a
dificuldade, não a 3 por bin. Isso está escrito no artigo (§4.2).

---

## 6. A Tabela 1 — o resultado central (leia com calma)

Esta é a tabela que sustenta o artigo inteiro. Tem **duas metades** que
respondem perguntas diferentes.

```
                    QUALIDADE (A/L2)              TEMPO por linguagem (us)
                 U[0,1] U{1/100} 3-part Falk    Rust       C      C++     Python
NF                1.328   1.323   1.235  1.198     39       70      64       530
FF                1.021   1.017   1.131  1.032  24.889  24.929  24.768  1.369.383
BF                1.011   1.009   1.131  1.031  30.988  34.090  35.653  1.743.061
FFD               1.001   1.000   1.106  1.014  27.148  27.987  27.298  1.483.583
BFD               1.001   1.000   1.106  1.014  41.308  41.934  41.877  2.538.086
```

*(n = 16.000, distribuicao uniforme discreta)*

### 6.1 Lendo a metade esquerda (QUALIDADE)

Cada celula e `razao = bins abertos / limite inferior L2`.

- `1.000` (FFD nas uniformes) = a heuristica atingiu o limitador. Como
  `L2 <= OPT`, **atingiu o proprio otimo**.
- `1.328` (NF na uniforme) = o NF abriu **32% a mais** bins que o minimo.

**O que voce observa:** a qualidade varia de `1.00` a `1.33`. Nao ha diferenca
significativa entre FFD (1.001) e BF (1.011). O NF e a excecao (1.328) — e e o
unico que e rapido.

### 6.2 Lendo a metade direita (CUSTO)

Tempo em microssegundos para empacotar **16.000 itens**:

- NF (Rust): **39 us** — quase instantaneo.
- FF (Rust): **24.889 us** = 25 ms. **640 vezes mais lento** que o NF.
- Python (BFD): **2.538.086 us** = 2,5 segundos. **65 vezes mais lento** que o
  Rust na mesma tarefa.

**O que voce observa:** o tempo varia em **tres ordens de grandeza** (de 39 us
a 2,5 s). A qualidade varia em `1.00-1.33`.

### 6.3 A assimetria (o argumento do artigo)

> A qualidade varia **pouco** entre as heuristicas. O tempo varia em **ordens
> de grandeza**.

Se voce esta escolhendo um algoritmo para usar amanha, e o NF e 640x mais
rapido, voce escolhe o NF — a menos que a qualidade seja *tao* diferente que
compense. Mas a qualidade do NF (1.328) e pior, nao melhor. **O artigo mostra
que na pratica, custo domina a escolha.**

---

## 7. A razao do NF vs FF: 640x (e por que nao e 494x)

O artigo diz que o NF e **640x mais rapido que o FF** (e **1.059x** que o BFD).

**Esse numero ja foi 494x e hoje e 640x.** A historia:

1. **Antigamente** o artigo cruzava duas series temporais diferentes — um erro
   metodologico (ver §8). Isso produzia 494x.
2. **A correcao**: o NF roda em ~40 us, tao rapido que o relogio de parede nao
   o mede com precisao (os valores iam de 12 a 55 us na mesma configuracao).
   Dividir a mediana de uma serie pela mediana de outra herda toda essa
   instabilidade.
3. **A solucao**: **parear por repeticao**. Em vez de dividir medianas de
   series separadas, mede-se a razao **na mesma repeticao** (mesma instancia,
   mesma execucao). O erro de preempcao atinge numerador e denominador e se
   cancela.
   - Razao por series separadas: banda de ~5x (instavel).
   - Razao pareada: banda de **1,06x** (confiavel).

**Se perguntarem "por que 640x e nao 494x?"**, a resposta e: *"Porque medimos a
razao dentro da mesma repeticao, e nao cruzando series — o NF e rapido demais
para o relogio resolver."* Esta na §4.3.

---

## 8. As duas series temporais (o erro metodologico mais importante)

O artigo mede **duas coisas** para cada heuristica, na mesma instancia:

- **Serie completa** (`tempo_us`): registra **em qual bin cada item caiu**.
  Custa mais (tem que guardar a atribuicao).
- **Serie bins-only** (`tempo_bins_us`): **apenas conta os bins**. Nao registra
  a atribuicao.

**So a serie bins-only e comparavel** com o C, C++ e Python, porque essas
linguagens (na versao reimplementada para o artigo) tambem so contam bins.

**Erro que o artigo ja cometeu e corrigiu:** cruzou as duas series e produziu um
numero errado. Aparece na §5 (linha 305 do `.tex`): dizia que o NF paga "2,9x
pelo rastreio (de 51 para 149 us)", mas os numeros reais eram `2,65x` (de
147,5 para 391,5 us). O `51` vinha de uma serie, o `149` de outra.

**Se perguntarem sobre isso**, o artigo assume explicitamente o erro e explica
por que as series sao separadas. E um ponto forte, nao uma fraqueza.


---

---

## 9. Os outros experimentos (o que mais aparece no artigo)

### 9.1 Expoentes de crescimento (§4.7, Tabela 3)

O artigo ajusta `log(tempo) = b0 + b1 * log(n)` e compara `b1` com a teoria:

| Heuristica | b1 medido | Teoria | Significado |
|---|---|---|---|
| NF | 0,966-0,999 | 1 | linear, bate |
| FF | 1,941-1,995 | 2 | quadratico, bate |
| BF | 1,945-1,963 | 2 | quadratico, bate |
| FFD | 1,920-1,962 | 2 | quadratico, bate |
| BFD | 1,928-1,962 | 2 | quadratico, bate |

**O que isso prova:** o comportamento empirico bate com a analise
assintotica. O NF e de fato `O(n)`; os quadraticos sao de fato `O(n^2)`.
Medidos em **nove tamanhos** de `n = 1.024` a `n = 256.000` (fator 250x).

**Detalhe importante:** o artigo diz que ampliar a faixa de escala foi
"decisivo". Numa faixa estreita (1.000 a 16.000, so 16x), os desvios chegavam
a 12%. Com 250x de faixa, os desvios ficam em -1,6% (NF) e -3,2% (FF) contra a
teoria.

### 9.2 Pior caso adversarial (§4.9, §5.4)

O artigo constroi familias de instancias **feitas para humilhar** as
heuristicas e compara com o **otimo exato** (via B&B). Resultado: **as familias
NAO atingem as garantias**. As curvas ficam coladas em 1,00 — ou seja, mesmo em
instancias adversariais, o FFD continua sendo otimo.

**Por que isso e um resultado negativo declarado?** Porque e contra a
expectativa: a teoria diz que existem instancias ruins, e o artigo nao as
encontrou. Ele reporta isso "com todas as letras" (§4.9), sem esconder.

**O que o §5.4 acrescenta:** compara o FFD (1,001) com a garantia (1,889) e diz
que **existe folga** — ou seja, o FFD esta longe do teto. A razao esta em
~54,5% da garantia.

### 9.3 Contraexemplo: ordenar pode piorar (§4.10)

Este e o resultado mais contra-intuitivo do artigo, e vale saber a instancia exata.

**A instancia (6 itens):**

```
L = [0,34  0,39  0,33  0,28  0,36  0,27]
```

**O que acontece:**

| Algoritmo | Ordem | Bins |
|---|---|---|
| First Fit | original `[0,34 0,39 0,33 0,28 0,36 0,27]` | **2** (= o otimo) |
| First Fit Decreasing | decrescente `[0,39 0,36 0,34 0,33 0,28 0,27]` | **3** |

**Ordenar custou um bin.** E um contraexemplo minimo (6 itens), verificado contra
o otimo exato por branch-and-bound.

**Por que isso importa:** a recomendacao popular "sempre ordene antes de
empacotar" tem excecao. FFD tem garantia melhor que FF (`11/9` contra `17/10`),
mas **nesta instancia** o FF cru vence. O artigo assume que ordenar e
"quase sempre bom, nao sempre".

**Consequencia:** a recomendacao "sempre ordene antes de empacotar" tem
contraexemplo. Depende da instancia.

### 9.4 Random-order (§4.11)

Repete o experimento com os itens em ordem aleatoria. Resultado: a razao **nao
muda** (diferencas de ordem 10^-4). O que governa a qualidade e a **estrutura
de tamanhos** dos itens, nao a **ordem de chegada**.

**Contraponto com a literatura:** o Best Fit sob ordem aleatoria fica entre
1,009 e 1,131, o que e **abaixo** da faixa de 1,08 a 1,50 que Kenyon estima e
abaixo da conjectura de ~1,15. Ou seja: o resultado empirico e **melhor** que a
teoriaotimista predicts.

### 9.5 Progresso item a item (§4.12)

O artigo rastreia **quantos bins** estao abertos ao longo do processamento. Achado:
o NF cresce mais rapido e termina acima das demais (reflexo direto da razao
1,33). FFD e BFD quase coincidem — em 199 das 200 configuracoes, produzem o
mesmo numero de bins, diferindo apenas em **qual** bin cada item ocupa.

---

## 10. A Tabela 2 — Gap entre garantia e comportamento (§5.1)

Compara a **garantia teorica** com a **razao media medida** (media das quatro
distribuicoes):

| Heuristica | Garantia | Razao media | % da garantia usada |
|---|---|---|---|
| NF | 2,000 | 1,271 | 63,5% |
| FF | 1,700 | 1,050 | 61,8% |
| BF | 1,700 | 1,045 | 61,5% |
| FFD | 1,889 | 1,030 | 54,5% |
| BFD | 1,889 | 1,030 | 54,5% |

**O caso mais informativo:** o FFD tem a **melhor qualidade medida** (1,030) e
a **maior folga sobre a garantia** (54,5%). Ou seja: *a heuristica com a
garantia mais apertada e a mais subaproveitada na pratica*. Ordenar os itens
vale **mais** do que a teoria de garantias sugere.

**A leitura contra-intuitiva:** o NF usa 63,5% da garantia, mas a garantia dele
e a mais fraca (2,0). Portanto o NF e a heuristica que mais "desaproveita" sua
garantia em termos absolutos — mas como a garantia e fraca, ele ainda pode ser
"ruim" (1,271) e ainda assim ser o mais rapido.

## 11. Como argumentar se perguntarem (as 8 perguntas provaveis)

**Q1. "Por que 640× e não 494×?"**
Resposta: razao por pareamento de repeticao (mesma instancia, mesma execucao).
O NF e tao rapido (~40 us) que o relogio nao resolve entre series separadas.
Ver §7. Esta na §4.3 do artigo.

**Q2. "Por que a qualidade usa L2 e nao OPT?"**
Resposta: L2 e um **limite inferior** valido, calculado em tempo polinomial.
OPT e NP-difícil. Como `L2 <= OPT`, a razao `A/L2` e **conservadora**: se a
heuristica parece boa contra L2, e boa de verdade. Esta na §4.3.

**Q3. "O L2 e justo? Nao esta fraco demais?"**
Resposta honesta: o L2 do codigo e mais fraco que um bound trivial em ~45% das
instancias. Isso **infla** as racoes (as Heath>/L2 sao maiores do que seriam com
um bound mais forte), o que **joga a favor** do argumento (o gap para a
garantia fica ainda maior). Nao foi corrigido de proposito, porque corrigir
exigiria regenerar 5 tabelas + 4 graficos + o abstract. Decisao do autor,
documentada.

**Q4. "Por que o B&B nao converge para os 3 pontos grandes?"**
Resposta: o B&B e **exponencial**. Medido: `pior_3particao` n >= 30 nao converge
nem em 1 hora por ponto. Os valores publicados (12, 15, 19) foram conferidos
contra o B&B com orcamento longo e sao o otimo de verdade. Fechar isso exigiria
um L2 completo (cortes de alpha mais agressivos), que e trabalho de dissertacao.
Esta no codigo (`exact.rs`) e no `COMO-RODAR.md`.

**Q5. "Como voce sabe que 494 era errado?"**
Resposta: porque o metodo do pareamento reduziu a banda de ~5× para 1,06×, e
o valor de 494 vem de cruzar series (metodologicamente errado). O valor de 640
tem banda de 1,06× entre as tres rodadas.

**Q6. "Por que o Rust foi comecado como mais lento e terminou mais rapido?"**
Resposta (o artigo assume isso em §5.3): a primeira versao media 1,3–3× o
tempo do C. Equalizar o trabalho (bins-only) e adotar construcoes vetorizaveis
(iteradores sem bounds check, pre-alocacao, reducoes branchless) inverteu o
placar. A constante multiplicativa nao e propriedade exclusiva da linguagem.

**Q7. "A heuristica e' a mais estavel?"**
Resposta honesta (corrigido hoje): **nao**. As variantes quadraticas (BF, BFD)
tem os menores coeficientes de variacao (5,5–6,8%); o NF tem os MAIORES
(5,9–8,8%). E o oposto do que se espera, porque o NF e tao rapido que uma
preempcao pesa mais em termos relativos.

**Q8. "Como voce valida que Rust, C, C++ e Python fazem o mesmo trabalho?"**
Resposta: as quatro linguagens produzem **bins identicos** em 100% das 4.000
execucoes comparadas (3.000 comparacoes pareadas). A validacao cruzada
confirma que so o tempo difere. Esta na §5.2.

---

## 12. As limitacoes declaradas (§5.5, Conclusao)

O artigo declara explicitamente (nao e modéstia, e honestidade metodologica):

1. **O B&B nao resolve todas as instancias** adversariais (exponencial).
2. **Nao reproduzimos o pior caso canonico** de Johnson et al. — resultado
   negativo declarado.
3. **O L2 e mais fraco** que um bound trivial em 45% das instancias (inflando
   as racoes a favor do argumento).
4. **Ordenar nem sempre ajuda** — o contraexemplo de 6 itens mostra que pode
   piorar.
5. **O experimento e' sintetico** (distribuicoes de benchmark, nao dados reais).

---

## 13. Declaracao de uso de ferramentas de IA

O artigo tem uma secao inteira sobre isso (com uma tabela, `tab:ia`). **Voce
precisa saber o que ela declara.** O conteudo real:

> *"O desenvolvimento deste trabalho contou com apoio de ferramentas de IA
> generativa em todas as etapas. A tabela declara, para cada ferramenta, sua
> finalidade, o que efetivamente foi feito e a revisao humana correspondente,
> em cumprimento aos requisitos de integridade academica da disciplina.
> Trabalho individual."*

**As ferramentas declaradas (leia antes da apresentacao):**

| Ferramenta | Finalidade | Revisao humana declarada |
|---|---|---|
| **Hermes Agent** (agente autonomo; modelos `longcat-2.0:free` e `space-bunny-alpha`) | Desenvolvimento e auditoria | Todo o codigo Rust, os experimentos, e a auditoria numerica |
| **Claude Sonnet 4.6** | Revisao academica | Revisao de estilo e coesao |
| **Gemini 3.1 Pro** | Revisao academica | Idem |

**O que a secao NAO diz:** nao diz "a IA fez o trabalho sozinha". Diz que houve
revisao humana em cada etapa. Isso e o que a disciplina exige.

**Se perguntarem "isso e' seu ou da IA?":** a resposta honesta e a que esta na
secao: *"Usei IA como ferramenta de desenvolvimento e auditoria, com revisao
humana em todas as etapas. O trabalho e individual e a responsabilidade e
minha."* Nao minimize nem exagere.

**Se quiser conferir o que a secao diz palavra por palavra**, abra o `.tex` e
veja a tabela `tab:ia` (ultima pagina do artigo).

## 14. Checklist de preparacao

**Antes de apresentar:**
- [ ] Consigo explicar o problema de bin packing sem ler nada.
- [ ] Consigo listar as 5 heuristicas e suas garantias de cabeca.
- [ ] Consigo explicar por que 640× e nao 494× (pareamento).
- [ ] Consigo explicar o que e L2 e por que e conservador.
- [ ] Consigo dizer por que o B&B nao converge (exponencial) sem rodear.
- [ ] Sei que FFD/BFD sao as melhores em qualidade, NF e o mais rapido.
- [ ] Sei o contraexemplo de 6 itens (ordenacao pode piorar).

### Bonus: uma lição sobre o branch-and-bound (se perguntarem sobre o codigo)

O B&B tem **duas** podas alem do corte por tempo:
1. **Poda por incumbent** — descarta o ramo se ja abrimos tantos bins
   quanto a melhor solucao encontrada.
2. **Poda por carga** — se a carga que falta nao cabe na folga dos bins
   abertos, e preciso abrir `ceil(carga_faltante / 1.0)` bins novos. Se isso
   ja alcanca a incumbent, o ramo morre.

Uma terceira otimizacao: **regra de simetria**. Dois bins com o mesmo residuo
sao intercambiaveis — colocar o item no primeiro ou no segundo leva ao mesmo
estado. Sem essa regra o DFS reexploraria as permutacoes.

**O detalhe que pode aparecer:** essas regras sao **otimizacoes, nao
condicoes de corretude**. Medido: desligando a regra de simetria (pular ate o
primeiro de cada grupo de residuo em vez de pular os duplicados), os 12 casos
das familias adversariais e 4 casos degenerados **continuam dando o mesmo
numero**. Isso significa que a regra nao e' o que garante o resultado — o
garante e a busca exata. A regra so economiza tempo.

**E o B&B "mente"?** Nao, depois da correcao de hoje. Antes, quando o orcamento
estourava, devolvia `Some(incumbent_do_FFD)` mascarado de otimo. Agora devolve
`None`, como o doc promete. Se o comando imprimir *"B&B nao convergiu"*, isso e'
o comportamento correto — ele esta dizendo "nao sei", em vez de chutar.

---

**Se quiser conferir o esperado:**
- `cd "/home/ben/Área de trabalho/Onde deve rodar a IA/Analise-de-Algoritmos/Artigo/2026-09-BinPacking"`
- `cargo test --release` → 30 testes, todos passando
- `cargo run --release --bin confere_afirmacoes` → 15/15 afirmacoes confirmadas
- PDF: `artigo/artigo-parcial.pdf` (22 paginas)

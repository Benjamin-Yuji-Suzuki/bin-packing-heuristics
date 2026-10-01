# Transcrição do Theorem 2.1 — Johnson et al. (1974)

Fonte: `74_04_one_dimensional_packing.pdf` (scan, 27 páginas, p. 299–325).
Construção nas páginas **302–303**, Theorem 2.1.

---

## O que o artigo afirma

> **THEOREM 2.1.** For every $k > 1$, there exists a list $L$, with
> $L^* = k$, such that $FF(L) = BF(L) > 1.7 L^* - 8$.

E o resultado do Corollary, mais adiante (linha 489 do OCR):

> $\lim_{k\to\infty} R_{FF}(k) = 1.7$, $\lim_{k\to\infty} R_{BF}(k) = 1.7$

**Leitura:** a razão $FF(L)/L^*$ **tende a 17/10 = 1,7** quando $L^*$
cresce. Esse é o limite tight que o artigo propõe.

---

## A construção (como transcrevi)

$N$ divisível por 17, com $0 < \varepsilon < 1/50$. A lista $L$ tem
$\dfrac{30N}{17}$ itens em **três regiões** de tamanhos quase iguais a
$1/2$, $1/3$ e $1/4$, nessa ordem na lista.

Com $\varepsilon_i = \varepsilon / 18^i$:

### Região 1 — $N/17$ blocos de 10 itens, em torno de $1/2$

| item | valor |
|---|---|
| $a_{1,i}$ | $\frac12 + 3\varepsilon_i$ |
| $a_{2,i}$ | $\frac12 - 13\varepsilon_i$ |
| $a_{3,i}$ | $\frac12 - 3\varepsilon_i$ |
| $a_{4,i}$ | $\frac12 + 9\varepsilon_i$ |
| $a_{5,i}$ | $\frac12 + 3\varepsilon_i$ |
| $a_{6..10,i}$ | $\frac12 - 20\varepsilon_i$ |

Preenchem **$2N/17$ bins**.

### Região 2 — $N/17$ blocos de 10 itens, em torno de $1/3$

| item | valor |
|---|---|
| $b_{1,i}$ | $\frac13 + 46\varepsilon_i$ |
| $b_{2,i}$ | $\frac14 + 12\varepsilon_i$ |
| $b_{3,i}$ | $\frac14 - 34\varepsilon_i$ |
| $b_{4,i}$ | $\frac13 - 10\varepsilon_i$ |
| $b_{5,6,i}$ | $\frac13 + 6\varepsilon_i$ |
| $b_{7..10,i}$ | $\frac13 + 4\varepsilon_i$ |

Preenchem **$5N/17$ bins**.

### Região 3 — $10N/17$ itens, todos iguais a $1/4 + \varepsilon$

Preenchem **$10N/17$ bins**.

**Total do FF e do BF: $N$ bins.**
**Ótimo: $1 + 10N/17$ bins.**
Razão: $\dfrac{N}{1 + 10N/17} \to \dfrac{17}{10} = 1{,}7$.

---

## O que a transcrição entrega de fato

**Razão medida: 1,00 a 1,09** — muito abaixo de 1,7.

| $N$ | OPT | FF | razão |
|---|---|---|---|
| 17 | 10 | 10 | 1,000 |
| 34 | 19 | 20 | **1,053** |
| 51 | 28 | 29 | 1,036 |

Melhor caso medido: **1,09** (com $\varepsilon = 10^{-1}$, $N=34$).

---

## Por que não reproduz — e por que isso NÃO é resultado do artigo

O PDF é um **scan em tons de cinza**, com tipografia de 1974. Na passagem
crítica do Theorem 2.1 os coeficientes de $\varepsilon$ são traços finos que o
OCR lê de forma ambígua:

| no OCR | leitura correta | problema |
|---|---|---|
| `6, = 6-18!'` | $\varepsilon_i = \varepsilon/18^i$ | o `!'` é o **superíndice $i$**, não divisão |
| `3 + 336,,` | $1/2 + 3\varepsilon_i$ | o `$1/2$` vira `5` ou `§` |
| `a_4; = % — 136,` | $\frac12 - 13\varepsilon_i$ | dígito ambíguo |
| `Ap; + 4,,4+--- +a,,;=2 4+ 36,,` | $a_1+\dots+a_4 = 1/2 + 3\varepsilon_i$ | o `$1/2$`some e sobra `2` |

A correção que **funcionou** foi usar a restrição do próprio artigo
("$a_1 + \dots + a_4 = 1/2 + 3\varepsilon_i$") para derivar o coeficiente dos
itens $a_{6..10}$: com $a_1..a_5$ somando $\frac52 + 33\varepsilon_i$, os
cinco últimos precisam somar $\frac12 - 3\varepsilon_i$. Isso fez a razão
subir de 1,00 para 1,09.

O que **não** consegui determinar é a leitura correta dos coeficientes
específicos de cada item. Como os valores dependem deles, a construção fica
 qualitativamente correta (o FF degrada) mas numericamente distante do
teorema.

---

## O que isso significa para o artigo

**Não afirmar que reproduzimos o limite de 1,7.** O que é legítimo:

1. Reportar a busca estruturada (1,50) e a tentativa canônica (1,09) como
   resultados **negativos** — é informação honesta e útil.
2. Explicar **por que** o artigo declara 1,7 e por que nós não reproduzimos.
3. Manter a tese principal, que não depende disso: o **gap** entre garantia e
   comportamento típico é enorme.

---

## Se for tentar de novo

O caminho é ler o PDF com um humano (ou com um OCR melhor em resolução maior,
com filtro para a tipografia de 1974). Os valores que faltam confirmar são
só **10 números** — os coeficientes de $\varepsilon$ em $a_{1..5}$ e
$b_{1..10}$. Com eles, a construção fecha e o teste `razao_acima_de_1_005`
vira `razao_tende_a_1_7`.
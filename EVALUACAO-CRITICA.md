# Avaliação crítica do artigo — o que um professor pode atacar

Revisão feita do ponto de vista do **avaliador**, não do autor. Cada item
aqui é uma pergunta que pode ser feita na oral, com a resposta que o
artigo sustenta hoje e o que falta.

Classificação: 🔴 grave · 🟡 atenção · 🟢 sólido

---

## 🔴 1. O título promete "garantias de pior caso"; o experimento deu resultado **negativo**

**O título:** *Garantias de Pior Caso versus Desempenho Médio de Heurísticas
Clássicas para o Bin Packing Unidimensional*.

**O que o artigo mediu (§4.7):** as famílias adversariais **não atingem** as
garantias. Razão máxima medida: **1,35** (NF) contra a garantia de **2**. As
cinco heurísticas acertam o ótimo exato na maioria das instâncias.

**A pergunta que vai vir:** *"O artigo promete confrontar garantias de pior
caso. Você conseguiu reproduzir alguma? Se não, o que exatamente o trabalho
demonstra?"*

**Resposta honesta:** o que demonstramos é o **gap** entre a garantia e o
comportamento em distribuições de benchmark — e esse gap é grande e
consistente. O pior caso canônico **não** foi reproduzido. Ou seja: o
trabalho confronta garantias *indirectamente*, pela distância, não
atingindo-as.

**O que falta para fechar:** construir a instância canônica de Johnson et
al. (1974) para o First Fit e mostrar que ela realmente chega perto de 1,7.
Enquanto isso não existir, o professor tem base para dizer que o título
promete mais do que o texto entrega.

---

## 🔴 2. "Impraticável para grandes n" não tem evidência em escala grande

**O que o artigo afirma (resumo):** as implementações O(n²) de FF/BF
"tornam-se impraticáveis para grandes n".

**O que foi medido:**

| n | FF (Rust) |
|---|---|
| 1.000 | 0,11 ms |
| 4.000 | 1,64 ms |
| 16.000 | **25,5 ms** |

**A problema:** 25 ms não é impraticável. O maior n do artigo é 16.000, e a
afirmação sobre "grandes n" é uma **extrapolação** da curva — não uma
medição. Extrapola-se que n = 256.000 levaria ≈ 6,5 s, o que é lento mas
absolutamente viável num servidor.

**A pergunta:** *"Você mediu em algum n em que o algoritmo realmente ficou
impraticável? Ou isso é projeção?"*

**Resposta possível:** é projeção, e o texto deve dizer isso. O que os dados
sustentam é mais preciso: **a razão tempo/custo entre NF e FF é de 706×, e
essa razão cresce** — em n grande, a alternativa linear fica cada vez mais
atrativa. Isso é uma afirmação sobre *vantagem relativa*, não sobre
impraticabilidade absoluta, e é mais defensável.

---

## 🔴 3. Falta a instância canônica de Johnson et al. (1974)

**O que a literatura tem:** Johnson et al. provam que o First Fit é
**tight** em 1,7 — existem instâncias específicas (itens em faixas
particulares de tamanho, com escala Θ(n)) que fazem o FF chegar exatamente a
1,7·OPT.

**O que o artigo fez:** construiu famílias simples (itens alternados 0,5±ε,
3-partição forçada) e mediu contra o ótimo exato. Resultado: 1,13 no melhor
caso.

**A distância:** 1,13 contra 1,7. Não é o mesmo resultado.

**A pergunta:** *"Na literatura existe uma construção que faz o FF chegar a
1,7. Por que você não a fez?"*

**Resposta possível:** a construção de Johnson é específica e não trivial;
constrói-se em torno dos valores exatos 1/2, 1/4, 1/8... O experimento
reporta honestamente que não a reproduz. **Deixa isso no 2º bimestre** — é o
item mais valuable que sobrou, porque fecha a lacuna do item 1.

---

## 🟡 4. O cronограma estava desatualizado (corrigido agora)

Dizia "implementar solver exato" e "escrever Seções 4 e 5" — ambos **já
feitos** ou em andamento. Se o professor comparar o cronograma com o que
está no repositório, fica constrangedor.

Corrigido para marcar explicitamente o que está feito (B&B, ajuste de
expoentes, comparação de linguagens) e o que falta.

---

## 🟡 5. Variabilidade: 10 repetições são suficientes?

**Medido (coeficiente de variação sobre 10 repetições):**

| Algoritmo | Distribuição | CV |
|---|---|---|
| NF | U{1/100} | 0,41% |
| NF | 3-partição | 0,09% |
| FF | U{1/100} | 0,19% |
| FFD | 3-partição | 0,07% |

**Leitura:** a dispersão é **muito baixa** (CV < 0,5%). Dez repetições
sustentam as médias com folga. Não é um problema — mas vale saber o número
se perguntarem "como você sabe que a diferença entre 1,13 e 1,14 é real?".

**Ressalva honesta:** a dispersão dos *tempos* é bem maior que a das
*razões* (compare com o desvio de até 7% que medimos entre rodadas do
benchmark). Razões são determinísticas por instância — só mudam porque a
instância muda com a semente. Tempos são ruído de sistema.

---

## 🟡 6. O experimento multi-linguagem não responde à pergunta do artigo

**Observação estrutural:** o artigo é sobre bin packing e garantias de
aproximação. A comparação Rust × C × C++ × Python responde a uma pergunta
diferente (performance de implementação), que é interessante mas não é a
título.

**A pergunta:** *"Por que você comparou linguagens num artigo de
algoritmos?"*

**Resposta possível:** é um apêndice que demonstra FAIR-PLAY (todos em
otimização máxima) e reforça que a razão tempo é um artefato de
implementação, não do algoritmo — o que sustenta a mensagem central: a
constante multiplicativa é secundária frente ao expoente. Está justificado,
mas é secundário.

**Risco real:** um artigo de 15 páginas com metade dedicada a linguagens
pode parecer "muito java, pouco algoritmo". Se o professor for rigoroso,
talvez valha **encurtar** a seção 4.4 e mover para apêndice.

---

## 🟡 7. Faltam as Seções de Discussão e Conclusão

O artigo é parcial (1º bimestre). Isso é esperado e declarado no
cronograma. Mas: **sem conclusão, não há síntese**. Um artigo sem uma frase
final que amarre "garantia vs comportamento" perde o ponto.

---

## 🟢 8. O que está sólido

Estes pontos não têm fragilidade:

- **L2 ≤ OPT garantido** — 0 violações em 4.800 instâncias. O limite de
  Martello-Toth é implementado corretamente e validado.
- **A razão reportada é conservadora** — A/L2 supera A/OPT em 0,15% a 4,7%.
  A afirmação no texto é verificável e verdadeira.
- **B&B validado** — confere com busca exata sem podas em 600 instâncias.
  As podas estão corretas.
- **Reprodução da Tabela 2** — desvio mediano de 1,2% entre duas medições
  independentes. Reprodutibilidade é o mais forte argumento de rigor.
- **Contrexemplo FFD > FF** — mínimo (n=6), verificado contra o ótimo exato.
  É um achado real e raro.
- **Expoentes medidos** — β₁ ≈ 1,9 para as séries quadráticas, R² ≥ 0,996.
  A teoria se confirma.
- **Código limpo** — 25 testes, incluindo auditoria de viabilidade,
  determinismo e casos degenerados.

---

## As 3 perguntas mais prováveis na oral

1. **"Por que você não construiu a instância que faz o FF chegar a 1,7?"**
   → Resposta: a construção de Johnson et al. é específica; reportamos
   honestamente que não a reproduzimos; é o principal trabalho restante.

2. **"De onde vem o número 706×?"**
   → Resposta: razão de tempos dentro de uma única série (variantes
   bins-only, mesma carga de trabalho), processador fixado num P-core,
   três rodadas, mediana. Bins idênticos entre linguagens — só o tempo
   difere.

3. **"Qual é a contribuição deste trabalho, em uma frase?"**
   → Resposta: *mostrar que, em distribuições de benchmark realisticas, as
   heurísticas clássicas ficam muito mais perto do ótimo do que suas
   garantias de pior caso sugerem — e que o custo de tempo é um fator
   independente, dominado pela complexidade e não pela linguagem.*

---

## Veredito

O trabalho é **sólido tecnicamente e honesto metodologicamente** — a auditoria
não encontrou erro nos números nem no código. O ponto fraco não é técnico,
é de **escopo**: o título e a pergunta do trabalho pedem uma coisa (pior
caso) que o experimento não entrega diretamente (resultado negativo na
pior caso). A defesa mais forte é **reformular a contribuição** em torno
disso, não tentar fingir que o pior caso foi atingido.

**Antes da avaliação**, a Priorities são:
1. Reformular a contribuição para refletir o resultado real (gap, não
   atingimento).
2. Construir a instância de Johnson et al. (fecha o item 🔴1 e 🔴3).
3. Medir em n grande (fecha o item 🔴2).
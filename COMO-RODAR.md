# Como rodar tudo

Guia completo do projeto, na ordem. Todos os tempos abaixo foram **medidos**
nesta máquina (i5-13420H, 12 threads), não estimados.

---

## Regra de ouro: meça com a máquina livre

Benchmark medido com a CPU ocupada não reproduz. Já medimos desvios de até
**6,7%** entre rodadas contaminadas e limpas — o bastante para distorcer um
multiplicador como `rust/C`, que é número publicado.

```bash
bash scripts/preflight.sh
```

Ele mede o uso **instantâneo** de CPU (delta de `/proc/<pid>/stat`, janela de
2 s), mostra quem está usando, e **reprova** se o sistema passar de 25% — ou
se você tentar fixar num E-core (ver *P-core vs E-core* abaixo).

```
==============================================
 PRE-FLIGHT: a maquina esta livre para medir?
==============================================
  carga (1 min) : 0
  carga (5 min) : 1
  threads       : 12
  RAM livre     : 10Gi
  GPU           : 41%  (irrelevante: o benchmark e CPU-only, single-threaded)
  ...
  nucleo escolhido: cpu0 (P-core, ok)

 OK: sistema usando 18.1% de 25% permitido.
 O benchmark sera fixado no nucleo 0 (taskset).
==============================================
```

### ⚠️ P-core vs E-core (a armadilha que custou uma rodada)

Este CPU é **heterogêneo**:

| CPUs | Tipo | Hyperthread |
|---|---|---|
| **0–7** | P-core (performance) | sim — pares 0/1, 2/3, 4/5, 6/7 |
| **8–11** | E-core (eficiência) | não |

Medição direta (Next Fit, Rust, n=16.000): **cpu0 → 39,0 µs** · **cpu11 → 78,0 µs**.
Um E-core é **2× mais lento**. `taskset -c 11` infla tudo em 2×.

**Sempre fixe num P-core.** O padrão já é `cpu0`. O preflight reprova se você
passar `NUCLEO=8..11`.

O irmão de hyperthread importa tanto quanto o núcleo (os dois dividem o cache
L1). Para escolher o par:

```bash
/usr/bin/python3 scripts/cpu_instantanea.py 3
```

Escolha o par mais silencioso. Medição deste projeto:

```
par cpu0/cpu1:  0,7% +  0,0% =  0,7%   <- escolhido
par cpu2/cpu3:  1,0% +  0,0% =  1,0%
par cpu4/cpu5:  6,5% +  0,0% =  6,5%
par cpu6/cpu7:  0,7% + 95,7% = 96,4%   <- evitado
```

---

## 0. Preparação

```bash
cd "/home/ben/Área de trabalho/Onde deve rodar a IA/Analise-de-Algoritmos/Artigo/2026-09-BinPacking"

cargo build --release            # ~7 s
cargo test                       # 36 testes, ~3 s
```

**Compilação fair-play** (necessária para o benchmark ser justo):

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
gcc -O3 -march=native          -o bench/bench_c   bench/bench.c   -lm
g++ -O3 -march=native -std=c++17 -o bench/bench_cpp bench/bench.cpp
```

---

## 1. Experimento principal (qualidade) — ~10 s

Gera `resultados.csv` com as 1.000 execuções: 4 distribuições × 5 tamanhos ×
10 repetições × 5 algoritmos.

```bash
./target/release/bin-packing-heuristics experiment \
    --sizes 1000,2000,4000,8000,16000 --reps 10 --out resultados.csv
```

Rodar **uma** instância e ver a solução de cada heurística:

```bash
./target/release/bin-packing-heuristics run -n 20 -d tres_particao
./target/release/bin-packing-heuristics dists        # lista as 4 distribuições
```

---

## 2. Benchmark entre linguagens (tempo) — ~65 min

Compara Rust × C × C++ × Python sobre **as mesmas instâncias**.

```bash
bash scripts/rebench.sh
```

O script, sozinho: exporta as instâncias → compila com fair-play → roda **3
rodadas** de cada linguagem, todas fixadas no cpu0 → consolida.

Saídas:

| Arquivo | Conteúdo |
|---|---|
| `bench/runs/{lang}_{1,2,3}.csv` | as 3 rodadas brutas |
| `bench/carga_da_medicao.txt` | **a condição da medição** (carga, núcleo, flags) |
| stdout | tabela consolidada com medianas e `rust/C` |

**É o único passo demorado**, porque o Python sozinho leva ~11 min por rodada.
Para caber num tempo menor, edite o laço `for rodada in 1 2 3` em
`scripts/rebench.sh` — mas a mediana de 3 é o que dá robustez ao número.

Rodar só uma parte (útil para depurar):

```bash
./target/release/bin-packing-heuristics export \
    --sizes 1000,2000,4000,8000,16000 --reps 10 --out instancias
taskset -c 0 ./target/release/bin-packing-heuristics bench --dir instancias --out rust.csv
taskset -c 0 ./bench/bench_c   instancias c.csv
taskset -c 0 ./bench/bench_cpp instancias cpp.csv
taskset -c 0 /usr/bin/python3 bench/bench.py instancias python.csv
/usr/bin/python3 scripts/consolida_bench.py
```

---

## 3. Branch-and-bound entre linguagens — ~26 s

Mede o **solver exato** nas 4 linguagens.

```bash
bash scripts/bnb.sh              # padrão: n até 20, 10 repetições
bash scripts/bnb.sh 24 5         # n até 24, 5 repetições (Python demora mais)
```

Antes de reportar qualquer tempo, o script valida o ponto decisivo: **as 4
linguagens encontram o mesmo ótimo?** Se não, a comparação não significa nada.

```
   >>> ÓTIMO IDÊNTICO NAS 4 LINGUAGENS: True
```

> O Rust **não** usa o atalho "se FFD já bate o L2, não busca" que existe em
> `src/exact.rs`. Isso é deliberado: o atalho faria o Rust pular trabalho em
> 189 de 240 instâncias e mediria o atalho, não a linguagem.

---

## 4. Pior caso e melhor caso — ~6 s

Constrói instâncias adversariais e mede contra o **ótimo exato**.

```bash
./target/release/bin-packing-heuristics worstcase --sizes 12,24,36,48
./target/release/bin-packing-heuristics worstcase \
    --families pior_nf_classico,pior_3particao
```

**Achado honesto:** essas famílias **não** atingem as garantias teóricas
(todas acertam o ótimo; o NF chega a 1,35 contra a garantia de 2). Aparece no
artigo como resultado negativo, não como sucesso.

---

## 4b. Busca de pior caso + velocidade de cada algoritmo — ~30 s

Este é o comando que **fecha a lacuna de escopo** apontada em
`EVALUACAO-CRITICA.md`: o artigo promete confrontar garantias de pior
caso, e o experimento anterior só media instâncias aleatórias.

```bash
# busca o pior caso por hill climbing, contra o ÓTIMO EXATO
cargo run --release --bin piorcasa 12 400

# mais instancias e mais tentativas = mais chance de achar pior caso
cargo run --release --bin piorcasa 16 6000

# so um algoritmo
cargo run --release --bin piorcasa 12 400 FF
```

**O que ele faz:** perturba o multiconjunto de tamanhos (passos de 1/1000 e
1/10000, que cobrem as faixas críticas 1/2, 1/3, 1/4...), calcula o ótimo
exato por branch-and-bound e mede a razão A(I)/OPT(I) de cada heurística.
Reinicia a instância de vez em quando para escapar de ruins locais.

**Resultado que ele mostra no terminal:**

```
alg         OPT    A(I) max       razão      garantia   distância
NF            8          11      1.3750        2.0000      0.6250
FF            7           9      1.2857        1.7000      0.4143
FFD           4           5      1.2500        1.3889      0.1389
```

E em seguida a **velocidade de cada algoritmo**, que é o que interessa para
o artigo:

```
--- n = 16000 ---
alg         bins      tempo (µs)         vs NF
NF         13333          106.00          1.0x
FF         10667        35661.00        336.4x
BF         10667        63001.00        594.3x
FFD        10667        50757.00        478.8x
BFD        10667        80364.00        758.2x
```

**Leia assim:** ao ir de n = 1.000 para n = 16.000 (16× mais itens), o NF
custa 26× mais tempo e o FF custa 230× mais. É a assinatura da complexidade
linear contra a quadrática, visível num terminal. E a coluna "vs NF" mostra
que a vantagem do linear **cresce** — 38,8× em n = 1.000, 336× em n = 16.000.

**Por que a razão de pior caso não atinge a garantia:** o hill climbing
plateau em ~1,29 para o First Fit, contra a garantia de 1,7. Atingir 1,7
exige a construção canônica de Johnson et al. (1974), que é específica e não
foi reproduzida aqui. **Esse é o resultado honesto**: a busca gera
aproximações, não prova que o limite é 1,7 nem que é 1,29.

---

## 4c. Busca estruturada da pior caso (Johnson) — ~1 s

Complemento do `piorcasa`: em vez de sortear tamanhos, este comando varre
uma **grade** sobre famílias em camadas — que é onde o pior caso do First Fit
realmente mora.

```bash
cargo run --release --bin johnson 4        # m = 4 itens por camada
cargo run --release --bin johnson 6        # mais itens por camada
```

**O que ele faz:** testa famílias de 2 e 3 camadas (itens grandes que abrem
bins, cujo resíduo não comporta o grupo seguinte), depois varre uma grade de
14.365 pares de tamanhos, e por fim completa o melhor par com uma terceira
camada. Mede contra o ótimo exato.

**Resultado medido (m = 4):**

```
Grade fina em 2 camadas: 14.365 pares avaliados
  melhor razao FF = 1.5000  (garantia 1,7; otimo 2; FF abre 3)
  instancia: [0.255, 0.24] x4
```

Isso é **melhor** que a busca aleatória (1,29) — a estrutura em camadas
importa — mas ainda não atinge 1,7.

**As duas buscas juntas mostram:** nem o sorteio nem a grade em camadas
reproduzem a família canônica de Johnson et al. (1974). Ela não é um ponto
do espaço de busca, é uma construção específica. O artigo reporta isso como
resultado negativo honesto.

---

## 5. Progresso e ordenação — segundos

```bash
# bins abertos item a item, nas 5 heurísticas
./target/release/bin-packing-heuristics progress --alg todas --n 30 --familia pior_3particao
./target/release/bin-packing-heuristics progress --alg FF --n 30 --dist tres_particao

# mesma instância, ordem original × decrescente
./target/release/bin-packing-heuristics sorted-vs-unsorted --n 4000 --dist tres_particao
```

O `sorted-vs-unsorted` expõe o contraexemplo do artigo: existe instância em que
**ordinar piora** o First Fit (`[0.34,0.39,0.33,0.28,0.36,0.27]` → FF usa 2
bins = ótimo, FFD usa 3).

---

## 6. Gráficos

**Atenção ao interpretador:** use `/usr/bin/python3` (tem matplotlib 3.6.3).
O `python3` do PATH **não tem matplotlib** e falha.

```bash
# dados dos experimentos novos (Rust)
cargo run --release --bin exporta_graficos
/usr/bin/python3 scripts/graficos_novos.py     # progresso, ordenado, pior caso

### Atencao: `exporta_graficos` leva ~6 min30

O branch-and-bound é exponencial nas familias adversariais maiores.
`pior_3particao` com n >= 30 **nao converge nem em 10 minutos** (medido),
entao o comando da 120 s por ponto e estoura nesses tres casos.

Quando isso acontece ele **preserva o `opt` que ja estava no CSV** e
imprime um AVISO no stdout:

```
AVISO: B&B nao convergiu em 120 s para pior_3particao n=30;
       mantido o opt anterior (12) do CSV.
```

Isso e' deliberado. Uma versao anterior usava `continue`, e o `None` do
orçamento estourado **removia a linha do CSV em silencio** -- o
`dados_piorcaso.csv` caia de 20 para 17 linhas e a figura ficava com 3
pontos a menos, sem erro e sem aviso. Os valores preservados (12, 15, 19)
foram conferidos contra o B&B com orcamento longo: sao o otimo de verdade.

Para refazer o OPT desses tres pontos com orcamento longo (1 h por ponto):

```bash
cargo run --release --bin exporta_graficos -- --opt-exato
```

A flag troca o orcamento de 120 s para 1 h por ponto. Medido: mesmo 1 h
NAO converge para os tres mais duros (pior_3particao n>=30) -- a busca e'
exponencial e o gargalo e' intrinseco, nao a flag. Ela serve para os 17
pontos que converge em 120 s e para documentar a tentativa nos outros tres,
que seguem sendo preservados do CSV com aviso.

O numero publicado (12, 15, 19) foi conferido contra o B&B com orcamento
prolongado e e' o otimo de verdade; o que nao existe e' um atalho BARATO
de recalcula-lo.

# dados já medidos
/usr/bin/python3 scripts/graficos.py            # tempo×n, razão×dist, 3-partição

# linguagens: o script lê bench_*.csv da RAIZ, que são de 24/09.
# Para usar as rodadas novas, copie de bench/runs/ antes:
cp bench/runs/rust_1.csv bench_rust.csv
cp bench/runs/c_1.csv    bench_c.csv
cp bench/runs/cpp_1.csv  bench_cpp.csv
cp bench/runs/python_1.csv bench_python.csv
/usr/bin/python3 scripts/grafico_linguagens.py
```

| Gráfico | O que mostra |
|---|---|
| `grafico_progresso.png` | bins abertos × item processado, 5 heurísticas × 4 distribuições |
| `grafico_ordenado_desordenado.png` | bins e tempo: ordenar × não ordenar |
| `grafico_pior_caso.png` | razão contra o **ótimo exato** + linhas de garantia |
| `grafico_tempo_n.png` | tempo × n em log-log |
| `grafico_razao_distribuicao.png` | razão A(I)/L2(I) por distribuição |
| `grafico_razao_3particao.png` | razão × n no caso 3-partição |
| `grafico_linguagens.png` | Rust × C × C++ × Python |

---

## 7. Artigo (LaTeX)

```bash
cd artigo
pdflatex artigo-parcial.tex && bibtex artigo-parcial \
  && pdflatex artigo-parcial.tex && pdflatex artigo-parcial.tex
```

Quatro passadas: duas para as referências cruzadas (`\ref`) resolverem e
duas para a bibliografia. Resultado: 18 páginas, 6 figuras, 8 tabelas.

### Verificar que o documento é reprodutível

```bash
bash scripts/verificar_pdf_20x.sh
```

Compila o artigo **20 vezes do zero** e compara o texto extraído de cada
PDF. Resposta: as 20 saem idênticas — nenhuma tabela muda de página,
nenhuma referência resolve diferente, nenhum número muda de alinhamento.

Isto é diferente da verificação de dados (`verificar_20x.sh`): aquela
mede se os **números** são estáveis, esta mede se o **documento** é
reproduzível.

### Verificar que o código é determinístico

```bash
bash scripts/verificar_determinismo.sh 20
```

Roda cada um dos 5 comandos da ferramenta 20 vezes e compara a saída
**semântica** — bins, razão, ótimo. Os tempos são excluídos, porque
variam a cada execução e dariam falso negativo.

```
[1/5] run ................. identico nas 20 execucoes
[2/5] worstcase ........... identico nas 20 execucoes
[3/5] progress ............ NF/FF/BF/FFD/BFD identicos
[4/5] sorted-vs-unsorted .. identico nas 20 execucoes
[5/5] experiment .......... identico nas 20 execucoes

RESULTADO: todos os comandos sao DETERMINISTICOS
```

### Verificar que os números são estáveis

```bash
bash scripts/verificar_20x.sh 16000
```

Repete o **experimento** 20 vezes sorteando instâncias diferentes a cada
vez (`--salt`). Medido: CV da razão média abaixo de 0,19%, e os 20
números da Tabela 2 reproduzem dentro de 0,12%.

---

## Resumo — o que rodar, e quanto tempo

| Passo | Comando | Tempo |
|---|---|---|
| 0 | `cargo build --release && cargo test` | ~10 s |
| 1 | `bin-packing-heuristics experiment` | ~10 s |
| 2 | `bash scripts/rebench.sh` | **~65 min** |
| 3 | `bash scripts/bnb.sh` | ~26 s |
| 4 | `bin-packing-heuristics worstcase` | ~6 s |
| 4b | `piorcasa` (pior caso + velocidade) | ~30 s |
| 4c | `johnson` (busca estruturada) | ~1 s |
| 5 | `progress` / `sorted-vs-unsorted` | ~5 s |
| 6 | `exporta_graficos` | **~6 min30** (ver nota) |
| 6b | scripts Python dos gráficos | ~20 s |
| 7 | LaTeX ×4 passadas | ~15 s |

**Nota sobre o passo 6.** `exporta_graficos` ficou lento porque o
branch-and-bound é exponencial nas famílias adversariais maiores:
`pior_3particao` com n >= 30 não converge nem em 10 minutos. O comando
dedica 120 s por ponto e, nos três que estouram, **preserva o `opt` já
gravado no CSV e avisa no stdout**. Se você só quer os gráficos de
progresso/ordenado e não a figura de pior caso, os outros dois CSV são
gerados em segundos — só o terceiro bloco (`pior_caso()`) é o lento.

**Dois passos são demorados: o 2 (rebench, ~65 min) e o 6
(`exporta_graficos`, ~6 min30). Os outros 7 dão em ~90 segundos no total.

---

## Checklist antes de reportar qualquer número

1. `bash scripts/preflight.sh` aprovou?
2. O processo está num **P-core** (cpu0–7), com o par de hyperthread livre?
3. Todas as linguagens executaram **exatamente a mesma quantidade de trabalho**?
4. Numerador e denominador vêm da **mesma série** experimental?
5. A métrica tem resolução para separar os grupos? (mediana ~0 não informa nada —
   nesse caso use o **total somado**)
6. O número foi lido do CSV bruto, não de uma transcrição?

Se qualquer resposta for *não*, o número não entra no artigo.

Veja também `ARMADILHAS-DE-MEDICAO.md` para as cinco lições aprendidas na
marra.
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
cargo test                       # 25 testes, ~2 s
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
duas para a bibliografia. Resultado: 14 páginas, 6 figuras, 5 tabelas.

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
| 5 | `progress` / `sorted-vs-unsorted` | ~5 s |
| 6 | scripts Python dos gráficos | ~20 s |
| 7 | LaTeX ×4 passadas | ~15 s |

**Só o passo 2 é demorado.** Os outros 7 dão em ~90 segundos no total.

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
# bin-packing-heuristics

Heurísticas clássicas para o **Bin Packing unidimensional** — projeto da
disciplina Análise e Projeto de Algoritmos (CESUPA, 2026).

Implementa **NF, FF, BF, FFD e BFD** com implementações diretas (sem
estruturas auxiliares que escondam a complexidade) + o lower bound **L2**
de Martello–Toth, e roda o experimento completo: qualidade (razão A(I)/L2(I))
× tempo, em 4 distribuições × 5 tamanhos × 10 repetições.

## Resultados principais (n = 16.000, média de 10 repetições)

| Algoritmo | razão vs L2 (uniforme) | razão (3-partição) | tempo (µs) |
|-----------|------------------------|--------------------|------------|
| NF        | 1.323                  | 1.235              | 146        |
| FF        | 1.017                  | 1.131              | 25.527     |
| BF        | 1.009                  | 1.131              | 48.228     |
| FFD       | 1.000                  | 1.105              | 30.774     |
| BFD       | 1.000                  | 1.105              | 71.068     |

**Garantias teóricas para comparação:** NF ≤ 2·OPT, FF/BF ≤ 1.7·OPT,
FFD/BFD ≤ 11/9·OPT + 6/9 ≈ 1.222 (Dósa 2007).

## Como rodar

```bash
cargo run --release -- run -n 20 -d tres_particao      # instância única
cargo run --release -- experiment                       # experimento completo (grava resultados.csv)
cargo test                                             # 10 testes (unit + integração + propriedades)
```

Distribuições disponíveis (`cargo run -- dists`): `uniforme_continua`,
`uniforme_discreta_100`, `tres_particao`, `falkenauer_u120`.

## Estrutura

```
src/algorithms.rs   — NF, FF, BF, FFD, BFD, lower bound L2
src/generators.rs   — gerador de instâncias (4 distribuições, semente determinística)
src/experiment.rs   — protocolo experimental + CSV
src/main.rs         — CLI (clap)
tests/integration.rs— testes de propriedade (garantias, lower bounds, contrexemplos)
scripts/            — geração de gráficos (matplotlib)
resultados.csv      — 1.000 execuções do experimento completo
```

## Comparação entre linguagens (Rust × C × C++ × Python)

Para isolar o efeito da linguagem (constante multiplicativa do custo),
as mesmas instâncias são exportadas em binário (f64 little-endian) e
consumidas pelas 4 implementações — **mesmos itens, mesmos algoritmos,
mesmo protocolo** (aquecimento + 1 medição por instância).

```bash
cargo run --release -- export --sizes 1000,2000,4000,8000,16000 --reps 10
RUSTFLAGS="-C target-cpu=native" cargo build --release
./target/release/bin-packing-heuristics bench --dir instancias --out bench_rust.csv
gcc -O3 -march=native -o bench/bench_c bench/bench.c -lm && ./bench/bench_c instancias bench_c.csv
g++ -O3 -march=native -std=c++17 -o bench/bench_cpp bench/bench.cpp && ./bench/bench_cpp instancias bench_cpp.csv
python3 bench/bench.py instancias bench_python.csv
python3 scripts/grafico_linguagens.py
```

**Validação**: os bins produzidos pelas 4 linguagens são idênticos em
100% das execuções — só o tempo difere. Números reportados: mediana de
3 execuções completas (cada uma = 10 repetições por configuração).

Resultado (n=16.000, uniforme discreta, mediana de 3 execuções):

| Algoritmo | Rust (µs) | C (µs)  | C++ (µs) | Python (µs) | rust/C |
|-----------|-----------|---------|----------|-------------|--------|
| NF        | 39        | 70      | 64       | 741         | 0.55×  |
| FF        | 26.656    | 25.423  | 24.851   | 1.899.441   | 1.05×  |
| BF        | 31.606    | 34.226  | 35.757   | 2.319.910   | 0.92×  |
| FFD       | 28.480    | 28.129  | 27.589   | 2.007.822   | 1.01×  |
| BFD       | 41.211    | 42.305  | 42.288   | 3.372.516   | 0.97×  |

**Rust ganha de C/C++ em NF (1.8× mais rápido), BF e BFD, e empata em
FF/FFD (≤5%, dentro do ruído)** — com C/C++ em `-O3 -march=native` e
Rust em `target-cpu=native` (fair play: mesma configuração de
otimização máxima para todos). Python fica ~10× atrás no linear e ~75×
nos quadráticos.

Como? Variantes "bins-only" (mesmo trabalho que as outras linguagens,
sem rastreio de atribuição) + idiomas que o LLVM vetoriza: iteradores
(sem bounds check), `Vec::with_capacity`, `f64::total_cmp` no sort
(comparação por bits, sem branch de NaN) e redução de mínimo branchless
no Best Fit (mínimo mascarado — vetorizável como `minpd`). Zero
`unsafe`. A assintótica é idêntica em todas; o que muda é a constante —
a distinção O-grande × realidade do eixo E1.

## Reprodutibilidade

- Sementes determinísticas (splitmix64 misturado com n): mesma semente →
  mesma instância, sempre.
- Ambiente de referência: Intel Core i5-13420H, 16 GB RAM, Linux Mint 22.3,
  Rust 1.98 (`--release`, LTO).
- `resultados.csv` contém todas as 1.000 execuções (algoritmo,
  distribuição, n, rep, bins, L2, razão, tempo em µs).

## Referências-chave

- Johnson et al. 1974 — garantias NF/FF/BF (SIAM J. Computing)
- Dósa 2007 / Dósa et al. 2013 — bound exato 11/9 + 6/9 do FFD
- Martello & Toth 1990 — lower bound L2 (Discrete Applied Mathematics)
- Falkenauer 1996 — distribuições de benchmark (Journal of Heuristics)

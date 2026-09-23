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
cargo run --release -- bench --dir instancias --out bench_rust.csv
gcc -O2 -o bench/bench_c bench/bench.c -lm && ./bench/bench_c instancias bench_c.csv
g++ -O2 -std=c++17 -o bench/bench_cpp bench/bench.cpp && ./bench/bench_cpp instancias bench_cpp.csv
python3 bench/bench.py instancias bench_python.csv
python3 scripts/grafico_linguagens.py
```

**Validação**: os bins produzidos pelas 4 linguagens são idênticos em
100% das 1.000 execuções — só o tempo difere.

Resultado (n=16.000, uniforme discreta, média de 10 repetições):

| Algoritmo | Rust (µs) | C (µs) | C++ (µs) | Python (µs) | rust/C |
|-----------|-----------|--------|----------|-------------|--------|
| NF        | 609       | 204    | 208      | 741         | 2.99×  |
| FF        | 139.231   | 90.399 | 101.605  | 1.899.441   | 1.54×  |
| BF        | 252.449   | 108.734| 135.771  | 2.319.910   | 2.32×  |
| FFD       | 134.031   | 106.616| 103.042  | 2.007.822   | 1.26×  |
| BFD       | 312.125   | 161.557| 168.337  | 3.372.516   | 1.93×  |

Rust fica 1.3–3× mais lento que C/C++ (custo do safe Rust: bounds
checking e abstrações de `Vec`). Python fica 10–15× atrás nos
quadráticos — mas apenas 1.2× no NF linear (otimização de laços do
CPython 3.11). A assintótica é idêntica; o que muda é a constante —
exatamente a distinção O-grande × realidade do eixo E1.

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

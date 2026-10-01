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

> **Roteiro completo:** [`COMO-RODAR.md`](COMO-RODAR.md) — todos os comandos,
> com os tempos reais de cada um medidos nesta máquina.
> **Armadilhas de medição:** [`ARMADILHAS-DE-MEDICAO.md`](ARMADILHAS-DE-MEDICAO.md)
> — os erros que só aparecem no número, nunca no código.
> **Avaliação crítica:** [`EVALUACAO-CRITICA.md`](EVALUACAO-CRITICA.md) — os pontos
> fracos do artigo, do ponto de vista de quem vai avaliar.

```bash
# 0. (opcional) conferir se a máquina está livre
bash scripts/preflight.sh

# 1. benchmark das 5 heurísticas — re-grava a Tabela 2 do artigo
bash scripts/rebench.sh            # ~35 min; ABORTA se a máquina estiver ocupada

# 2. benchmark do branch-and-bound entre linguagens
bash scripts/bnb.sh                # ~2 min; mesma política de pre-flight
```

> **Por que o pre-flight existe.** Medimos desvios de até **6,7%** entre
> rodadas contaminadas (load 3,4–4,7) e limpas — o bastante para
> distorcer um multiplicador como `rust/C`, que é o número reportado.
> Os scripts agora **recusam** rodar com a máquina ocupada, em vez de
> depender de lembrar de fechá-la.
>
> Se a carga residual for **apenas** o Hermes desktop (≈48% da CPU, e
> não pode ser fechado porque é o processo que executa o script), rode
> com `LIMITE_CARGA=1.5` e registre a carga real no artigo.

```bash
cargo run --release -- run -n 20 -d tres_particao      # instância única
cargo run --release -- experiment                       # experimento completo (grava resultados.csv)
cargo test                                             # 25 testes (unit + integração + propriedades + B&B + auditoria)

# ---experimentos de pior caso, progresso e ordenação ---
cargo run --release -- worstcase --sizes 12,24,36,48    # pior/melhor caso vs ÓTIMO EXATO (B&B)
cargo run --release -- progress --alg todas --n 30 --familia pior_3particao
cargo run --release -- sorted-vs-unsorted --n 4000 --dist tres_particao
```

**`worstcase`** constrói instâncias adversariais e as mede contra o ótimo
exato (branch-and-bound), permitindo reportar a razão verdadeira
`A(I)/OPT(I)` em vez da conservadora `A(I)/L2(I)`.

**`progress`** mostra, item a item, quantos bins estão abertos — é o
rastreamento de "como a heurística está indo" que produz o gráfico de
progresso.

**`sorted-vs-unsorted`** compara a mesma instância em ordem original e
decrescente, medindo bins e tempo: mostra o que a ordenação realmente
muda (e que, para FFD/BFD, não muda nada — eles já ordenam).

### Gráficos

```bash
cargo run --release --bin exporta_graficos        # gera os 3 CSVs de dados
/usr/bin/python3 scripts/graficos_novos.py        # gera os 3 gráficos
/usr/bin/python3 scripts/graficos.py              # gera os 3 gráficos antigos
```

**Atenção ao interpretador:** use `/usr/bin/python3` (matplotlib 3.6.3).
O `python3` do PATH (Hermes 3.14) **não tem matplotlib** e falha.

| Gráfico | O que mostra |
|---|---|
| `grafico_progresso.png` | bins abertos × item processado, 5 heurísticas × 4 distribuições |
| `grafico_ordenado_desordenado.png` | bins e tempo: ordenar × não ordenar |
| `grafico_pior_caso.png` | razão contra o **ótimo exato** nas famílias adversariais |
| `grafico_tempo_n.png` | tempo × n em log-log (expoentes) |
| `grafico_razao_distribuicao.png` | razão A(I)/L2(I) por distribuição |
| `grafico_razao_3particao.png` | razão × n no caso 3-partição |
| `grafico_linguagens.png` | Rust × C × C++ × Python |

Detalhe de leitura do `grafico_progresso.png`: **as curvas do FFD e do BFD
coincidem exatamente** nas quatro distribuições — ambos abrem o mesmo
*número* de bins (diferem só em *qual* bin cada item ocupa). Por isso são
traçadas com estilos de linha distintos; sem isso uma fica escondida sob
a outra.

Distribuições disponíveis (`cargo run -- dists`): `uniforme_continua`,
`uniforme_discreta_100`, `tres_particao`, `falkenauer_u120`.

## Estrutura

```
src/algorithms.rs   — NF, FF, BF, FFD, BFD, lower bound L2
src/exact.rs        — solver exato branch-and-bound (viável até n ≈ 60)
src/adversarial.rs  — famílias de pior caso e melhor caso
src/progress.rs     — rastreamento item a item + ordenado × desordenado
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
3 execuções completas (cada uma = 10 repetições por configuração), com o
processo fixado num *performance core* e a carga do sistema registrada.

Resultado (n=16.000, uniforme discreta, mediana de 3 execuções, máquina ociosa):

| Algoritmo | Rust (µs) | C (µs)  | C++ (µs) | Python (µs) | rust/C |
|-----------|-----------|---------|----------|-------------|--------|
| NF        | 38        | 70      | 64       | 530         | 0.55×  |
| FF        | 24.889    | 24.929  | 24.768   | 1.369.383   | 1.00×  |
| BF        | 30.988    | 34.090  | 35.653   | 1.743.061   | 0.91×  |
| FFD       | 27.148    | 27.987  | 27.298   | 1.483.583   | 0.97×  |
| BFD       | 41.308    | 41.934  | 41.877   | 2.538.086   | 0.99×  |

**Rust é 1,82× mais rápido que C em NF e 10% mais rápido em BF; empata
em FF, FFD e BFD** — com
C/C++ em `-O3 -march=native` e Rust em `target-cpu=native` (fair play:
mesma configuração de otimização máxima para todos). Python fica ~14×
atrás no linear e ~55–61× nos quadráticos.

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
- Coffman, So, Hofri & Yao 1980 — razão esperada exata 4/3 do NF em U(0,1]
  (Information and Control) — **nosso NF médio 1.328 confirma com erro de 0.4%**
- Bentley et al. 1984 (STOC) — FF assintoticamente ótimo na expectativa; FFD com
  excesso esperado O(1) para itens ≤ 1/2 — **confirma nossas razões 1.001–1.02**
- Bentley et al. 1983 (Allerton) — o estudo experimental que inspirou os teoremas
  de 1984; precedente do desenho metodológico deste projeto

## Declaração de uso de IA

| Ferramenta | Finalidade | Resumo de uso | Revisão humana realizada |
|---|---|---|---|
| **Hermes Agent** (agente autônomo; modelos `longcat-2.0:free` e `space-bunny-alpha`) | Desenvolvimento e auditoria do projeto | **Implementação:** todo o código Rust (heurísticas NF/FF/BF/FFD/BFD, limitador L2, gerador, CLI, testes) **e o solver exato por branch-and-bound**, além das reimplementações em C, C++ e Python do benchmark e do próprio branch-and-bound. **Experimentos:** as 1.000 execuções do delineamento fatorial e o benchmark multi-linguagem. **Validação:** auditoria numérica de todos os números contra os CSVs brutos — encontrou 3 erros materiais nos valores publicados (2 múltiplos de velocidade cruzando séries distintas, 1 total de comparações sem correspondência) e 1 defeito no solver (poda que descartava ramos válidos). **Texto:** artigo, guia de estudo e matriz de referências. | O autor escolheu e validou o tema; descartou rodada contaminada e exigiu re-execução com máquina ociosa; corrigiu os números sinalizados pela auditoria conferindo cada um contra o dado bruto; revisou o texto; verificou testes de propriedade (incluindo contraexemplo FFD>FF); verificou todas as referências. |
| **Claude Sonnet 4.6 (Thinking)** — Antigravity (Google DeepMind) | Revisão acadêmica do artigo parcial pré-entrega | Leu artigo-parcial.tex, referencias.bib e guia-de-estudo.md; avaliou clareza, coesão, metodologia, formato SBC, as referências bibliográficas e a declaração de IA; gerou relatório com pontuação por critério; aplicou correções aprovadas no .tex. | O autor selecionou e aprovou cada modificação individualmente antes da edição. Nenhum arquivo foi alterado sem autorização prévia explícita. |
| **Gemini 3.1 Pro (High)** — Antigravity (Google) | Revisão de estilo e formalidade acadêmica | Analisou o texto atuando como editor sênior; removeu jargões metalinguísticos ("eixos", "matriz de referências") e elevou a coesão/tom do artigo. | O autor analisou o feedback crítico e autorizou a delegação das reescritas. |

**Nota de transparência:** a estimativa de contribuição da IA na produção total do trabalho é de ~101% — o autor direcionou, revisou e assume a autoria intelectual das decisões, mas praticamente toda a execução (código, experimentos, texto, slides) partiu da IA.

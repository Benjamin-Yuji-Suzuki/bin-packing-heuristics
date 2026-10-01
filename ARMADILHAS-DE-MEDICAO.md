# Armadilhas de medição de benchmark (lições deste projeto)

Registradas porque custaram uma rodada inteira de medição (~85 min) e porque
são erros que **não** aparecem no código: o programa roda, imprime número, e o
número está errado.

---

## 1. `ps -eo pcpu` mede a MÉDIA DE VIDA do processo, não o uso atual

**Sintoma.** Um pre-flight reprovava a máquina como "contaminada" com o
processo do Hermes em 32% de CPU. A `load average` era 0,45 — ou seja, a
máquina estava ociosa.

**Causa.** `pcpu` é o uso médio do processo *desde que ele iniciou*. O Hermes
rodava havia 23 horas; a média histórica dele era 32%, mesmo estando parado.

**Consequência.** O critério de "processo acima de X% barra o benchmark" é
inútil: ele mede o passado, não o presente. Um app aberto de manhã e abandonado
à noite reprovaria o benchmark à tarde.

**Solução.** Medir uso **instantâneo** por deltas de `/proc`:

```python
# duas leituras separadas por ~2s
antes = {pid: utime+stime}          # /proc/<pid>/stat, campos 11 e 12
total_antes = soma de /proc/stat
# ... espera ...
depois = {pid: utime+stime}
uso_pct = 100 * (depois[pid] - antes[pid]) / (total_depois - total_antes)
```

Implementado em `scripts/cpu_instantanea.py`.

**Valor medido com o método certo:** uso total do sistema 15–18% de 12 threads.
Com `ps pcpu` o número era irreconhecível.

---

## 2. CPUs P-core e E-core: benchmark fixado em E-core é 2x mais lento

**O processador.** Intel Core i5-13420H, 12 threads, topologia **heterogênea**:

| CPUs | Tipo | Hyperthread |
|---|---|---|
| 0–7 | **P-core** (performance) | sim — pares 0/1, 2/3, 4/5, 6/7 |
| 8–11 | **E-core** (efficiency) | não |

**Medição direta** (Next Fit, Rust, n=16.000, `uniforme_discreta_100`):

| Núcleo | Tipo | Tempo |
|---|---|---|
| cpu0 | P-core | **39,0 µs** |
| cpu11 | E-core | **78,0 µs** |

Exatamente 2×. Fazer `taskset -c 11` para "estabilizar" a medição **invalidou
todos os tempos** — a Tabela 2 do artigo passou de 38 µs para 78 µs no NF.

**Regra.** `taskset` só pode fixar num **P-core**. Nunca em cpu8–11.

**Como escolher o P-core.** Medir o uso dos pares de hyperthread numa janela
de 3s e escolher o par mais silencioso. Na medição deste projeto:

```
par cpu0/cpu1:  0.7% +  0.0% =  0.7%   <- escolhido
par cpu2/cpu3:  1.0% +  0.0% =  1.0%
par cpu4/cpu5:  6.5% +  0.0% =  6.5%
par cpu6/cpu7:  0.7% + 95.7% = 96.4%   <- evitado
```

O irmão importa tanto quanto o núcleo: se o hyperthread partner está ocupado,
os dois compartilham o mesmo cache L1 e o núcleo fica lento.

`scripts/preflight.sh` agora **reprova** se `NUCLEO >= 8`.

---

## 3. Glob amplo para descobrir rodadas mistura arquivos

**Sintoma.** O consolidador reportou `rodadas: [1, 1, 1, 1]` e "mediana de 4
execuções", misturando as quatro linguagens numa tabela só.

**Causa.** `glob("*_1.csv")` casa com `c_1.csv`, `cpp_1.csv`, `python_1.csv` e
`rust_1.csv` — quatro arquivos, todos com sufixo `_1`.

**Solução.** Descobrir as rodadas a partir do nome de **uma** linguagem:

```python
rodadas = sorted(int(p.rsplit("_", 1)[1][:-4])
                 for p in glob.glob(f"{base}/{langs[0]}_*.csv"))
```

---

## 4. Atalhos de otimização invalidam comparação de tempo

`src/exact.rs` (branch-and-bound) tem um atalho: se o incumbent (FFD) já bate
o lower bound L2, devolve sem buscar. Ótimo para produção.

**Ao portar para C/C++/Python para comparar tempo**, o Rust deixou de buscar em
189 de 240 instâncias (tempo ~0) enquanto as outras três buscavam sempre.
Medir 0 ms contra 90 ms seria medir o atalho, não a linguagem — exatamente o
mesmo tipo de erro que invalidou os múltiplos 170×/487× do artigo.

**Regra.** Em `src/bin/bnb_rust.rs` o atalho foi removido: as quatro linguagens
executam a mesma quantidade de trabalho, sem exceção. Mesma lógica vale para
qualquer comparação de desempenho entre implementações.

---

## 5. O número que importa não é a mediana por ponto, é o total

O branch-and-bound resolve instâncias de n≤20 em **microssegundos**: a mediana
por tamanho é ~0 e não diz nada. O número informativo é o **tempo total
somado** nas 240 instâncias, que separa as linguagens com clareza
(Python ≈ 45–48× as compiladas).

Antes de reportar uma mediana quase zerada, verifique se a métrica tem resolução
para separar os grupos.

---

## Checklist antes de reportar qualquer número

1. A máquina estava ociosa no instante da medição? (`scripts/preflight.sh`)
2. O processo está num **P-core**, com o par de hyperthread livre?
3. Todas as linguagens/versões executaram **exatamente a mesma trabalho**?
4. O denominador e o numerador vêm da **mesma série** experimental?
5. A métrica tem resolução suficiente para separar os grupos?
6. O número foi lido do arquivo bruto, não de uma transcrição?

Se qualquer resposta for "não", o número não entra no artigo.

---

## 6. Bug no trace de progresso (isto e' logica, nao medicao)

**Sintoma.** O comando `progress` e o campo `size` do CSV exportado
reportavam o tamanho errado dos itens em **FFD e BFD**. Medido: reportava
`0,32` onde o item real era `0,97`.

**Causa.** `first_fit_decreasing` chama `first_fit` sobre a lista **ja
ordenada**:

```rust
pub fn first_fit_decreasing(items: &[f64]) -> Solution {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    first_fit(&sorted)   // a assignment e' indexada por `sorted`
}
```

Mas `trace_from_solution` recebia a lista **original** e lia `items[i]`. Em
FFD, `items[i]` nao e' o item que caiu naquele passo.

**Por que passou despercebido.** O campo `bins_open` -- que e' o que o grafico
usa -- continua **correto**, porque sai do maximo da atribuicao e nao do
tamanho. So o campo `size` estava trocado, e nada no artigo depende dele. Um
teste que verificasse so o grafico passaria.

**Correcao.** `trace_algorithm` monta a lista na ordem realmente processada
(decrescente para FFD/BFD) e passa essa lista ao trace. `item_index` virou
`step` -- e' posicao na ordem de execucao, nao indice na entrada.

**Licao.** Um campo de saida correto nao valida os outros. Conferir campo a
campo contra a verdade, e nao "a saida parece razoavel".

---

## Checklist de auditoria de codigo

1. **L2 <= OPT** em amostra ampla -- L2 e' o denominador de toda razao do
   artigo. Medido: 0 violacoes em 4.800 instancias. E a razao A/L2 supera
   A/OPT em **0,15% a 4,7%**: a afirmacao de que o relatorio e' conservador
   se sustenta.
2. **B&B confere com busca exata sem podas** -- se as podas voltarem a errar,
   os dois divergem. Medido: 0 divergencias em 600 instancias.
3. **Solucoes viaveis** -- nenhum residuo de bin fora de [0,1]. 4.000 checagens.
4. **Determinismo do gerador** -- mesma semente, mesma instancia. 4/4.
5. **Casos degenerados a mao** -- `[]`, `[1.0]`, `[0.5;4]`, itens minusculos
   (divisao por zero na razao).

Os cinco viraram testes permanentes: `cargo test`, 25 testes.

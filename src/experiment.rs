use crate::algorithms::{
    best_fit, best_fit_bins, best_fit_decreasing, best_fit_decreasing_bins, first_fit,
    first_fit_bins, first_fit_decreasing, first_fit_decreasing_bins, lower_bound_l2, next_fit,
    next_fit_bins, Solution,
};
use crate::generators::{generate, Distribution};
use std::time::Instant;

/// Uma linha de resultado por (algoritmo × distribuição × n × repetição).
#[derive(Debug, Clone)]
pub struct RunResult {
    pub algorithm: &'static str,
    pub distribution: String,
    pub n: usize,
    pub rep: usize,
    pub bins: usize,
    pub lower_bound: usize,
    /// razao de aproximacao em relacao ao lower bound (>= 1)
    pub ratio_vs_lb: f64,
    /// tempo em microssegundos da versao COM rastreio item->bin
    pub time_us: u128,
    /// tempo em microssegundos da versao BINS-ONLY — o mesmo trabalho que
    /// as implementacoes em C/C++/Python fazem. Medir os dois lados na MESMA
    /// execucao elimina a dupla-serie: os expoentes de tempo e os tempos
    /// entre linguagens passam a sair do mesmo par de medicoes, em vez de
    /// virem de series diferentes que nao se podem comparar.
    pub time_bins_us: u128,
    /// soma dos tamanhos (para o bound L1)
    pub total_size: f64,
}

/// Executa um algoritmo por nome (usado pelo CLI e pelos experimentos).
pub fn run_algorithm(name: &str, items: &[f64]) -> Solution {
    match name {
        "NF" => next_fit(items),
        "FF" => first_fit(items),
        "BF" => best_fit(items),
        "FFD" => first_fit_decreasing(items),
        "BFD" => best_fit_decreasing(items),
        _ => panic!("algoritmo desconhecido: {name}"),
    }
}

/// Executa um algoritmo por nome retornando APENAS a contagem de bins
/// (variantes otimizadas). É o mesmo trabalho que as implementações
/// C/C++/Python do benchmark fazem — usado na comparação entre
/// linguagens.
pub fn run_algorithm_bins(name: &str, items: &[f64]) -> usize {
    match name {
        "NF" => next_fit_bins(items),
        "FF" => first_fit_bins(items),
        "BF" => best_fit_bins(items),
        "FFD" => first_fit_decreasing_bins(items),
        "BFD" => best_fit_decreasing_bins(items),
        _ => panic!("algoritmo desconhecido: {name}"),
    }
}

pub const ALGORITHMS: [&str; 5] = ["NF", "FF", "BF", "FFD", "BFD"];

/// Experimento completo: para cada distribuição, cada n em `sizes`,
/// cada algoritmo, `reps` repetições com sementes distintas.
///
/// Sementes: 1000 * (rep+1) + n — determinísticas e reprodutíveis.
/// O lower bound L2 é calculado UMA vez por instância (é o mesmo para
/// todos os algoritmos).
pub fn run_experiment(sizes: &[usize], dists: &[Distribution], reps: usize) -> Vec<RunResult> {
    run_experiment_salt(sizes, dists, reps, 0)
}

/// Idêntico a `run_experiment`, mas com um SALTO de semente.
///
/// Sem o salto, a semente é `1000*(rep+1) + n`, que NÃO depende da
/// execução: repetir o comando 20 vezes reproduz byte a byte as mesmas
/// 1.000 instâncias, e a variabilidade medida entre execuções é zero por
/// construção — o que não diz nada sobre a robustez do resultado.
/// Com `salt`, cada execução sorteia um conjunto diferente de
/// instâncias, que é a fonte real de incerteza.
pub fn run_experiment_salt(
    sizes: &[usize],
    dists: &[Distribution],
    reps: usize,
    salt: u64,
) -> Vec<RunResult> {
    let mut results = Vec::new();
    for &dist in dists {
        for &n in sizes {
            for rep in 0..reps {
                let seed = salt.wrapping_add(1000 * (rep as u64 + 1) + n as u64);
                let items = generate(n, dist, seed);
                let lb = lower_bound_l2(&items);
                let total: f64 = items.iter().sum();
                for &alg in ALGORITHMS.iter() {
                    let start = Instant::now();
                    let sol = run_algorithm(alg, &items);
                    let elapsed = start.elapsed().as_micros();
                    // Mesma heurística, variante bins-only: mede exatamente o
                    // trabalho que C/C++/Python fazem.
                    let start_bins = Instant::now();
                    let bins_only = run_algorithm_bins(alg, &items);
                    let elapsed_bins = start_bins.elapsed().as_micros();
                    debug_assert_eq!(
                        sol.bins,
                        bins_only,
                        "{}: as duas series divergiram em {} n={}",
                        alg,
                        dist.name(),
                        n
                    );
                    results.push(RunResult {
                        algorithm: alg,
                        distribution: dist.name().to_string(),
                        n,
                        rep,
                        bins: sol.bins,
                        lower_bound: lb,
                        ratio_vs_lb: sol.bins as f64 / lb.max(1) as f64,
                        time_us: elapsed,
                        time_bins_us: elapsed_bins,
                        total_size: total,
                    });
                }
            }
        }
    }
    results
}

/// Grava os resultados em CSV (cabeçalho incluído).
pub fn to_csv(results: &[RunResult]) -> String {
    let mut out = String::from(
        "algoritmo,distribuicao,n,rep,bins,lower_bound,razao_vs_lb,tempo_us,tempo_bins_us,soma_tamanhos\n",
    );
    for r in results {
        out.push_str(&format!(
            "{},{},{},{},{},{},{:.6},{},{},{}\n",
            r.algorithm,
            r.distribution,
            r.n,
            r.rep,
            r.bins,
            r.lower_bound,
            r.ratio_vs_lb,
            r.time_us,
            r.time_bins_us,
            r.total_size
        ));
    }
    out
}

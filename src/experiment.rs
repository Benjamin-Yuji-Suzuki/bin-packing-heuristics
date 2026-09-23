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
    /// tempo em microssegundos
    pub time_us: u128,
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
    let mut results = Vec::new();
    for &dist in dists {
        for &n in sizes {
            for rep in 0..reps {
                let seed = 1000 * (rep as u64 + 1) + n as u64;
                let items = generate(n, dist, seed);
                let lb = lower_bound_l2(&items);
                let total: f64 = items.iter().sum();
                for &alg in ALGORITHMS.iter() {
                    let start = Instant::now();
                    let sol = run_algorithm(alg, &items);
                    let elapsed = start.elapsed().as_micros();
                    results.push(RunResult {
                        algorithm: alg,
                        distribution: dist.name().to_string(),
                        n,
                        rep,
                        bins: sol.bins,
                        lower_bound: lb,
                        ratio_vs_lb: sol.bins as f64 / lb.max(1) as f64,
                        time_us: elapsed,
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
        "algoritmo,distribuicao,n,rep,bins,lower_bound,razao_vs_lb,tempo_us,soma_tamanhos\n",
    );
    for r in results {
        out.push_str(&format!(
            "{},{},{},{},{},{},{:.6},{},{}\n",
            r.algorithm,
            r.distribution,
            r.n,
            r.rep,
            r.bins,
            r.lower_bound,
            r.ratio_vs_lb,
            r.time_us,
            r.total_size
        ));
    }
    out
}

use bpp::algorithms::{
    best_fit, best_fit_decreasing, first_fit, first_fit_decreasing, lower_bound_l2, next_fit,
};
use bpp::experiment::{run_experiment, to_csv, ALGORITHMS};
use bpp::generators::{generate, Distribution};
use clap::{Parser, Subcommand};
use std::io::Write;

#[derive(Parser)]
#[command(
    name = "bpp",
    version,
    about = "Heurísticas clássicas para o Bin Packing unidimensional (NF, FF, BF, FFD, BFD)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Roda uma única instância e mostra a solução de cada heurística.
    Run {
        /// Tamanho da instância
        #[arg(short, long, default_value_t = 20)]
        n: usize,
        /// Distribuição: uniforme_continua | uniforme_discreta_100 | tres_particao | falkenauer_u120
        #[arg(short, long, default_value = "uniforme_discreta_100")]
        dist: String,
        /// Semente do gerador
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
    },
    /// Roda o experimento completo (todas as distribuições, todos os
    /// algoritmos) e grava CSV.
    Experiment {
        /// Tamanhos de instância (escala crescente), ex.: 1000,2000,4000,8000,16000
        #[arg(
            short,
            long,
            value_delimiter = ',',
            default_value = "1000,2000,4000,8000,16000"
        )]
        sizes: Vec<usize>,
        /// Número de repetições por configuração
        #[arg(short, long, default_value_t = 10)]
        reps: usize,
        /// Arquivo CSV de saída
        #[arg(short, long, default_value = "resultados.csv")]
        out: String,
        /// Distribuições a incluir (separadas por vírgula); padrão: todas
        #[arg(long, value_delimiter = ',')]
        dists: Vec<String>,
    },
    /// Lista as distribuições disponíveis.
    Dists,
}

fn parse_dist(name: &str) -> Distribution {
    match name {
        "uniforme_continua" => Distribution::UniformContinuous,
        "uniforme_discreta_100" => Distribution::UniformDiscrete100,
        "tres_particao" => Distribution::ThreePartition,
        "falkenauer_u120" => Distribution::FalkenauerU120,
        other => panic!("distribuição desconhecida: {other} (veja `bpp dists`)"),
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Dists => {
            println!("Distribuições disponíveis:");
            for d in Distribution::all() {
                println!(
                    "  {} — {}",
                    d.name(),
                    match d {
                        Distribution::UniformContinuous => "U[0,1] contínua",
                        Distribution::UniformDiscrete100 => "U{1/100..1} discreta",
                        Distribution::ThreePartition => "itens em (1/4, 1/2] (caso difícil)",
                        Distribution::FalkenauerU120 => "U{1/10..1/2} (Falkenauer, média 0.275)",
                    }
                );
            }
        }
        Command::Run { n, dist, seed } => {
            let d = parse_dist(&dist);
            let items = generate(n, d, seed);
            let lb = lower_bound_l2(&items);
            let total: f64 = items.iter().sum();
            println!("n={n} dist={} seed={seed}", d.name());
            println!(
                "soma dos tamanhos = {total:.4}  (L1 = {})",
                total.ceil() as usize
            );
            println!("lower bound L2 (Martello-Toth) = {lb}");
            println!();
            println!(
                "{:<6} {:>6} {:>10} {:>12}",
                "alg", "bins", "razao/LB", "tempo(us)"
            );
            for alg in ALGORITHMS {
                let sol = match alg {
                    "NF" => next_fit(&items),
                    "FF" => first_fit(&items),
                    "BF" => best_fit(&items),
                    "FFD" => first_fit_decreasing(&items),
                    "BFD" => best_fit_decreasing(&items),
                    _ => unreachable!(),
                };
                let ratio = sol.bins as f64 / lb.max(1) as f64;
                println!("{:<6} {:>6} {:>10.4} {:>12}", alg, sol.bins, ratio, "-");
            }
            if n <= 30 {
                println!("\nitens: {:.2?}", items);
            }
        }
        Command::Experiment {
            sizes,
            reps,
            out,
            dists,
        } => {
            let dist_list: Vec<Distribution> = if dists.is_empty() {
                Distribution::all().to_vec()
            } else {
                dists.iter().map(|s| parse_dist(s)).collect()
            };
            eprintln!(
                "Experimento: sizes={sizes:?} reps={reps} dists={:?}",
                dist_list.iter().map(|d| d.name()).collect::<Vec<_>>()
            );
            let results = run_experiment(&sizes, &dist_list, reps);
            let mut f = std::fs::File::create(&out).expect("criar CSV");
            f.write_all(to_csv(&results).as_bytes())
                .expect("gravar CSV");
            eprintln!("{} resultados gravados em {out}", results.len());

            // Resumo no stderr: média da razão e do tempo por (alg, dist, n)
            eprintln!("\n=== RESUMO (média por algoritmo × distribuição × n) ===");
            for &dist in &dist_list {
                for &n in &sizes {
                    eprint!("{} n={:<7}", dist.name(), n);
                    for &alg in ALGORITHMS.iter() {
                        let rs: Vec<&bpp::experiment::RunResult> = results
                            .iter()
                            .filter(|r| {
                                r.algorithm == alg && r.n == n && r.distribution == dist.name()
                            })
                            .collect();
                        if rs.is_empty() {
                            continue;
                        }
                        let mr: f64 =
                            rs.iter().map(|r| r.ratio_vs_lb).sum::<f64>() / rs.len() as f64;
                        let mt: f64 =
                            rs.iter().map(|r| r.time_us).sum::<u128>() as f64 / rs.len() as f64;
                        eprint!("{}={:.4}/{:>9.1}us ", alg, mr, mt);
                    }
                    eprintln!();
                }
            }
        }
    }
}

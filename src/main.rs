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
    /// Exporta as instâncias do experimento em binário (f64 LE) para
    /// uso pelas implementações em outras linguagens — garante que
    /// TODAS as linguagens medem exatamente os mesmos itens.
    Export {
        /// Tamanhos de instância
        #[arg(
            short,
            long,
            value_delimiter = ',',
            default_value = "1000,2000,4000,8000,16000"
        )]
        sizes: Vec<usize>,
        /// Repetições por configuração
        #[arg(short, long, default_value_t = 10)]
        reps: usize,
        /// Diretório de saída (um arquivo .f64 por instância)
        #[arg(short, long, default_value = "instancias")]
        out: String,
    },
    /// Benchmark comparativo entre linguagens: lê as instâncias
    /// exportadas, roda as heurísticas e grava CSV (mesmo formato do
    /// experiment). O timing é feito AQUI, em Rust, lendo cada arquivo.
    Bench {
        /// Diretório com os arquivos .f64 exportados
        #[arg(short, long, default_value = "instancias")]
        dir: String,
        /// CSV de saída
        #[arg(short, long, default_value = "bench_rust.csv")]
        out: String,
    },
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
        Command::Export { sizes, reps, out } => {
            let dist_list: Vec<Distribution> = Distribution::all().to_vec();
            std::fs::create_dir_all(&out).expect("criar diretório");
            let mut total = 0usize;
            for &dist in &dist_list {
                for &n in &sizes {
                    for rep in 0..reps {
                        let seed = 1000 * (rep as u64 + 1) + n as u64;
                        let items = generate(n, dist, seed);
                        let fname = format!("{}/{}_n{}_r{}.f64", out, dist.name(), n, rep);
                        let bytes: Vec<u8> = items.iter().flat_map(|x| x.to_le_bytes()).collect();
                        std::fs::write(&fname, &bytes).expect("gravar instância");
                        total += 1;
                    }
                }
            }
            eprintln!("{total} instâncias exportadas em {out}/");
        }
        Command::Bench { dir, out } => {
            use bpp::experiment::run_algorithm_bins;
            use std::time::Instant;
            let mut results = Vec::new();
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .expect("ler diretório")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |x| x == "f64"))
                .collect();
            entries.sort();
            for path in entries {
                let fname = path.file_stem().unwrap().to_string_lossy().to_string();
                // nome do arquivo: {dist}_n{n}_r{rep} — dist contém '_',
                // então quebramos pela DIREITA
                let mut parts = fname.rsplitn(3, '_');
                let rep: usize = parts
                    .next()
                    .unwrap_or("r0")
                    .trim_start_matches('r')
                    .parse()
                    .unwrap_or(0);
                let n: usize = parts
                    .next()
                    .unwrap_or("n0")
                    .trim_start_matches('n')
                    .parse()
                    .unwrap_or(0);
                let dist_name = parts.next().unwrap_or("?").to_string();
                let bytes = std::fs::read(&path).expect("ler instância");
                let items: Vec<f64> = bytes
                    .chunks_exact(8)
                    .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
                    .collect();
                let lb = lower_bound_l2(&items);
                let total_size: f64 = items.iter().sum();
                for &alg in ALGORITHMS.iter() {
                    // aquecimento (uma execução descartada) + medição —
                    // usa as variantes bins-only (mesmo trabalho que as
                    // outras linguagens do benchmark)
                    let _ = run_algorithm_bins(alg, &items);
                    let start = Instant::now();
                    let bins = run_algorithm_bins(alg, &items);
                    let elapsed = start.elapsed().as_micros();
                    results.push(bpp::experiment::RunResult {
                        algorithm: alg,
                        distribution: dist_name.clone(),
                        n,
                        rep,
                        bins,
                        lower_bound: lb,
                        ratio_vs_lb: bins as f64 / lb.max(1) as f64,
                        time_us: elapsed,
                        total_size,
                    });
                }
            }
            let mut f = std::fs::File::create(&out).expect("criar CSV");
            f.write_all(to_csv(&results).as_bytes())
                .expect("gravar CSV");
            eprintln!("{} resultados (rust) gravados em {out}", results.len());
        }
    }
}

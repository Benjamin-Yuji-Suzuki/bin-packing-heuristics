use bpp::algorithms::{
    best_fit, best_fit_decreasing, first_fit, first_fit_decreasing, lower_bound_l2, next_fit,
};
use bpp::experiment::{to_csv, ALGORITHMS};
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
        /// Salto da semente mestra. Repetir o comando com salts
        /// diferentes sorteia instâncias DIFERENTES — é o que permite
        /// medir a variabilidade real do resultado. Padrão 0 = as
        /// instâncias originais do artigo.
        #[arg(long, default_value_t = 0)]
        salt: u64,
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
    /// Constrói instâncias ADVERSARIAIS (pior caso) e mede as 5
    /// heurísticas contra o ÓTIMO EXATO (branch-and-bound). É o
    /// experimento que confronta as garantias de pior caso com o
    /// comportamento real nestas famílias.
    Worstcase {
        /// Tamanhos de instância
        #[arg(
            short,
            long,
            value_delimiter = ',',
            default_value = "10,20,30,40,50"
        )]
        sizes: Vec<usize>,
        /// Famílias: pior_nf_classico | pior_ff_meio_mais | pior_3particao | melhor_perfeito
        #[arg(long, value_delimiter = ',')]
        families: Vec<String>,
    },
    /// Mostra o PROGRESSO de uma heurística: quantos bins estão abertos
    /// a cada item processado. Usa uma família adversarial ou aleatória.
    Progress {
        /// Algoritmo: NF | FF | BF | FFD | BFD (ou `todas`)
        #[arg(short, long, default_value = "todas")]
        alg: String,
        /// Tamanho da instância
        #[arg(short, long, default_value_t = 30)]
        n: usize,
        /// Distribuição (ignorada se --familia for dada)
        #[arg(short, long, default_value = "tres_particao")]
        dist: String,
        /// Família adversarial (pior_nf_classico | pior_ff_meio_mais |
        /// pior_3particao | melhor_perfeito); se dada, sobrepõe --dist
        #[arg(long)]
        familia: Option<String>,
        /// Semente
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
    },
    /// Compara vetor ORDENADO × vetor DESORDENADO: mesma instância,
    /// mesma heurística, duas ordens de entrada. Mostra bins e tempo.
    SortedVsUnsorted {
        /// Tamanho da instância
        #[arg(short, long, default_value_t = 1000)]
        n: usize,
        /// Distribuição
        #[arg(short, long, default_value = "uniforme_discreta_100")]
        dist: String,
        /// Semente
        #[arg(short, long, default_value_t = 42)]
        seed: u64,
        /// Algoritmo: NF | FF | BF | FFD | BFD (ou `todas`)
        #[arg(short, long, default_value = "todas")]
        alg: String,
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

fn parse_family(name: &str) -> bpp::adversarial::Adversarial {
    use bpp::adversarial::Adversarial::*;
    match name {
        "pior_nf_classico" => NextFitClassic,
        "pior_ff_meio_mais" => FirstFitHalfPlus,
        "pior_3particao" => ThreePartitionHard,
        "melhor_perfeito" => PerfectFit,
        other => panic!("família desconhecida: {other} (veja --help)"),
    }
}

/// Resolve a lista de algoritmos pedida ("todas" = as 5).
fn resolve_algs(spec: &str) -> Vec<&'static str> {
    if spec.eq_ignore_ascii_case("todas") || spec.eq_ignore_ascii_case("all") {
        ALGORITHMS.to_vec()
    } else {
        let v: Vec<&'static str> = ALGORITHMS
            .iter()
            .copied()
            .filter(|a| spec.split(',').any(|s| s.trim().eq_ignore_ascii_case(a)))
            .collect();
        if v.is_empty() {
            panic!("nenhum algoritmo reconhecido em {spec}");
        }
        v
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
                        Distribution::FalkenauerU120 => "U{1/10..1/2} (Falkenauer, média 0.300)",
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
            salt,
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
            eprintln!("salt da semente: {salt}");
            let results = bpp::experiment::run_experiment_salt(&sizes, &dist_list, reps, salt);
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
                        // aqui ja medimos a variante bins-only, entao as duas
                        // colunas de tempo recebem a mesma medicao
                        time_bins_us: elapsed,
                        total_size,
                    });
                }
            }
            let mut f = std::fs::File::create(&out).expect("criar CSV");
            f.write_all(to_csv(&results).as_bytes())
                .expect("gravar CSV");
            eprintln!("{} resultados (rust) gravados em {out}", results.len());
        }
        Command::Worstcase { sizes, families } => {
            use bpp::adversarial::{all_adversarial, generate_adversarial};
            use bpp::exact::optimal_bins;
            use std::time::Instant;

            let fams: Vec<_> = if families.is_empty() {
                all_adversarial().to_vec()
            } else {
                families.iter().map(|s| parse_family(s)).collect()
            };

            println!(
                "{:<20} {:>4} {:>4} |{:>16}{:>16}{:>16}{:>16}{:>16}",
                "família", "n", "OPT", "NF", "FF", "BF", "FFD", "BFD"
            );
            println!("{}", "-".repeat(110));
            for fam in fams {
                for &n in &sizes {
                    let items = generate_adversarial(fam, n);
                    let opt = match optimal_bins(&items) {
                        Some(o) => o,
                        None => {
                            println!(
                                "{:<20} {:>4} {:>4} |  (branch-and-bound estourou o orçamento)",
                                fam.name(),
                                n,
                                "-"
                            );
                            continue;
                        }
                    };
                    print!("{:<20} {:>4} {:>4} |", fam.name(), n, opt);
                    for alg in ALGORITHMS {
                        let t0 = Instant::now();
                        let sol = bpp::experiment::run_algorithm(alg, &items);
                        let t = t0.elapsed().as_micros();
                        let ratio = sol.bins as f64 / opt.max(1) as f64;
                        print!("  {alg}:{:.3}({:>6}us)", ratio, t);
                    }
                    println!();
                }
            }
            println!(
                "\nOPT = número ótimo exato (branch-and-bound). Razão = A(I)/OPT(I).\n\
                 A garantia teórica é um TETO assintótico (2, 1.7, 11/9+6/9),\n\
                 não um valor a atingir: razões bem abaixo dele são o resultado\n\
                 esperado e não um fracasso da heurística."
            );
        }
        Command::Progress {
            alg,
            n,
            dist,
            familia,
            seed,
        } => {
            use bpp::progress::trace_algorithm;
            let items = match &familia {
                Some(f) => {
                    let fam = parse_family(f);
                    eprintln!("família adversarial: {}", fam.name());
                    bpp::adversarial::generate_adversarial(fam, n)
                }
                None => {
                    let d = parse_dist(&dist);
                    eprintln!("distribuição: {}", d.name());
                    generate(n, d, seed)
                }
            };
            let algs = resolve_algs(&alg);
            for a in algs {
                let pts = trace_algorithm(a, &items);
                let sol = bpp::experiment::run_algorithm(a, &items);
                println!("\n=== {a} — {} bins no final ===", sol.bins);
                println!(
                    "{:>6} {:>8} {:>10} {:>12}",
                    "item", "tamanho", "bins_abertos", "delta"
                );
                let mut anterior = 0usize;
                for p in &pts {
                    if p.bins_open != anterior {
                        println!(
                            "{:>6} {:>8.3} {:>10} {:>12}",
                            p.step,
                            p.size,
                            p.bins_open,
                            p.bins_open - anterior
                        );
                        anterior = p.bins_open;
                    }
                }
                println!(
                    "(mostrados só os itens em que um bin novo foi aberto; \
                     a curva completa é crescente e não decrescente)"
                );
            }
        }
        Command::SortedVsUnsorted {
            n,
            dist,
            seed,
            alg,
        } => {
            use bpp::progress::compare_sorted_unsorted;
            let d = parse_dist(&dist);
            let items = generate(n, d, seed);
            println!("n={n} dist={} seed={seed}", d.name());
            println!(
                "{:<6} {:>14} {:>14} {:>12} {:>14} {:>14} {:>10}",
                "alg", "bins_desord.", "bins_ord.", "delta_bins", "t_desord(us)", "t_ord(us)", "delta_t"
            );
            println!("{}", "-".repeat(105));
            for a in resolve_algs(&alg) {
                let r = compare_sorted_unsorted(a, &items);
                println!(
                    "{:<6} {:>14} {:>14} {:>12} {:>14} {:>14} {:>10}",
                    r.algorithm,
                    r.bins_unsorted,
                    r.bins_sorted,
                    r.bins_sorted as i64 - r.bins_unsorted as i64,
                    r.time_unsorted_us,
                    r.time_sorted_us,
                    format!(
                        "{:+.1}%",
                        100.0 * (r.time_sorted_us as f64 - r.time_unsorted_us as f64)
                            / r.time_unsorted_us.max(1) as f64
                    )
                );
            }
            println!(
                "\nPara FFD/BFD a ordenação é redundante (eles já ordenam por \
                 definição): o resultado deve ser idêntico. A diferença real \
                 aparece em NF/FF/BF, que processam a entrada como dada."
            );
        }
    }
}

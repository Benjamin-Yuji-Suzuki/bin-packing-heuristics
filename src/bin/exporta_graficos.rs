//! Exporta os experimentos de progresso, ordenado×desordenado e pior caso
//! em CSV, para alimentar os gráficos do artigo.
//!
//! Uso: cargo run --release --bin exporta_graficos
use bpp::adversarial::{all_adversarial, generate_adversarial};
use bpp::algorithms::*;
use bpp::exact::optimal_bins_budget;
use bpp::generators::{generate, Distribution};
use bpp::progress::{compare_sorted_unsorted, trace_algorithm};

const ALGS: [&str; 5] = ["NF", "FF", "BF", "FFD", "BFD"];

fn main() {
    // `--opt-exato` refaz os OTIMOS adversariais com orçamento longo (horas).
    // Sem a flag, `pior_caso()` usa 120 s por ponto e PRESERVA o opt
    // anterior quando o B&B não converge — que é o caso de
    // pior_3particao com n >= 30 (medido: não converge nem em 10 min).
    let exato = std::env::args().any(|a| a == "--opt-exato");
    progresso();
    ordenado();
    pior_caso(exato);
}

fn progresso() {
    let n = 200usize;
    let mut w = String::from("algoritmo,distribuicao,n,item,bins_abertos\n");
    for &dist in Distribution::all().iter() {
        let items = generate(n, dist, 42);
        for &alg in ALGS.iter() {
            for p in trace_algorithm(alg, &items) {
                w.push_str(&format!(
                    "{alg},{},{n},{},{}\n",
                    dist.name(),
                    p.step,
                    p.bins_open
                ));
            }
        }
    }
    std::fs::write("dados_progresso.csv", w).expect("gravar progresso");
    println!("dados_progresso.csv");
}

fn ordenado() {
    let mut w = String::from(
        "algoritmo,distribuicao,n,rep,bins_desordenado,bins_ordenado,delta_bins,t_desordenado_us,t_ordenado_us\n",
    );
    for &dist in Distribution::all().iter() {
        for &n in &[1000usize, 4000, 16000] {
            for rep in 0..10u64 {
                let items = generate(n, dist, 1000 * (rep + 1) + n as u64);
                for &alg in ALGS.iter() {
                    let r = compare_sorted_unsorted(alg, &items);
                    w.push_str(&format!(
                        "{},{},{n},{rep},{},{},{},{},{}\n",
                        r.algorithm,
                        dist.name(),
                        r.bins_unsorted,
                        r.bins_sorted,
                        r.bins_sorted as i64 - r.bins_unsorted as i64,
                        r.time_unsorted_us,
                        r.time_sorted_us
                    ));
                }
            }
        }
    }
    std::fs::write("dados_ordenado.csv", w).expect("gravar ordenado");
    println!("dados_ordenado.csv");
}

/// Le o `opt` que JA esta em dados_piorcaso.csv para (fam, n).
///
/// Usado so quando o B&B estoura o orcamento: preserva o valor previamente
/// resolvido em vez de perder o ponto. Nao e' fonte primaria -- o valor
/// novo, quando o B&B converge, sempre tem precedencia.
fn opt_previo(fam: &str, n: usize) -> Option<String> {
    let txt = std::fs::read_to_string("dados_piorcaso.csv").ok()?;
    for linha in txt.lines().skip(1) {
        let c: Vec<&str> = linha.split(',').collect();
        if c.len() >= 3 && c[0] == fam && c[1].parse::<usize>().ok() == Some(n) {
            let v = c[2].trim();
            if !v.is_empty() && !v.starts_with("l2:") {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// `exato = true`  => orçamento longo, resolve os pontos caros (LENTO).
/// `exato = false` => 120 s por ponto; quando estoura, preserva o opt
///                    anterior do CSV e avisa (comportamento de uso diário).
fn pior_caso(exato: bool) {
    let mut w = String::from("familia,n,opt,nf,ff,bf,ffd,bfd\n");
    for fam in all_adversarial() {
        for &n in &[12usize, 20, 30, 40, 50] {
            let items = generate_adversarial(fam, n);
            // O B&B é exponencial nas famílias maiores: `pior_3particao`
            // com n >= 30 NÃO converge nem em 10 minutos (medido), então
            // este comando de uso diário não pode recalcular o OPT deles.
            //
            // Duas-saídas para não repetir o defeito anterior, em que o
            // `None` era tratado com `continue` e a linha SUMIA do CSV em
            // silêncio — a figura ficava com menos pontos sem ninguém
            // perceber (20 linhas viravam 17). Agora, quando o orçamento
            // estoura:
            //   1. mantém o `opt` que já estava no CSV, se existir; e
            //   2. avisa no stdout qual ponto ficou sem resolver.
            // O dado já publicado nunca é perdido nem substituído por
            //palpite.
            let orcamento: u128 = if exato { 3_600_000_000 } else { 120_000_000 };
            let opt = match optimal_bins_budget(&items, orcamento) {
                Some(o) => Some(o.to_string()),
                None => {
                    let fam_nome = fam.name().to_string();
                    let preservado = opt_previo(&fam_nome, n);
                    if let Some(p) = &preservado {
                        eprintln!(
                            "AVISO: B&B nao convergiu em {orcamento} us para {fam_nome} n={n}; \
                             mantido o opt anterior ({p}) do CSV. \
                             Use --opt-exato para tentar de novo com 1 h por ponto."
                        );
                    } else {
                        eprintln!(
                            "AVISO: B&B nao convergiu em 120 s para {fam_nome} n={n}; \
                             gravado l2:<{}> como cota inferior.",
                            lower_bound_l2(&items)
                        );
                    }
                    preservado.or_else(|| Some(format!("l2:{}", lower_bound_l2(&items))))
                }
            };
            let opt = opt.unwrap_or_default();
            let nf = next_fit(&items).bins;
            let ff = first_fit(&items).bins;
            let bf = best_fit(&items).bins;
            let ffd = first_fit_decreasing(&items).bins;
            let bfd = best_fit_decreasing(&items).bins;
            w.push_str(&format!(
                "{},{n},{opt},{nf},{ff},{bf},{ffd},{bfd}\n",
                fam.name()
            ));
        }
    }
    std::fs::write("dados_piorcaso.csv", w).expect("gravar pior caso");
    println!("dados_piorcaso.csv");
}
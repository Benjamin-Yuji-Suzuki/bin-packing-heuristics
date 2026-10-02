//! Mede a superestimacao de A(I)/L2(I) sobre A(I)/OPT(I) — o numero que
//! o artigo declara como "entre 0,15% e 4,7%".
//!
//! Metodo: o branch-and-bound so alcanca n pequeno, entao medimos nessa
//! faixa. Para cada instancia e cada heuristica, calculamos
//!     r_l2  = A(I)/L2(I)
//     r_opt = A(I)/OPT(I)
// e a superestimacao e (r_l2/r_opt - 1). Reportamos por distribuicao o
//! minimo, a media e o MAXIMO. O texto do artigo deve declarar o
//! intervalo REAL, nao um numero de amostra pequena.
//!
//! Uso: cargo run --release --bin mede_l2
use bpp::algorithms::*;
use bpp::exact::optimal_bins;
use bpp::generators::{generate, Distribution};

const ALGS: [&str; 5] = ["NF", "FF", "BF", "FFD", "BFD"];
const DISTS: [Distribution; 4] = [
    Distribution::UniformContinuous,
    Distribution::UniformDiscrete100,
    Distribution::ThreePartition,
    Distribution::FalkenauerU120,
];

fn bins(alg: &str, items: &[f64]) -> usize {
    match alg {
        "NF" => next_fit(items).bins,
        "FF" => first_fit(items).bins,
        "BF" => best_fit(items).bins,
        "FFD" => first_fit_decreasing(items).bins,
        _ => best_fit_decreasing(items).bins,
    }
}

fn main() {
    println!("Superestimacao de A(L2) sobre A(OPT) — em que faixa ela esta?\n");
    println!(
        "{:<22}{:>10}{:>10}{:>10}{:>12}",
        "distribuicao", "minimo", "media", "maximo", "amostras"
    );
    println!("{}", "-".repeat(66));

    let mut global_min = f64::MAX;
    let mut global_max = 0.0f64;
    let mut todas: Vec<f64> = Vec::new();

    for &d in DISTS.iter() {
        let mut vals: Vec<f64> = Vec::new();
        for n in [6usize, 8, 10, 12, 14] {
            for seed in 0..300u64 {
                let items = generate(n, d, seed);
                let opt = match optimal_bins(&items) {
                    Some(o) if o > 0 => o,
                    _ => continue,
                };
                let l2 = lower_bound_l2(&items);
                if l2 == 0 {
                    continue;
                }
                for alg in ALGS {
                    let a = bins(alg, &items) as f64;
                    if a == 0.0 {
                        continue;
                    }
                    let r_l2 = a / l2 as f64;
                    let r_opt = a / opt as f64;
                    vals.push((r_l2 / r_opt - 1.0) * 100.0);
                }
            }
        }
        if vals.is_empty() {
            continue;
        }
        let mn = vals.iter().cloned().fold(f64::MAX, f64::min);
        let mx = vals.iter().cloned().fold(0.0f64, f64::max);
        let md = vals.iter().sum::<f64>() / vals.len() as f64;
        global_min = global_min.min(mn);
        global_max = global_max.max(mx);
        todas.extend(vals.iter().copied());
        println!(
            "{:<22}{:>9.2}%{:>9.2}%{:>9.2}%{:>12}",
            d.name(),
            mn,
            md,
            mx,
            vals.len()
        );
    }

    if todas.is_empty() {
        println!("nenhuma amostra valida");
        return;
    }

    let media = todas.iter().sum::<f64>() / todas.len() as f64;
    println!("{}", "-".repeat(66));
    println!(
        "{:<22}{:>9.2}%{:>9.2}%{:>9.2}%{:>12}",
        "TODAS",
        global_min,
        media,
        global_max,
        todas.len()
    );

    println!("");
    println!("O ARTIGO AFIRMA: \"entre 0,15% e 4,7%\"");
    println!(
        "MEDIDO ({} amostras): {:.2}% a {:.2}%",
        todas.len(),
        global_min,
        global_max
    );
    if global_max > 4.7 {
        println!(">>> O ARTIGO DEVE SER CORRIGIDO: o teto declarado (4,7%) fica abaixo do maximo medido.");
    } else {
        println!(">>> O intervalo declarado no artigo cobre o medido.");
    }
    println!("");
    // Detalhe por tamanho: o maximo vem de n muito pequeno?
    println!("\nDetalhe por tamanho (3-particao, onde o maximo aparece):");
    for n in [6usize, 8, 10, 12, 14] {
        let mut v: Vec<f64> = Vec::new();
        for seed in 0..300u64 {
            let it = generate(n, Distribution::ThreePartition, seed);
            let opt = match optimal_bins(&it) { Some(o) if o > 0 => o, _ => continue };
            let l2 = lower_bound_l2(&it);
            if l2 == 0 { continue; }
            for alg in ALGS {
                let a = bins(alg, &it) as f64;
                if a == 0.0 { continue; }
                v.push(((a / l2 as f64) / (a / opt as f64) - 1.0) * 100.0);
            }
        }
        if v.is_empty() { continue; }
        let mn = v.iter().cloned().fold(f64::MAX, f64::min);
        let mx = v.iter().cloned().fold(0.0f64, f64::max);
        let md = v.iter().sum::<f64>() / v.len() as f64;
        println!("  n = {:2}: media {:>6.2}%   min {:>6.2}%   max {:>6.2}%", n, md, mn, mx);
    }
    println!("");
    println!("NOTA: faixa medida em n = 6..14 (limite do branch-and-bound).");
    println!("Em n grande a superestimacao tende a 0, porque L2 se aproxima de OPT.");
}
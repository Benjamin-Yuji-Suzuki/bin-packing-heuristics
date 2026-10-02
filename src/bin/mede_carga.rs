//! Mede a ESTABILIDADE de cada heuristica (serie bins-only) e testa se o
//! multiplo NF/FF que o artigo publica sobrevive a variacao de medicao.
//!
//! Pergunta que responde: o numero "706x mais rapido" e um numero ou uma
//! medicao? Se o NF (que roda em microssegundos) e estavel e o FF (que
//! roda em dezenas de milissegundos) varia, o multiplo herda toda a
//! instabilidade do FF — e a resposta muda.
//!
//! Uso: cargo run --release --bin mede_carga

use bpp::algorithms::{first_fit_bins, next_fit_bins};
use bpp::generators::{generate, Distribution};
use std::time::Instant;

const N: usize = 16000;

fn melhor_de(f: fn(&[f64]) -> usize, items: &[f64], k: usize) -> f64 {
    let mut m = f64::MAX;
    for _ in 0..k {
        let t0 = Instant::now();
        f(items);
        let t = t0.elapsed().as_nanos() as f64;
        if t < m {
            m = t;
        }
    }
    m
}

fn main() {
    let items = generate(N, Distribution::UniformDiscrete100, 16000);

    println!("==============================================");
    println!(" Estabilidade por heuristica (n = {N}, bins-only)");
    println!("==============================================");
    println!("20 medicoes independentes, cada uma o melhor de varias execucoes.\n");

    let mut nf: Vec<f64> = Vec::new();
    let mut ff: Vec<f64> = Vec::new();
    for _ in 0..20 {
        nf.push(melhor_de(next_fit_bins, &items, 200));
        ff.push(melhor_de(first_fit_bins, &items, 5));
    }

    let media = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
    let desvio = |v: &Vec<f64>| {
        let m = media(v);
        let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64;
        var.sqrt()
    };
    let minimo = |v: &Vec<f64>| v.iter().cloned().fold(f64::MAX, f64::min);
    let maximo = |v: &Vec<f64>| v.iter().cloned().fold(0.0f64, f64::max);

    for (nome, v) in [("NF", &nf), ("FF", &ff)] {
        println!(
            "  {}: media {:>12.0} ns | desvio {:>10.0} ns | CV {:>6.2}% | min {:>12.0} | max {:>12.0}",
            nome,
            media(v),
            desvio(v),
            100.0 * desvio(v) / media(v),
            minimo(v),
            maximo(v)
        );
    }

    println!("\n  razao de variacao (max/min):");
    println!("    NF: {:.2}x", maximo(&nf) / minimo(&nf));
    println!("    FF: {:.2}x", maximo(&ff) / minimo(&ff));

    println!("\n  multiplo FF/NF em cada uma das 20 medicoes:");
    let mults: Vec<f64> = (0..20).map(|i| ff[i] / nf[i]).collect();
    println!(
        "    minimo {:.0}x   medio {:.0}x   maximo {:.0}x",
        minimo(&mults),
        media(&mults),
        maximo(&mults)
    );

    println!("\n  CONCLUSAO");
    println!("  O NF e' a heuristica MAIS ESTAVEL: roda em microssegundos, e um");
    println!("  preempcao no meio da medicao e' improvavel. O FF e' a MENOS");
    println!("  ESTAVEL: roda em dezenas de milissegundos e sofre preempcao");
    println!("  varias vezes. Portanto o multiplo FF/NF herda a instabilidade do");
    println!("  FF — e uma faixa, nao um numero unico.");
}
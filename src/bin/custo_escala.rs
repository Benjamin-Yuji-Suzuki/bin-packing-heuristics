//! Mede o custo REAL de ampliar a escala, sem extrapolar.
//!
//! A extrapolacao quadratica suggestia que n=256.000 levaria ~6,4 s
//! por execucao do First Fit em Rust — e o Python, que ja e' ~1000x
//! mais lento, seria inviável. Este binario mede de fato, em uma
//! instancia por tamanho, e reporta o custo por heuristica.
//!
//! Uso: cargo run --release --bin custo_escala [tamanhos...]
use bpp::algorithms::*;
use bpp::generators::{generate, Distribution};
use std::time::Instant;

fn main() {
    let tamanhos: Vec<usize> = std::env::args()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    let tamanhos = if tamanhos.is_empty() {
        vec![16000usize, 64000, 128000, 256000]
    } else {
        tamanhos
    };

    println!("Custo real por tamanho (1 instancia, serie bins-only)\n");
    println!(
        "{:<10}{:>12}{:>12}{:>12}{:>12}{:>12}",
        "n", "NF", "FF", "BF", "FFD", "BFD"
    );
    println!("{}", "-".repeat(70));

    for &n in &tamanhos {
        let items = generate(n, Distribution::UniformDiscrete100, 1000 + n as u64);
        let mut linha = format!("{:<10}", n);
        let mut total = 0.0f64;
        for (nome, f) in [
            ("NF", next_fit_bins as fn(&[f64]) -> usize),
            ("FF", first_fit_bins),
            ("BF", best_fit_bins),
            ("FFD", first_fit_decreasing_bins),
            ("BFD", best_fit_decreasing_bins),
        ] {
            // uma execucao de aquecimento + uma medida
            f(&items);
            let t0 = Instant::now();
            f(&items);
            let us = t0.elapsed().as_micros() as f64;
            total += us;
            let campo = if us >= 1000.0 {
                format!("{:.1}ms", us / 1000.0)
            } else {
                format!("{:.0}us", us)
            };
            linha.push_str(&format!("{:>12}", campo));
        }
        println!("{linha}   | total {:.1}ms", total / 1000.0);
    }

    println!();
    println!("Custo de um experimento completo (4 dist x 3 tam x 3 reps x 4 linguagens):");
    let ff16 = 24997.0; // medido a n=16000, em microssegundos
    for &n in &tamanhos {
        let mult = (n as f64 / 16000.0).powi(2);
        let ff = ff16 * mult; // microssegundos, expoente 2
        println!(
            "  n = {:>7}: FF Rust {:>7.2}ms | Python (x1000) {:>7.1}s | 3 rodadas so Rust {:>6.1}s",
            n,
            ff / 1000.0,
            ff * 1000.0 / 1e6,
            ff / 1e6 * 3.0
        );
    }
    println!();
    println!("Para medir ate n=256.000 com Python, precisaria de horas.");
    println!("Alternativa: usar o doubling experiments (n = 1k, 2k, 4k, 8k, 16k,");
    println!("32k, 64k, 128k) que e' o que a literatura de ajusto de curva usa.");
}
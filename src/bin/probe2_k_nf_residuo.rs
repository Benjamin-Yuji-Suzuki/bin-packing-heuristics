//! PROBE K — algorithms.rs:31 `next_fit`: o vetor `residual` guarda o
//! residuo do bin FECHADO, mas o residuo final e' empurrado duas vezes
//! quando o ultimo item fecha o bin. Verifica se `residual.len()` ainda
//! igual a `bins`, e se algum residuo de next_fit bate exatamente 1.0 (o que
//! denotaria um bin vazio accounted).
//!
//! Uso: cargo run --release --bin probe2_k_nf_residuo

use bpp::algorithms::{next_fit, next_fit_bins};

fn main() {
    println!("=== PROBE K: next_fit — contabilidade do vetor residual ===\n");

    // Varre padroes onde bins/close alternam de forma extrema.
    let mut piores: Vec<(usize, usize, usize, f64)> = Vec::new();
    for n in 1..=12usize {
        // todos os itens > 0.5: NF fecha um bin por item
        let grande: Vec<f64> = (0..n).map(|_| 0.6).collect();
        let s = next_fit(&grande);
        if s.residual.len() != s.bins {
            piores.push((n, s.bins, s.residual.len(), 0.6));
        }
    }
    for n in 1..=12usize {
        let meio: Vec<f64> = (0..n).map(|_| 0.5).collect();
        let s = next_fit(&meio);
        if s.residual.len() != s.bins {
            piores.push((n, s.bins, s.residual.len(), 0.5));
        }
    }
    if piores.is_empty() {
        println!("  residual.len() == bins em todos os padroes testados (n<=12).");
    } else {
        for (n, b, r, x) in &piores {
            println!("  DIVERGE n={n} x={x}: bins={b} residual.len()={r}");
        }
    }

    // Um residuo exatamente 1.0 indicaria um bin accounted sem item.
    println!("\n=== PROBE K2: algum bin de next_fit fica VAZIO (residuo == 1.0)? ===\n");
    for n in [1usize, 2, 3, 5, 10] {
        let grande: Vec<f64> = (0..n).map(|_| 0.6).collect();
        let s = next_fit(&grande);
        let vazios = s.residual.iter().filter(|&&r| r > 1.0 - 1e-12).count();
        println!(
            "  n={n:3} (itens 0.6): bins={} residual={:?} bins vazios={vazios}",
            s.bins, s.residual
        );
    }

    // Item que exatamente fecha o bin no ultimo passo.
    println!("\n=== PROBE K3: ultimo item fecha o bin exatamente ===\n");
    for itens in [
        vec![0.5, 0.5],
        vec![0.7, 0.3],
        vec![0.6, 0.4],
        vec![0.25, 0.25, 0.25, 0.25],
    ] {
        let s = next_fit(&itens);
        let sb = next_fit_bins(&itens);
        println!(
            "  {itens:?} -> bins={} residual={:?} | bins-only={sb} | bate={}",
            s.bins,
            s.residual,
            s.bins == sb
        );
    }
}
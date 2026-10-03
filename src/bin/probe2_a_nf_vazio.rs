//! PROBE A — algorithms.rs:152 `next_fit_bins(&[])` vs `next_fit(&[])`.
//!
//! A auditoria anterior corrigiu `next_fit([])` (agora 0 bins) mas a
//! variante bins-only continua comecando em `bins = 1`. O contrato
//! declarado em algorithms.rs:136-141 ("mesmo algoritmo, mesma saida
//! (contagem)") e a assercao de experiment.rs:103 (debug_assert)
//! dizem que as duas 系列 DEVEM concordar. O teste
//! `variantes_bins_only_iguais_as_completas` so cobre n in {10,50,200}.
//!
//! Uso: cargo run --release --bin probe2_a_nf_vazio

use bpp::algorithms::*;
use bpp::experiment::run_algorithm_bins;

fn main() {
    println!("=== PROBE A: next_fit / next_fit_bins com lista VAZIA ===\n");
    let vazio: Vec<f64> = Vec::new();

    let completo = next_fit(&vazio).bins;
    let so_bins = next_fit_bins(&vazio);
    let via_run = run_algorithm_bins("NF", &vazio);
    println!("next_fit(&[]).bins        = {completo}");
    println!("next_fit_bins(&[])        = {so_bins}");
    println!("run_algorithm_bins(NF,[]) = {via_run}");

    if completo != so_bins {
        println!(
            "\n>>> DIVERGENCIA CONFIRMADA: variante completa = {completo}, \
             variante bins-only = {so_bins} (diferenca = {})",
            so_bins as i64 - completo as i64
        );
    } else {
        println!("\n>>> sem divergencia");
    }

    println!("\n--- todas as 5 heuristicas, completa vs bins-only, n = 0 ---");
    let pares: [(&str, usize, usize); 5] = [
        ("NF", next_fit(&vazio).bins, next_fit_bins(&vazio)),
        ("FF", first_fit(&vazio).bins, first_fit_bins(&vazio)),
        ("BF", best_fit(&vazio).bins, best_fit_bins(&vazio)),
        (
            "FFD",
            first_fit_decreasing(&vazio).bins,
            first_fit_decreasing_bins(&vazio),
        ),
        (
            "BFD",
            best_fit_decreasing(&vazio).bins,
            best_fit_decreasing_bins(&vazio),
        ),
    ];
    let mut ruins = 0;
    for (nome, comp, sb) in pares {
        let marca = if comp == sb { "ok  " } else { "DIVERGE" };
        if comp != sb {
            ruins += 1;
        }
        println!("  {marca} {nome:4} completa={comp}  bins-only={sb}");
    }
    println!("\nheuristicas divergentes em n=0: {ruins}/5");

    // O mesmo divergences para n=1 (o caso minimo nao-vazio), que o
    // Sweep do artigo nunca alcanca mas que qualquer chamador pode passar.
    println!("\n--- controle: n = 1 (item unico 0.5) ---");
    let um = vec![0.5f64];
    println!(
        "  NF  completa={}  bins-only={}",
        next_fit(&um).bins,
        next_fit_bins(&um)
    );
}

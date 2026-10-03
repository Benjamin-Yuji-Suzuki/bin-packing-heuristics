//! PROBE G — varredura de invariantes em TODO o crate.
//!
//! Nenhum teste do crate checa:
//!   I1  bins == residual.len()          (declarado em algorithms.rs:47)
//!   I2  assignment.len() == items.len()
//!   I3  assignment[i] < bins
//!   I4  nenhum residual < -1e-9         (capacidade violada)
//!   I5  nenhum residual > 1 + 1e-9       (item cabendo "sozinho" sobra > 1)
//!   I6  completo == bins-only, em TODAS as 5 heuristicas
//!   I7  L2 <= todas as heuristicas
//!
//! Varre o gerador principal, as familias adversariais e a construcao de
//! Johnson (incluindo a faixa de eps que o assert aceita).
//!
//! Uso: cargo run --release --bin probe2_g_invariantes

use bpp::adversarial::{all_adversarial, generate_adversarial};
use bpp::algorithms::*;
use bpp::generators::{generate, Distribution};
use bpp::johnson1974::{pior_caso_com_reposicao, pior_caso_ff};

const EPS: f64 = 1e-9;

#[derive(Default)]
struct Contadores {
    n_inst: usize,
    i1_bins_eq_resid: usize,
    i2_len: usize,
    i3_idx: usize,
    i4_neg: usize,
    i5_sobra: usize,
    i6_diverg: usize,
    i7_l2: usize,
}

fn checa(items: &[f64], c: &mut Contadores) {
    c.n_inst += 1;
    let algs: [(&str, fn(&[f64]) -> Solution); 5] = [
        ("NF", next_fit),
        ("FF", first_fit),
        ("BF", best_fit),
        ("FFD", first_fit_decreasing),
        ("BFD", best_fit_decreasing),
    ];
    let bins_only: [(&str, fn(&[f64]) -> usize); 5] = [
        ("NF", next_fit_bins),
        ("FF", first_fit_bins),
        ("BF", best_fit_bins),
        ("FFD", first_fit_decreasing_bins),
        ("BFD", best_fit_decreasing_bins),
    ];

    for (i, (nome, f)) in algs.iter().enumerate() {
        let s = f(items);
        if s.bins != s.residual.len() {
            c.i1_bins_eq_resid += 1;
            if c.i1_bins_eq_resid <= 3 {
                println!(
                    "  I1 {nome}: bins={} != residual.len()={}  (itens={})",
                    s.bins,
                    s.residual.len(),
                    items.len()
                );
            }
        }
        if s.assignment.len() != items.len() {
            c.i2_len += 1;
            println!("  I2 {nome}: assignment={} itens={}", s.assignment.len(), items.len());
        }
        if s.assignment.iter().any(|&b| b >= s.bins) {
            c.i3_idx += 1;
            println!("  I3 {nome}: indice de bin >= bins ({})", s.bins);
        }
        if s.residual.iter().any(|&r| r < -EPS) {
            c.i4_neg += 1;
            if c.i4_neg <= 3 {
                let p = s.residual.iter().cloned().fold(f64::INFINITY, f64::min);
                println!("  I4 {nome}: residuo {p:.9} < 0 (capacidade violada)");
            }
        }
        if s.residual.iter().any(|&r| r > 1.0 + EPS) {
            c.i5_sobra += 1;
            if c.i5_sobra <= 3 {
                let p = s.residual.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                println!("  I5 {nome}: residuo {p:.9} > 1");
            }
        }
        let sb = bins_only[i].1(items);
        if s.bins != sb {
            c.i6_diverg += 1;
            println!("  I6 {}: completa={} bins-only={}", nome, s.bins, sb);
        }
    }
    let l2 = lower_bound_l2(items);
    if algs.iter().any(|(_, f)| f(items).bins < l2) {
        c.i7_l2 += 1;
        println!("  I7 L2={l2} acima de uma heuristica");
    }
}

fn main() {
    let mut c = Contadores::default();
    println!("=== PROBE G: varredura de invariantes ===\n");

    for &dist in Distribution::all().iter() {
        for n in [1usize, 2, 3, 5, 10, 50, 100, 500, 1000] {
            for seed in 0..8u64 {
                let items = generate(n, dist, 1000 * (seed + 1) + n as u64);
                checa(&items, &mut c);
            }
        }
    }
    println!("[1] gerador principal: {} instancias", c.n_inst);

    let antes = c.n_inst;
    for kind in all_adversarial() {
        for n in [0usize, 1, 2, 3, 5, 10, 37, 100, 101, 200] {
            let items = generate_adversarial(kind, n);
            checa(&items, &mut c);
        }
    }
    println!("[2] familias adversariais: {} instancias", c.n_inst - antes);

    // Johnson com eps VALIDO (dentro do que o artigo exige: < 1/50)
    let antes = c.n_inst;
    for &n in &[17usize, 34, 51] {
        for k in 1..20usize {
            let eps = k as f64 / 1000.0;
            if eps >= 1.0 / 50.0 {
                break;
            }
            let (l, _) = pior_caso_ff(n, eps);
            checa(&l, &mut c);
            let com_rep = pior_caso_com_reposicao(n, eps, 7);
            checa(&com_rep, &mut c);
        }
    }
    println!("[3] Johnson com eps valido (< 1/50): {} instancias", c.n_inst - antes);

    // Johnson com eps INVALIDO (o que o assert aceita) — aqui tudo quebra
    let antes = c.n_inst;
    for &eps in &[0.133f64, 0.15, 0.2, 0.25, 0.3, 0.4] {
        let (l, _) = pior_caso_ff(17, eps);
        checa(&l, &mut c);
    }
    println!("[4] Johnson com eps que o ASSERT aceita: {} instancias\n", c.n_inst - antes);

    println!("--- RESUMO (as colunas so contam nas 3 primeiras faixas) ---");
    println!("  instancias varridas                       : {}", c.n_inst);
    println!("  I1 bins != residual.len()                 : {}", c.i1_bins_eq_resid);
    println!("  I2 assignment.len() != items.len()        : {}", c.i2_len);
    println!("  I3 assignment[i] >= bins                  : {}", c.i3_idx);
    println!("  I4 residual < 0 (capacidade violada)      : {}", c.i4_neg);
    println!("  I5 residual > 1                           : {}", c.i5_sobra);
    println!("  I6 completa != bins-only                  : {}", c.i6_diverg);
    println!("  I7 L2 acima de heuristica                 : {}", c.i7_l2);
    println!("\n(I4/I5/I6 da faixa [4] vem do eps invalido do PROBE C, nao de bug novo.)");
}

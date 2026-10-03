//! PROBE D — algorithms.rs:259 `candidate_alphas` usa um conjunto de
//! cortes INCOMPLETO para o L2 de Martello & Toth.
//!
//! L2(alpha) = |{s > 1-alpha}| + ceil( sum{ alpha <= s <= 1-alpha } )
//! tem descontinuidades em alpha = s_i E em alpha = 1 - s_i:
//!   - alpha = s_i    : o item entra/sai da janela "media" pela borda esquerda
//!   - alpha = 1 - s_i: o item entra/sai da janela pela borda direita
//!                      (e entra/sai do conjunto "grande")
//! `candidate_alphas` so oferece {0.5} U {s_i : s_i < 0.5}. Faltam os
//! pontos 1 - s_i dos itens GRANDES (s_i > 0.5), que caem em (0, 0.5).
//!
//! Se faltar um ponto de descontinuidade, o maximo verdadeiro de L2 nao
//! e' amostrado e o bound sai FRACO (nunca acima do optimo — so menos
//! apertado). Este probe compara o L2 do crate com o L2 de referencia
//! avaliado sobre o conjunto COMPLETO de breakpoints.
//!
//! Uso: cargo run --release --bin probe2_d_l2_alphas

use bpp::algorithms::lower_bound_l2;
use bpp::generators::{generate, Distribution};

/// L2 de referencia: mesma formula e mesmas tolerancias do crate, porem
/// avaliada sobre TODOS os breakpoints (s_i e 1-s_i, mais 0.5).
fn l2_referencia(items: &[f64]) -> usize {
    if items.is_empty() {
        return 0;
    }
    let mut cands: Vec<f64> = vec![0.5];
    for &s in items {
        if s > 1e-9 && s < 0.5 - 1e-9 {
            cands.push(s);
        }
        // breakpoint que o crate NAO considera:
        let b = 1.0 - s;
        if b > 1e-9 && b < 0.5 - 1e-9 {
            cands.push(b);
        }
    }
    cands.sort_by(|a, b| a.total_cmp(b));
    cands.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let mut best = (items.iter().sum::<f64>() - 1e-9).ceil().max(0.0) as usize;
    for &alpha in &cands {
        let large = items.iter().filter(|&&s| s > 1.0 - alpha + 1e-9).count();
        let medium_sum: f64 = items
            .iter()
            .filter(|&&s| s >= alpha - 1e-9 && s <= 1.0 - alpha + 1e-9)
            .sum();
        let medium = (medium_sum - 1e-9).ceil().max(0.0) as usize;
        if large + medium > best {
            best = large + medium;
        }
    }
    best
}

fn main() {
    println!("=== PROBE D: L2 do crate vs L2 com o conjunto COMPLETO de breakpoints ===\n");

    let mut checados = 0usize;
    let mut mais_fraco = 0usize;
    let mut totais: (usize, usize) = (0, 0);
    let mut exemplos: Vec<(usize, String, usize, usize)> = Vec::new();
    let mut maior_dif = 0usize;

    for &dist in Distribution::all().iter() {
        for n in [20usize, 50, 100, 200, 500, 1000] {
            for seed in 0..60u64 {
                let items = generate(n, dist, seed);
                let do_crate = lower_bound_l2(&items);
                let referencia = l2_referencia(&items);
                checados += 1;
                totais.0 += do_crate;
                totais.1 += referencia;
                if referencia > do_crate {
                    mais_fraco += 1;
                    let dif = referencia - do_crate;
                    if dif > maior_dif {
                        maior_dif = dif;
                    }
                    if exemplos.len() < 6 {
                        exemplos.push((n, format!("{:?}", dist), do_crate, referencia));
                    }
                }
            }
        }
    }

    println!("instancias avaliadas                     : {checados}");
    println!("casos onde o L2 do crate e' MENOR que o L2 completo (bound mais fraco): {mais_fraco}");
    println!(
        "soma dos L2 (crate) = {}   soma dos L2 (completo) = {}   perda total = {}",
        totais.0,
        totais.1,
        totais.1 as i64 - totais.0 as i64
    );
    println!(
        "perda media por instancia: {:.4} bins",
        (totals_diff(&totais, checados))
    );
    println!("maior deficit individual: {maior_dif} bins");
    for (n, dist, a, b) in &exemplos {
        println!("  n={n:5} {dist:<22} crate={a:5}  completo={b:5}  (falta {})", b - a);
    }

    // Caso minimo, para inspecao manual.
    if let Some(ex) = menor_contraexemplo() {
        let (items, crate_lb, ref_lb) = ex;
        println!("\n--- contraexemplo minimo ---");
        println!("  itens          = {items:?}");
        println!("  soma           = {:.6}", items.iter().sum::<f64>());
        println!("  L2 do crate    = {crate_lb}");
        println!("  L2 completo    = {ref_lb}");
        let mut cands: Vec<f64> = vec![0.5];
        for &s in &items {
            if s > 1e-9 && s < 0.5 - 1e-9 {
                cands.push(s);
            }
            let b = 1.0 - s;
            if b > 1e-9 && b < 0.5 - 1e-9 {
                cands.push(b);
            }
        }
        println!("  cortes que o crate considera: {cands:?} (so os < 0.5 + 0.5)");
        for &a in &cands {
            let large = items.iter().filter(|&&s| s > 1.0 - a + 1e-9).count();
            let ms: f64 = items
                .iter()
                .filter(|&&s| s >= a - 1e-9 && s <= 1.0 - a + 1e-9)
                .sum();
            let med = (ms - 1e-9).ceil().max(0.0) as usize;
            println!("    alpha={a:.6}: large={large} + ceil(sum={ms:.6})={med} => {}", large + med);
        }
    }
}

fn totals_diff(t: &(usize, usize), n: usize) -> f64 {
    (t.1 as f64 - t.0 as f64) / n as f64
}

fn menor_contraexemplo() -> Option<(Vec<f64>, usize, usize)> {
    for &dist in Distribution::all().iter() {
        for n in [5usize, 6, 7, 8, 10, 12, 20, 40] {
            for seed in 0..4000u64 {
                let items = generate(n, dist, seed);
                let a = lower_bound_l2(&items);
                let b = l2_referencia(&items);
                if b > a {
                    return Some((items, a, b));
                }
            }
        }
    }
    None
}

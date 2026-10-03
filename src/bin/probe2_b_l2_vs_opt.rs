//! PROBE B — algorithms.rs:226 `lower_bound_l2` contra o OTIMO EXATO.
//!
//! O teste existente `l2_nunca_ultrapassa_opt` (algorithms.rs:334) NAO
//! testa o que o nome promete: ele compara `lb <= ffd`, nao `lb <= OPT`.
//! FFD >= OPT sempre, entao `lb <= ffd` passa mesmo com `lb > OPT`.
//! Se L2 passar do optimo, a razao A/L2 do artigo deixa de ser
//! conservadora — que e' o argumento que o codigo justifica ao longo
//! das linhas 230-247.
//!
//! Este probe fecha a lacuna: compara L2 com o B&B exato.
//!
//! Uso: cargo run --release --bin probe2_b_l2_vs_opt

use bpp::algorithms::*;
use bpp::exact::optimal_bins_budget;
use bpp::generators::{generate, Distribution};

const BUDGET: u128 = 20_000_000; // 20 s por instancia

fn main() {
    let mut checados = 0usize;
    let mut acima_de_opt = 0usize;
    let mut acima_de_fff = 0usize;
    let mut piores: Vec<(usize, String, usize, usize, usize)> = Vec::new();

    println!("=== PROBE B: L2 vs OPTIMO EXATO ===");
    println!("(o teste do crate so checa lb <= FFD; aqui checamos lb <= OPT)\n");

    // --- 1. Todas as distribuicoes do gerador principal, n pequeno ---
    for &dist in Distribution::all().iter() {
        for n in [5usize, 6, 7, 8, 9, 10, 11, 12] {
            for seed in 0..400u64 {
                let items = generate(n, dist, seed);
                let Some(opt) = optimal_bins_budget(&items, BUDGET) else {
                    continue;
                };
                checados += 1;
                let lb = lower_bound_l2(&items);
                let ffd = first_fit_decreasing(&items).bins;
                if lb > opt {
                    acima_de_opt += 1;
                    if acima_de_fff == 0 || lb > ffd {
                        acima_de_fff += 1;
                    }
                    if piores.len() < 5 {
                        piores.push((n, format!("{:?}", dist), lb, opt, ffd));
                    }
                }
            }
        }
    }

    // --- 2. Familias adversariais (itens degenerados, onde o ceil bate
    //        em inteiro exato e a tolerancia de 1e-9 decide) ---
    use bpp::adversarial::{all_adversarial, generate_adversarial};
    for fam in all_adversarial() {
        for n in [4usize, 5, 6, 7, 8, 9, 10, 11, 12] {
            let items = generate_adversarial(fam, n);
            let Some(opt) = optimal_bins_budget(&items, BUDGET) else {
                continue;
            };
            checados += 1;
            let lb = lower_bound_l2(&items);
            let ffd = first_fit_decreasing(&items).bins;
            if lb > opt {
                acima_de_opt += 1;
                if piores.len() < 5 {
                    piores.push((n, format!("{fam:?}"), lb, opt, ffd));
                }
            }
        }
    }

    // --- 3. Varredura exaustiva em grade de 1/10 (todas as multisets
    //        de tamanho <= 8) — o pior caso para `ceil` com tolerancia ---
    let grades: Vec<f64> = (1..=10).map(|k| k as f64 / 10.0).collect();
    for mask in 0u32..(1u32 << 14) {
        let mut items: Vec<f64> = Vec::new();
        let mut m = mask;
        for _ in 0..7 {
            items.push(grades[(m % 10) as usize]);
            m /= 10;
        }
        let Some(opt) = optimal_bins_budget(&items, BUDGET) else {
            continue;
        };
        checados += 1;
        let lb = lower_bound_l2(&items);
        if lb > opt {
            acima_de_opt += 1;
            if piores.len() < 8 {
                piores.push((items.len(), format!("{items:?}"), lb, opt, 0));
            }
        }
    }

    // --- 4. Grade de 1/100 com tolernancia: pares que somam 1 + 1e-9 ---
    for k in 1..=100usize {
        for j in 1..=100usize {
            let items = vec![k as f64 / 100.0, j as f64 / 100.0];
            let Some(opt) = optimal_bins_budget(&items, BUDGET) else {
                continue;
            };
            checados += 1;
            let lb = lower_bound_l2(&items);
            if lb > opt {
                acima_de_opt += 1;
                if piores.len() < 10 {
                    piores.push((2, format!("{items:?}"), lb, opt, 0));
                }
            }
        }
    }

    println!("instancias com OTIMO provado : {checados}");
    println!("casos com L2 > OPT           : {acima_de_opt}");
    for (n, inst, lb, opt, ffd) in &piores {
        println!("  n={n:3} {inst:<60} L2={lb} OPT={opt} FFD={ffd}");
    }
    if acima_de_opt == 0 {
        println!("\n>>> L2 nunca passou do otimo exato em {checados} instancias provadas.");
    } else {
        println!("\n>>> L2 PASSOU DO OTIMO em {acima_de_opt}/{checados} instancias provadas.");
    }
}

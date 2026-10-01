//! Construção e busca ESTRUTURADA da pior caso do First Fit.
//!
//! POR QUE GRADE E NÃO SORTEIO: o pior caso do FF (Johnson et al. 1974)
//! não é uma sequência aleatória de tamanhos — é uma construção em
//! camadas: um grupo de itens grandes abre bins cujo resíduo é pequeno
//! demais para o grupo seguinte, forçando o FF a abrir bins novos enquanto
//! o OPT reaproveita esse resíduo. O hill climbing aleatório acha
//! "instâncias ruins" mas erra a estrutura; varrer uma grade sobre as
//! famílias em camadas cobre exatamente onde o pior caso mora.
//!
//! Executa: cargo run --release --bin johnson [n_por_camada]

use bpp::algorithms::*;
use bpp::exact::optimal_bins;
use std::time::Instant;

fn arred(x: f64) -> f64 {
    // quantiza em 1e-4: mantem as faixas criticas (1/2, 1/3, 1/4) e torna
    // a busca exaustiva sobre uma grade finita
    (x * 10_000.0).round() / 10_000.0
}

fn razao(alg: &str, items: &[f64], opt: usize) -> (usize, f64) {
    let bins = match alg {
        "NF" => next_fit(items).bins,
        "FF" => first_fit(items).bins,
        "BF" => best_fit(items).bins,
        "FFD" => first_fit_decreasing(items).bins,
        "BFD" => best_fit_decreasing(items).bins,
        _ => unreachable!(),
    };
    (bins, bins as f64 / opt.max(1) as f64)
}

/// Constrói uma instância em `k` camadas: `m` itens de cada tamanho,
/// na ordem de tamanho DECRESCENTE (é essa ordem que cria o pior caso:
/// os grandes abrem bins, os seguintes não cabem no resíduo).
fn camadas(tamanhos: &[f64], m: usize) -> Vec<f64> {
    let mut v = Vec::with_capacity(tamanhos.len() * m);
    for &t in tamanhos {
        for _ in 0..m {
            v.push(t);
        }
    }
    v
}

fn main() {
    let m: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(4);

    println!("Busca ESTRUTURADA da pior caso — famílias em camadas");
    println!("m = {m} itens por camada\n");

    // ---------- 1. Famílias canônicas de 2 e 3 camadas ----------
    // Cada linha é (nome, tamanhos das camadas em ordem decrescente).
    let mut familias: Vec<(String, Vec<f64>)> = Vec::new();
    for &(a, b) in &[(0.5001, 0.3334), (0.5001, 0.2501), (0.5001, 0.2001),
                     (0.4001, 0.3334), (0.4001, 0.2501), (0.3334, 0.2501),
                     (0.6001, 0.3334), (0.6001, 0.2001), (0.7501, 0.3334)] {
        familias.push((format!("2cam [{a},{b}]"), vec![a, b]));
    }
    for &(a, b, c) in &[(0.5001, 0.3334, 0.2501), (0.5001, 0.2501, 0.1667),
                        (0.6001, 0.3334, 0.2501), (0.4001, 0.3334, 0.2501),
                        (0.5001, 0.3334, 0.2001), (0.7501, 0.5001, 0.3334)] {
        familias.push((format!("3cam [{a},{b},{c}]"), vec![a, b, c]));
    }

    println!("{:<26}{:>5}{:>7}{:>7}{:>7}{:>7}{:>7}", 
             "família", "OPT", "NF", "FF", "BF", "FFD", "BFD");
    println!("{}", "-".repeat(72));
    let mut melhor_global: Option<(f64, String, Vec<f64>, usize, usize)> = None;

    for (nome, tam) in &familias {
        let items = camadas(tam, m);
        let opt = match optimal_bins(&items) {
            Some(o) => o,
            None => continue,
        };
        let mut linha = format!("{:<26}{:>5}", nome, opt);
        for alg in ["NF", "FF", "BF", "FFD", "BFD"] {
            let (bins, r) = razao(alg, &items, opt);
            linha.push_str(&format!("{:>7.3}", r));
            if alg == "FF" {
                if melhor_global.as_ref().map_or(true, |(m0, ..)| r > *m0) {
                    melhor_global = Some((r, nome.clone(), items.clone(), opt, bins));
                }
            }
        }
        println!("{linha}");
    }

    // ---------- 2. Grade fina sobre 2 camadas ----------
    println!("\nGrade fina em 2 camadas (passo 0,005, de 0,10 a 0,95):");
    println!("{}", "-".repeat(72));
    let mut melhor_ff = 0.0f64;
    let mut melhor_inst: Option<(Vec<f64>, usize, usize, String)> = None;
    let mut avaliadas = 0usize;
    let t0 = Instant::now();
    for ia in 20..190 {
        let a = arred(ia as f64 * 0.005);
        for ib in 20..190 {
            let b = arred(ib as f64 * 0.005);
            if b >= a {
                continue; // exige ordem decrescente
            }
            let items = camadas(&[a, b], m);
            let opt = match optimal_bins(&items) {
                Some(o) => o,
                None => continue,
            };
            avaliadas += 1;
            let (bins, r) = razao("FF", &items, opt);
            if r > melhor_ff {
                melhor_ff = r;
                melhor_inst = Some((items, opt, bins, format!("[{a}, {b}]")));
            }
        }
    }
    println!("  {avaliadas} pares avaliados em {:.1} s", t0.elapsed().as_secs_f64());
    if let Some((inst, opt, bins, desc)) = melhor_inst.clone() {
        println!("  melhor razao FF = {melhor_ff:.4}  (garantia 1,7; otimo {opt}; FF abre {bins})");
        println!("  instancia: {desc} x{m}");
        println!(
            "  itens: {:?}",
            inst.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>()
        );
    }

    // ---------- 3. Grade em 3 camadas (só o melhor par 2-cam como base) ----------
    if let Some((_, _, _, desc2)) = melhor_inst.clone() {
        println!("\nGrade em 3 camadas, completando o melhor par de 2 camadas:");
        println!("{}", "-".repeat(72));
        // extrai os dois tamanhos do melhor par
        let limpos: Vec<f64> = desc2
            .trim_matches(['[', ']'])
            .split(", ")
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();
        if limpos.len() == 2 {
            let (a, b) = (limpos[0], limpos[1]);
            let mut melhor3 = 0.0f64;
            let mut inst3: Option<(Vec<f64>, usize, usize)> = None;
            for ic in 5..(a * 200.0) as usize {
                let c = arred(ic as f64 * 0.005);
                if c >= b {
                    continue;
                }
                let items = camadas(&[a, b, c], m);
                let opt = match optimal_bins(&items) {
                    Some(o) => o,
                    None => continue,
                };
                let (bins, r) = razao("FF", &items, opt);
                if r > melhor3 {
                    melhor3 = r;
                    inst3 = Some((items, opt, bins));
                }
            }
            if let Some((inst, opt, bins)) = inst3 {
                println!("  melhor razao FF = {melhor3:.4}  (otimo {opt}; FF abre {bins})");
                println!("  instancia: [{a}, {b}, camada variable] x{m}");
                println!(
                    "  itens: {:?}",
                    inst.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>()
                );
            }
        }
    }

    // ---------- 4. Veredito ----------
    println!("\n{}", "=".repeat(72));
    println!("VEREDITO");
    println!("{}", "=".repeat(72));
    let g = 1.7;
    println!("  garantia do First Fit ....... {g:.3}");
    match melhor_inst {
        Some((_, _, _, desc)) => {
            println!("  melhor razao encontrada ... {melhor_ff:.4}  em {desc} x{m}");
            println!("  distancia para a garantia . {:.4}", g - melhor_ff);
            if melhor_ff >= 1.55 {
                println!("  => Aproximacao boa: a construcao em camadas reproduz");
                println!("     o mecanismo do pior caso.");
            } else {
                println!("  => A construcao em camadas NAO alcanca a garantia.");
                println!("     Reportar honestamente: a busca estruturada tambem");
                println!("     falha em atingir 1,7; a construcao canonica de");
                println!("     Johnson et al. nao foi reproduzida.");
            }
        }
        None => println!("  nenhuma instancia avaliavel"),
    }
}
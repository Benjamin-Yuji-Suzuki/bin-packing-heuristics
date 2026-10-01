//! Busca pela pior caso de cada heurística (hill climbing sobre o
//! multiconjunto de tamanhos), medindo contra o ÓPTIMO EXATO.
//!
//! POR QUE BUSCAR E NÃO CONSTRUIR À MÃO: a construção canônica de pior
//! caso do First Fit é específica (Johnson et al. 1974) e depende de itens
//! em faixas exatas de tamanho. Em vez de citá-la de memória, buscamos
//! empiricamente a razão A(I)/OPT(I) máxima que cada heurística alcança,
//! e comparamos com a garantia teórica. O que a busca encontra é o que o
//! experimento sustenta; o que ela não alcança fica declarado como
//! resultado negativo.
//!
//! Uso: cargo run --release --bin piorcasa <n> <tentativas> [algoritmo]

use bpp::algorithms::*;
use bpp::exact::optimal_bins;
use std::time::Instant;

fn razao(alg: &str, items: &[f64], opt: usize) -> f64 {
    let bins = match alg {
        "NF" => next_fit(items).bins,
        "FF" => first_fit(items).bins,
        "BF" => best_fit(items).bins,
        "FFD" => first_fit_decreasing(items).bins,
        "BFD" => best_fit_decreasing(items).bins,
        _ => unreachable!(),
    };
    bins as f64 / opt.max(1) as f64
}

fn mutacao(items: &mut Vec<f64>, rng: &mut u64) {
    // MUTACAO: desloca um item aleatorio numa escala fina.
    let i = (proximo(rng) as usize) % items.len();
    let direcao = if proximo(rng) % 2 == 0 { 1.0 } else { -1.0 };
    
    let passo = if proximo(rng) % 2 == 0 { 1e-3 } else { 1e-4 };
    let novo = (items[i] + direcao * passo).clamp(0.0001, 1.0);
    items[i] = arredondar(novo);
}

fn arredondar(x: f64) -> f64 {
    // arredonda para 4 casas: mantem o espaco de busca discreto e estavel
    (x * 10_000.0).round() / 10_000.0
}

fn proximo(rng: &mut u64) -> u64 {
    // xorshift64 — determinístico e rápido
    let mut x = *rng;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *rng = x;
    x
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(12);
    let tentativas: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(400);
    let alvo: Vec<String> = if args.len() > 3 {
        args[3].split(',').map(|s| s.to_string()).collect()
    } else {
        ["NF", "FF", "BF", "FFD", "BFD"].iter().map(|s| s.to_string()).collect()
    };

    // Garantias de pior caso (Johnson et al. 1974; Dósa 2007).
    // A razão ASSINTÓTICA é o fator multiplicativo; o FFD tem ainda o
    // termo ADITIVO +6/9, que vira 6/(9·OPT) e desaparece quando OPT
    // cresce. Em instância pequena o termo pesa, então o FFD só pode ser
    // comparado com o OPT real — nunca só com 11/9.
    let razao_garantia = |alg: &str, opt: usize| -> f64 {
        match alg {
            "NF" => 2.0,
            "FF" | "BF" => 1.7,
            "FFD" | "BFD" => 11.0 / 9.0 + (6.0 / 9.0) / opt.max(1) as f64,
            _ => 0.0,
        }
    };

    println!("Busca de pior caso por hill climbing — n = {n}, {tentativas} tentativas");
    println!("otimo exato por branch-and-bound; razao reportada = A(I)/OPT(I)\n");
    println!(
        "{:<5}{:>10}{:>12}{:>12}{:>14}{:>12}",
        "alg", "OPT", "A(I) max", "razão", "garantia", "distância"
    );
    println!("{}", "-".repeat(72));

    let mut melhor_por_alg: std::collections::HashMap<String, (f64, Vec<f64>, usize)> =
        std::collections::HashMap::new();

    for alg in &alvo {
        let mut rng = 0x9E3779B97F4A7C15u64 ^ (n as u64);
        let mut melhor_razao = 0.0f64;
        let mut melhor_itens: Vec<f64> = Vec::new();
        let mut melhor_opt = 0usize;
        let mut melhor_bins = 0usize;

        let mut atual: Vec<f64> = (0..n)
            .map(|_| {
                // começa perto de 0.5, onde as garantias fazem mais sentido
                arredondar(0.4 + (proximo(&mut rng) % 200) as f64 / 1000.0)
            })
            .collect();

        for _ in 0..tentativas {
            let opt = match optimal_bins(&atual) {
                Some(o) => o,
                None => break,
            };
            let r = razao(alg, &atual, opt);
            if r > melhor_razao {
                melhor_razao = r;
                melhor_itens = atual.clone();
                melhor_opt = opt;
                melhor_bins = (r * opt as f64).round() as usize;
            }
            // ESCAPAR DE RUIM LOCAL: reinicia a instancia de vez em quando
            if proximo(&mut rng) % 7 == 0 {
                atual = (0..n)
                    .map(|_| arredondar(0.15 + (proximo(&mut rng) % 700) as f64 / 1000.0))
                    .collect();
            } else {
                mutacao(&mut atual, &mut rng);
            }
        }

        let g = razao_garantia(alg, melhor_opt);
        let dist = if g > 0.0 { g - melhor_razao } else { 0.0 };
        println!(
            "{:<5}{:>10}{:>12}{:>12.4}{:>14.4}{:>12.4}",
            alg, melhor_opt, melhor_bins, melhor_razao, g, dist
        );
        println!("      instância: {:?}", melhor_itens.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>());
        melhor_por_alg.insert(alg.clone(), (melhor_razao, melhor_itens, melhor_opt));
    }

    println!("\n{}", "=".repeat(72));
    println!("VELOCIDADE por algoritmo.");
    println!("Em n = {n} o tempo e' sub-micronsegundo e o relogio reporta zero.");
    println!("Para medir de verdade, repetimos o MESMO padrao de pior caso ate");
    println!("n grande — e' o mesmo fenomeno, porem com tempo mensuravel.");
    println!("{}", "=".repeat(72));

    let ref_itens = melhor_por_alg
        .values()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, v, _)| v.clone())
        .unwrap_or_default();
    if ref_itens.is_empty() {
        return;
    }
    println!("padrao de pior caso usado ({n} itens):");
    println!("  {:?}\n", ref_itens.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>());

    for n_grande in [1000usize, 16000] {
        let inst: Vec<f64> = (0..n_grande)
            .map(|i| ref_itens[i % ref_itens.len()])
            .collect();
        println!("--- n = {n_grande} ---");
        println!("{:<6}{:>10}{:>16}{:>14}", "alg", "bins", "tempo (µs)", "vs NF");
        let mut t_nf = 0.0f64;
        for alg in &["NF", "FF", "BF", "FFD", "BFD"] {
            let mut melhor_t = f64::MAX;
            let mut bins = 0usize;
            let reps = if n_grande >= 16000 { 5 } else { 20 };
            for _ in 0..reps {
                let t0 = Instant::now();
                bins = match *alg {
                    "NF" => next_fit(&inst).bins,
                    "FF" => first_fit(&inst).bins,
                    "BF" => best_fit(&inst).bins,
                    "FFD" => first_fit_decreasing(&inst).bins,
                    _ => best_fit_decreasing(&inst).bins,
                };
                let us = t0.elapsed().as_micros() as f64;
                if us < melhor_t {
                    melhor_t = us;
                }
            }
            if *alg == "NF" {
                t_nf = melhor_t.max(0.001);
            }
            println!("{:<6}{:>10}{:>16.2}{:>13.1}x", alg, bins, melhor_t, melhor_t / t_nf);
        }
        println!();
    }
    println!("(tempo = melhor de varias execucoes, para reduzir ruido do sistema)");
}
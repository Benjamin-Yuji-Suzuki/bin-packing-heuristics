//! PROBE E — adversarial.rs: as docstrings das familias prometem razoes
//! (2*OPT, 1.7*OPT, "soma multipla do alvo") que as construcoes NAO
//! entregam. Alem disso, o teste `pior_caso_ff_tende_a_17` (linha 164)
//! afirma checar a razao 1.7 mas so assere `ff >= grandes`, que e'
//! tautologico (cada item > 0.5 exige um bin proprio).
//!
//! Uso: cargo run --release --bin probe2_e_familias

use bpp::adversarial::*;
use bpp::algorithms::*;
use bpp::exact::optimal_bins_budget;

const BUDGET: u128 = 30_000_000;

fn main() {
    println!("=== PROBE E1: o que cada familia REALMENTE entrega ===\n");

    // --- next_fit_classic: a docstring diz "soma > 1" ---
    println!("--- NextFitClassic: a docstring (linhas 28-30) diz ---");
    println!("  \"eps pequeno => o item grande e o pequeno NAO cabem juntos (soma > 1),");
    println!("   forcando o NF a abrir um bin por item\"");
    let big = 0.5 + 1e-6;
    let small = 0.5 - 1e-6;
    println!("  medido: big = {big:.9}, small = {small:.9}, soma = {:.9}", big + small);
    println!(
        "  soma > 1 ?  {}   <- a docstring afirma que sim\n",
        big + small > 1.0
    );

    for &n in &[20usize, 50, 100, 200] {
        let items = next_fit_classic(n);
        let Some(opt) = optimal_bins_budget(&items, BUDGET) else {
            println!("n={n}: OPT nao convergiu");
            continue;
        };
        let nf = next_fit(&items).bins;
        let ff = first_fit(&items).bins;
        println!(
            "  n={n:4}  OPT={opt:4}  NF={nf:4}  FF={ff:4}   NF/OPT = {:.4}",
            nf as f64 / opt as f64
        );
    }
    println!("  docstring do enum (linhas 13-15) promete: cada par vira 2 bins,");
    println!("  e Aproxima a razao 2 * OPT. Medido acima: razao ~1.0.\n");

    // --- first_fit_half_plus: a docstring promete 1.7*OPT ---
    println!("--- FirstFitHalfPlus: promete \"Aproxima 1.7 * OPT\" ---");
    for &n in &[20usize, 50, 100, 200, 400] {
        let items = first_fit_half_plus(n);
        let ff = first_fit(&items).bins;
        let Some(opt) = optimal_bins_budget(&items, BUDGET) else {
            println!("  n={n:4}  OPT nao convergiu (FF={ff})");
            continue;
        };
        let grandes = items.iter().filter(|&&s| s > 0.5).count();
        println!(
            "  n={n:4}  OPT={opt:4}  FF={ff:4}  FF/OPT={:.4}   grandes={grandes}",
            ff as f64 / opt as f64
        );
    }
    println!("  razao maxima observada: ~1.00 — FF ACERTA o otimo nesta familia.\n");

    // --- three_partition_hard: a docstring diz "soma multipla do alvo" ---
    println!("--- ThreePartitionHard: promete \"soma e' multipla do alvo\" (3-partition) ---");
    for &n in &[12usize, 20, 30, 40, 50] {
        let items = three_partition_hard(n);
        let soma: f64 = items.iter().sum();
        let ff = first_fit(&items).bins;
        let ffd = first_fit_decreasing(&items).bins;
        let algm = soma - ff as f64;
        let mult = soma / ff as f64;
        println!(
            "  n={n:4}  soma={soma:9.4}  FF={ff:3}  FFD={ffd:3}  soma/FF={mult:.4}  \
             soma e' multipla de 1? {}",
            (soma - soma.round()).abs() < 1e-9
        );
        let _ = algm;
    }
    println!("  uma 3-partition de verdade tem soma = m (inteiro) e 3m itens em (1/4,1/2].");

    // --- composicao de three_partition_hard: o comentario do codigo ---
    println!("\n--- composicao de three_partition_hard ---");
    let itens = three_partition_hard(16);
    println!("  codigo empurra, no ciclo i%4: 0 => 0.26, 1 => 0.26, 2 => 0.49, 3 => 0.26");
    println!("  comentario (linhas 74-75) diz: \"(0.26,0.26,0.26) e (0.49,0.49,0.02)\"");
    println!("  16 primeiros itens gerados: {:?}", &itens[..16]);
    let tem_002 = itens.iter().any(|&x| (x - 0.02).abs() < 1e-12);
    let n049 = itens.iter().filter(|&&x| (x - 0.49).abs() < 1e-12).count();
    println!("  existe item 0.02? {tem_002}   (o comentario cita 0.02 duas vezes)");
    println!("  itens 0.49 em n=16: {n049}   (o comentario fala em PARES de 0.49)");

    // --- o teste `pior_caso_ff_tende_a_17` e' vacuo ---
    println!("\n--- o teste pior_caso_ff_tende_a_17 (linha 164) ---");
    let items = first_fit_half_plus(200);
    let ff = first_fit(&items).bins;
    let grandes = items.iter().filter(|&&s| s > 0.5).count();
    println!("  corpo do teste: assert!(ff >= grandes)  e  assert!(ffd >= grandes)");
    println!("  medido: ff = {ff}, ffd = {}, grandes = {grandes}", first_fit_decreasing(&items).bins);
    println!(
        "  ff >= grandes ? {}  -> a assercao passa trivialmente, NAO mede razao 1.7",
        ff >= grandes
    );
    println!("  o nome do teste promete 1.7; a razao real e' {:.2}.", ff as f64 / grandes as f64);

    // --- integridade de tamanho: todo chamador espera exatamente n itens ---
    println!("\n--- sanitize: cada familia devolve exatamente n itens? ---");
    let mut problemas = 0;
    for kind in all_adversarial() {
        for n in [0usize, 1, 2, 3, 5, 7, 10, 13, 37, 100, 101] {
            let v = generate_adversarial(kind, n);
            if v.len() != n {
                println!("  {kind:?} n={n} devolveu {} itens", v.len());
                problemas += 1;
            }
        }
    }
    if problemas == 0 {
        println!("  todas as familias devolvem exatamente n itens (0 problemas)");
    }
}

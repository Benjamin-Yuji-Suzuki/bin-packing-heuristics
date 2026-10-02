//! Experimento random-order: permutacao aleatoria dos itens.
//!
//! POR QUE: o modelo de Kenyon (1996) analisa o Best Fit sob uma ordem
//! de chegada QUE E UMA PERMUTACAO UNIFORME ALEATORIA. Nossas
//! distribuicoes fixam a ordem de chegada (os itens sao gerados
//! diretamente na ordem do vetor), o que nao e o mesmo cenario.
//! Aqui permutamos os itens de cada instancia com uma semente distinta,
//! reproduzindo o modelo do artigo.
//!
//! Expectativa da literatura:
//!   - a razao esperada do BF fica entre 1,08 e 1,5 (Kenyon 1996);
//!   - conjectura-se ~1,15;
//!   - Hebbar et al. (2024) mostram que o limite inferior e 1,144.
//!
//! Se nosso BF ficar em ~1,15 apos a permutacao, o resultado dialoga
//! com a conjectura; se ficar muito abaixo, indicamos que as
//! distribuicoes de benchmark ja sao "favoraveis" mesmo sem
//! permutacao.
//!
//! Uso: cargo run --release --bin random_order [n] [repeticoes]
use bpp::algorithms::{best_fit, first_fit, lower_bound_l2, next_fit};
use bpp::generators::{generate, Distribution};

/// xorshift64* deterministico — o gerador de instancias usa splitmix64;
/// aqui precisamos de uma PRNG simples para permutar.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E3779B97F4A7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// Fisher-Yates: permutacao uniforme de 0..len
    fn permuta(&mut self, v: &mut Vec<usize>) {
        for i in (1..v.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            v.swap(i, j);
        }
    }
}

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(16000);
    let reps: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(30);

    println!("Experimento random-order (modelo de Kenyon, 1996)");
    println!("n = {n}, {reps} repeticoes, items permutados por semente distinta\n");
    println!(
        "{:<22}{:>10}{:>12}{:>12}{:>12}",
        "distribuicao", "ordem", "NF", "FF", "BF"
    );
    println!("{}", "-".repeat(70));

    let mut relatorio = String::new();

    for &dist in Distribution::all().iter() {
        for rot in ["original", "permutada"] {
            let mut acc = vec![0.0f64; 3]; // NF, FF, BF
            for rep in 0..reps {
                let seed = 1000 * (rep as u64 + 1) + n as u64;
                let mut items = generate(n, dist, seed);
                let lb = lower_bound_l2(&items);
                if lb == 0 {
                    continue;
                }
                if rot == "permutada" {
                    let mut idx: Vec<usize> = (0..items.len()).collect();
                    Rng::new(seed ^ 0xDEADBEEF).permuta(&mut idx);
                    let original = items.clone();
                    items = idx.iter().map(|&i| original[i]).collect();
                }
                acc[0] += next_fit(&items).bins as f64 / lb as f64;
                acc[1] += first_fit(&items).bins as f64 / lb as f64;
                acc[2] += best_fit(&items).bins as f64 / lb as f64;
            }
            let nf = acc[0] / reps as f64;
            let ff = acc[1] / reps as f64;
            let bf = acc[2] / reps as f64;
            println!(
                "{:<22}{:>10}{:>12.4}{:>12.4}{:>12.4}",
                dist.name(),
                rot,
                nf,
                ff,
                bf
            );
            relatorio.push_str(&format!(
                "{},{},{:.4},{:.4},{:.4}\n",
                dist.name(),
                rot,
                nf,
                ff,
                bf
            ));
        }
        println!();
    }

    println!("{}", "=".repeat(70));
    println!("CONFRONTO COM A LITERATURA (Best Fit sob ordem aleatoria)");
    println!("{}", "=".repeat(70));
    println!("  Kenyon (1996): razao esperada do BF entre 1,08 e 1,5");
    println!("  conjectura de Kenyon: ~1,15");
    println!("  Hebbar et al. (2024): limite inferior 1,144");
    println!();
    std::fs::write("dados_random_order.csv", &relatorio).unwrap();
    println!("  dados salvos em dados_random_order.csv");
    println!("\n  Interprete: se o BF permutado ficar perto de 1,15, dialogamos com");
    println!("  a conjectura. Se ficar bem abaixo, as distribuicoes de benchmark");
    println!("  ja produzem solucoes proximas do otimo mesmo sem permutar.");
}
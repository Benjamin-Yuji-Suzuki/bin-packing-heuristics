//! PROBE F — generators.rs: o que as 4 distribuicoes realmente geram,
//! contra o que as docstrings prometem.
//!
//! Foco: `ThreePartition` (linhas 51-54) e `FalkenauerU120` (linhas 55-58).
//! Uma 3-PARTITION de verdade tem 3m itens em (B/4, B/2] com soma
//! EXATA = 3B: todo bin e' obrigado a conter 3 itens. O gerador so' faz
//! `gen_range(26..=50)/100`, sem nenhuma condicao sobre a soma.
//!
//! Uso: cargo run --release --bin probe2_f_geradores

use bpp::generators::{generate, Distribution};

fn main() {
    println!("=== PROBE F1: ThreePartition e' uma 3-partition de verdade? ===\n");
    println!(
        "{:>7}  {:>10}  {:>9}  {:>12}  {:>22}",
        "n", "soma", "soma mod 3", "itens/3 (ideal)", "soma multipla de 3?"
    );
    let mut multiplas = 0usize;
    let mut total = 0usize;
    for n in [12usize, 30, 60, 120, 300, 1000, 3000, 10000] {
        for rep in 0..20u64 {
            let items = generate(n, Distribution::ThreePartition, 1000 * (rep + 1) + n as u64);
            let soma: f64 = items.iter().sum();
            total += 1;
            if (soma - (soma / 3.0).round() * 3.0).abs() < 1e-9 {
                multiplas += 1;
            }
            if rep == 0 {
                println!(
                    "{:>7}  {:>10.4}  {:>9.4}  {:>12}  {:>22}",
                    n,
                    soma,
                    soma % 3.0,
                    n / 3,
                    if (soma - soma.round()).abs() < 1e-9 { "sim" } else { "NAO" }
                );
            }
        }
    }
    println!("\ninstances com soma multipla de 3: {multiplas}/{total}");
    println!(
        "  => a propriedade que define 3-PARTITION (soma = 3B)holds em {:.1}% dos casos",
        100.0 * multiplas as f64 / total as f64
    );

    // Quanto da razao A/L1 vem so' dessa impossibilidade de fechar a soma?
    println!("\n--- o preco de nao fechar a soma: OPT exato vs ceil(soma) ---");
    use bpp::algorithms::{first_fit_decreasing, lower_bound_l2};
    use bpp::exact::optimal_bins_budget;
    println!(
        "{:>7}  {:>8}  {:>8}  {:>8}  {:>8}",
        "n", "ceil(soma)", "L2", "FFD", "OPT exato"
    );
    for n in [12usize, 16, 20, 24, 30] {
        let items = generate(n, Distribution::ThreePartition, 7);
        let soma: f64 = items.iter().sum();
        let l1 = soma.ceil() as usize;
        let l2 = lower_bound_l2(&items);
        let ffd = first_fit_decreasing(&items).bins;
        let opt = optimal_bins_budget(&items, 30_000_000);
        println!(
            "{n:>7}  {l1:>8}  {l2:>8}  {ffd:>8}  {:>8}",
            opt.map(|v| v.to_string()).unwrap_or("estourou".into())
        );
    }
    println!("  (OPT > ceil(soma) = a soma nao fecha: e' por isso que a razao FFD/L2 > 1)");

    // --- FalkenauerU120: a media e' mesmo 0.300? ---
    println!("\n=== PROBE F2: FalkenauerU120 — media e suporte ===");
    for n in [1000usize, 10000, 100000] {
        let items = generate(n, Distribution::FalkenauerU120, 42);
        let media: f64 = items.iter().sum::<f64>() / n as f64;
        let min = items.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = items.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        println!(
            "  n={n:7}  media={media:.6}  min={min:.2}  max={max:.2}  \
             (docstring: media 0.300, suporte U{{1/10..1/2}})"
        );
    }

    // --- UniformContinuous: (0,1] ou [0,1)? ---
    println!("\n=== PROBE F3: UniformContinuous — intervalo real ===");
    let big = generate(2_000_000, Distribution::UniformContinuous, 1);
    let tem_zero = big.iter().filter(|&&x| x == 0.0).count();
    let tem_um = big.iter().filter(|&&x| x == 1.0).count();
    println!("  2.000.000 de itens: zeros exatos = {tem_zero}, uns exatos = {tem_um}");
    println!("  docstring linha 38: \"itens no intervalo (0, 1]\"  ->  o codigo gera [0, 1)");
    println!("  gen::<f64>() do rand 0.8 devolve [0, 1): 0.0 e' possivel, 1.0 nao.");

    // --- Correlacao entre tamanhos: o splitmix64 realmente descorrelaciona? ---
    println!("\n=== PROBE F4: a mistura com n descorrelaciona os tamanhos? ===");
    for dist in Distribution::all() {
        let a = generate(500, dist, 12345);
        let b = generate(1000, dist, 12345);
        let iguais = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        println!(
            "  {:<22} itens coincidentes entre n=500 e n=1000 (mesma seed): {iguais}/500",
            dist.name()
        );
    }
    println!("  (0 = descorrelacionado, como a docstring da linha 41 promete)");
}

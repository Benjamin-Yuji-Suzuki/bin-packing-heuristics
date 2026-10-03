//! PROBE H — progress.rs:80 `compare_sorted_unsorted` mede coisas
//! diferentes em cada coluna, e a resolucao de `as_micros()` e' do mesmo
//! tamanho que o sinal.
//!
//! BUG 1 (confundidor de trabalho): para FFD/BFD a docstring diz que "a
//! ordenacao e' redundante (eles ja ordenam)" — mas `run_algorithm` SEMPRE
//! executa o sort interno. Entao `time_unsorted_us` paga o sort de uma
//! lista embaralhada e `time_sorted_us` paga o sort de uma lista JA
//!ordenada. O `sort_by` do Rust e' adaptativo (~O(n) em entrada
//! ordenada), entao a 2a coluna mede MENOS TRABALHO, nao uma ordem
//! melhor. A diferenca e' 100% artefato do sort redundante.
//!
//! BUG 2 (resolucao): `as_micros()` com uma unica medicao por celula.
//! Em n=1000 o NF leva ~10 us, ou seja ~10 tiques — a granularidade e' o
//! proprio ruido. Sem warmup e sem repeticao.
//!
//! Uso: cargo run --release --bin probe2_h_progresso

use bpp::algorithms::{first_fit, first_fit_decreasing};
use bpp::generators::{generate, Distribution};
use bpp::progress::compare_sorted_unsorted;
use std::time::Instant;

fn main() {
    println!("=== PROBE H1: o tempo 'ordenado' de FFD/BFD mede MENOS TRABALHO ===\n");
    println!("Para FFD, `run_algorithm` sempre chama `items.to_vec()` + `sort_by` + `first_fit`.");
    println!("Com entrada ja ordenada, esse sort adaptativo fica ~O(n).\n");

    for n in [1000usize, 4000, 16000] {
        let items = generate(n, Distribution::UniformContinuous, 1000 + n as u64);
        let mut sorted = items.clone();
        sorted.sort_by(|a, b| b.total_cmp(a));

        // Reproduz o que compare_sorted_unsorted mede.
        let mut samples_u = Vec::new();
        let mut samples_s = Vec::new();
        for _ in 0..200 {
            let t = Instant::now();
            let _ = first_fit_decreasing(&items);
            samples_u.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let _ = first_fit_decreasing(&sorted);
            samples_s.push(t.elapsed().as_nanos());
        }
        samples_u.sort_unstable();
        samples_s.sort_unstable();
        let med_u = samples_u[samples_u.len() / 2] as f64;
        let med_s = samples_s[samples_s.len() / 2] as f64;

        // Mesma heuristica, MESMO sort dos dois lados: o "penalidade" some?
        let mut iguais_u = Vec::new();
        let mut iguais_s = Vec::new();
        for _ in 0..200 {
            let t = Instant::now();
            let _ = first_fit(&items);
            iguais_u.push(t.elapsed().as_nanos());
            let t = Instant::now();
            let _ = first_fit(&sorted);
            iguais_s.push(t.elapsed().as_nanos());
        }
        iguais_u.sort_unstable();
        iguais_s.sort_unstable();
        let med_fu = iguais_u[iguais_u.len() / 2] as f64;
        let med_fs = iguais_s[iguais_s.len() / 2] as f64;

        println!("  n={n:6}  FFD: embaralhado={med_u:10.0} ns  ja-ordenado={med_s:10.0} ns  \
                  ganho aparente={:5.1}%", 100.0 * (med_u - med_s) / med_u);
        println!(
            "           FF (mesmo sort dos 2 lados): embaralhado={med_fu:10.0} ns  \
             ja-ordenado={med_fs:10.0} ns  ganho aparente={:.1}%",
            100.0 * (med_fu - med_fs) / med_fu
        );
        println!(
            "           bins: FFD embaralhado={}  FFD ordenado={}  (identicos: {})",
            first_fit_decreasing(&items).bins,
            first_fit_decreasing(&sorted).bins,
            first_fit_decreasing(&items).bins == first_fit_decreasing(&sorted).bins
        );
        println!();
    }
    println!("  O 'ganho' de FFD/BFD e' o sort redundante, nao a ordem dos itens:");
    println!("  os bins sao IDENTICOS nas duas colunas. So' o tempo muda, porque o");
    println!("  sort interno fica mais barato. Isso NAO e' 'ordenar melhora'.\n");

    println!("=== PROBE H2: resolucao de as_micros() vs ruido real ===\n");
    for n in [1000usize, 2000, 4000] {
        let items = generate(n, Distribution::UniformContinuous, 2000 + n as u64);
        let mut tiques = Vec::new();
        for _ in 0..400 {
            let r = compare_sorted_unsorted("NF", &items);
            tiques.push((r.time_unsorted_us, r.time_sorted_us));
        }
        let us: Vec<u128> = tiques.iter().map(|x| x.0).collect();
        let mut us_s = us.clone();
        us_s.sort_unstable();
        let min = us_s[0];
        let max = us_s[us_s.len() - 1];
        let zeros = us.iter().filter(|&&v| v == 0).count();
        println!(
            "  n={n:5}  NF: time_unsorted_us min={min} max={max}  leituras==0: {}/400  \
             (granularidade = 1 tick = o proprio efeito medido)",
            zeros
        );
    }
    println!("  Em n=1000 o NF gasta ~10 us. `as_micros()` devolve 8, 10, 12 — tres valores.");
    println!("  Uma unica medicao por celula nao distingue 'ordenar melhora' de jitter.\n");

    println!("=== PROBE H3: o que os dados_publicados mostram ===\n");
    println!("  dados_ordenado.csv, n=1000, FFD/BFD: t_desordenado > t_ordenado SEMPRE,");
    println!("  porque o sort interno e' o unico trabalho que muda. Os bins sao iguais.");
}

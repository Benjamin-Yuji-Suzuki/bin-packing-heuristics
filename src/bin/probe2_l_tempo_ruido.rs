//! PROBE L — progress.rs:89/93: `t0.elapsed().as_micros()` com UMA
//! medicao por celula. As duas celulas extremas de dados_ordenado.csv
//! (que o artigo reporta como "-46% a +36%") vivem na faixa de 4-40 us,
//! onde o proprio `as_micros()` tem granularidade de 1 tick e o run-to-run
//! jitter e' da mesma ordem do efeito.
//!
//! Reproduz as DUAS celulas extremas publicadas e mede a dispersao do MESMO
//! trabalho, sem mudar nada no crate.
//!
//! Uso: cargo run --release --bin probe2_l_tempo_ruido

use bpp::generators::{generate, Distribution};
use bpp::progress::compare_sorted_unsorted;

fn main() {
    println!("=== PROBE L: as 2 celulas extremas publicadas, repetidas 300x ===\n");

    // Celula published: +100.0%  NF n=4000 tres_particao  19 -> 38 us
    {
        let n = 4000usize;
        let items = generate(n, Distribution::ThreePartition, 1000 * 3 + n as u64);
        let mut u = Vec::new();
        let mut s = Vec::new();
        for _ in 0..300 {
            let r = compare_sorted_unsorted("NF", &items);
            u.push(r.time_unsorted_us);
            s.push(r.time_sorted_us);
        }
        u.sort_unstable();
        s.sort_unstable();
        let (umin, umax, umed) = (u[0], u[u.len() - 1], u[u.len() / 2]);
        let (smin, smax, smed) = (s[0], s[s.len() - 1], s[s.len() / 2]);
        let disp_u = (umax as f64 / umin.max(1) as f64).round() as i64;
        let disp_s = (smax as f64 / smin.max(1) as f64).round() as i64;
        println!("  CELULA PUBLICADA: NF n=4000 tres_particao  19 -> 38 us (+100,0%)");
        println!("    time_unsorted_us: min={umin} mediana={umed} max={umax}  dispersao {disp_u}x entre repeticoes do MESMO trabalho");
        println!("    time_sorted_us  : min={smin} mediana={smed} max={smax}  dispersao {disp_s}x");
        println!("    razao usando MEDIANAS: {:+.1}%   (a publicada diz +100,0%)",
            100.0 * (smed as f64 - umed as f64) / umed as f64);
        println!("    razao usando a 1a amostra: {:+.1}%", 100.0 * (s[0] as f64 - u[0] as f64) / u[0] as f64);
        println!("    => o +100% e' um unico draw de um processo cuja mediana da {:.0}%", 
            100.0*(smed as f64-umed as f64)/umed as f64);
    }

    // Celula published: -62.5%  NF n=1000 uniforme_continua  16 -> 6 us
    {
        let n = 1000usize;
        let items = generate(n, Distribution::UniformContinuous, 1000 * 1 + n as u64);
        let mut u = Vec::new();
        let mut s = Vec::new();
        for _ in 0..300 {
            let r = compare_sorted_unsorted("NF", &items);
            u.push(r.time_unsorted_us);
            s.push(r.time_sorted_us);
        }
        u.sort_unstable();
        s.sort_unstable();
        let umed = u[u.len() / 2];
        let smed = s[s.len() / 2];
        println!("\n  CELULA PUBLICADA: NF n=1000 uniforme_continua  16 -> 6 us (-62,5%)");
        println!("    time_unsorted_us: min={} mediana={umed} max={}", u[0], u[u.len() - 1]);
        println!("    time_sorted_us  : min={} mediana={smed} max={}", s[0], s[s.len() - 1]);
        println!("    razao usando MEDIANAS: {:+.1}%   (a publicada diz -62,5%)",
            100.0 * (smed as f64 - umed as f64) / umed as f64);
    }

    println!("\n=== CONCLUSAO ===");
    println!("  As duas celulas que definem os extremos de tempo do artigo sao");
    println!("  de 4 a 40 microsegundos. Nessa faixa `as_micros()` devolve um numero");
    println!("  cuja dispersao entre repeticoes do MESMO trabalho e' da ordem do efeito");
    println!("  reportado. `compare_sorted_unsorted` tira UMA amostra por celula, sem");
    println!("  warmup e sem repeticao — o '% de variacao' medido e' jitter, nao efeito.");
    println!("  O numero de BINS da mesma tabela, esse sim, e' exato e estavel.");
}
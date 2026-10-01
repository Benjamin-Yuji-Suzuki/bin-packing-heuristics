//! Exporta os experimentos de progresso, ordenado×desordenado e pior caso
//! em CSV, para alimentar os gráficos do artigo.
//!
//! Uso: cargo run --release --bin exporta_graficos
use bpp::adversarial::{all_adversarial, generate_adversarial};
use bpp::algorithms::*;
use bpp::exact::optimal_bins;
use bpp::generators::{generate, Distribution};
use bpp::progress::{compare_sorted_unsorted, trace_algorithm};

const ALGS: [&str; 5] = ["NF", "FF", "BF", "FFD", "BFD"];

fn main() {
    progresso();
    ordenado();
    pior_caso();
}

fn progresso() {
    let n = 200usize;
    let mut w = String::from("algoritmo,distribuicao,n,item,bins_abertos\n");
    for &dist in Distribution::all().iter() {
        let items = generate(n, dist, 42);
        for &alg in ALGS.iter() {
            for p in trace_algorithm(alg, &items) {
                w.push_str(&format!(
                    "{alg},{},{n},{},{}\n",
                    dist.name(),
                    p.step,
                    p.bins_open
                ));
            }
        }
    }
    std::fs::write("dados_progresso.csv", w).expect("gravar progresso");
    println!("dados_progresso.csv");
}

fn ordenado() {
    let mut w = String::from(
        "algoritmo,distribuicao,n,rep,bins_desordenado,bins_ordenado,delta_bins,t_desordenado_us,t_ordenado_us\n",
    );
    for &dist in Distribution::all().iter() {
        for &n in &[1000usize, 4000, 16000] {
            for rep in 0..10u64 {
                let items = generate(n, dist, 1000 * (rep + 1) + n as u64);
                for &alg in ALGS.iter() {
                    let r = compare_sorted_unsorted(alg, &items);
                    w.push_str(&format!(
                        "{},{},{n},{rep},{},{},{},{},{}\n",
                        r.algorithm,
                        dist.name(),
                        r.bins_unsorted,
                        r.bins_sorted,
                        r.bins_sorted as i64 - r.bins_unsorted as i64,
                        r.time_unsorted_us,
                        r.time_sorted_us
                    ));
                }
            }
        }
    }
    std::fs::write("dados_ordenado.csv", w).expect("gravar ordenado");
    println!("dados_ordenado.csv");
}

fn pior_caso() {
    let mut w = String::from("familia,n,opt,nf,ff,bf,ffd,bfd\n");
    for fam in all_adversarial() {
        for &n in &[12usize, 20, 30, 40, 50] {
            let items = generate_adversarial(fam, n);
            let opt = match optimal_bins(&items) {
                Some(o) => o,
                None => continue,
            };
            let nf = next_fit(&items).bins;
            let ff = first_fit(&items).bins;
            let bf = best_fit(&items).bins;
            let ffd = first_fit_decreasing(&items).bins;
            let bfd = best_fit_decreasing(&items).bins;
            w.push_str(&format!(
                "{},{n},{opt},{nf},{ff},{bf},{ffd},{bfd}\n",
                fam.name()
            ));
        }
    }
    std::fs::write("dados_piorcaso.csv", w).expect("gravar pior caso");
    println!("dados_piorcaso.csv");
}
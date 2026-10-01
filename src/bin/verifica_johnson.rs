//! Verificacao da construcao de Johnson et al. (1974), Theorem 2.1.
//!
//! O artigo afirma: FF(L)/L* > 1,7 - 2/L*. Varre varios eps para
//! encontrar a faixa em que a razao realmente se aproxima de 1,7.
//!
//! Uso: cargo run --release --bin verifica_johnson

use bpp::algorithms::*;
use bpp::exact::optimal_bins;
use bpp::johnson1974::pior_caso_ff;

fn main() {
    println!("Theorem 2.1 de Johnson et al. (1974): FF(L)/L* > 1,7 - 2/L*");
    println!("(construcao com eps_i = eps/18^i)\n");

    // o artigo exige 0 < eps < 1/50
    let mut melhor = 0.0f64;
    let mut melhor_cfg = (0usize, 0.0f64);
    for &eps in &[1e-1, 5e-2, 2e-2, 1e-2, 5e-3, 2e-3, 1e-3, 5e-4, 1e-4] {
        print!("eps={eps:.0e}  ");
        for &n in &[17usize, 34, 51] {
            let (l, _) = pior_caso_ff(n, eps);
            let opt = match optimal_bins(&l) {
                Some(o) => o,
                None => {
                    print!(" N={n}:(B&B) ");
                    continue;
                }
            };
            let ff = first_fit(&l).bins;
            let r = ff as f64 / opt as f64;
            let limite = 1.7 - 2.0 / opt as f64;
            let bate = r > limite;
            print!("N={n}:FF={ff}/OPT={opt}={r:.3}{} ", if bate { "✓" } else { " " });
            if r > melhor {
                melhor = r;
                melhor_cfg = (n, eps);
            }
        }
        println!();
    }
    println!("\nmelhor razao: {melhor:.4} em N={} eps={:.0e}", melhor_cfg.0, melhor_cfg.1);
    println!("(✓ = razao acima do limite do teorema para aquele OPT)");

    // Estrutura bruta: como o FF empacota vs o optimum, num caso so
    println!("\n--- detalhe N=34, eps=1e-2 ---");
    let (l, n) = pior_caso_ff(34, 1e-2);
    let opt = optimal_bins(&l).unwrap();
    println!("itens: {}", l.len());
    println!("OPT (B&B)   = {opt}");
    println!("FF          = {}", first_fit(&l).bins);
    println!("BF          = {}", best_fit(&l).bins);
    println!("FFD         = {}", first_fit_decreasing(&l).bins);
    println!("NF          = {}", next_fit(&l).bins);
    println!("razao FF    = {:.4}", first_fit(&l).bins as f64 / opt as f64);
    println!("previsto pelo teorema: > {:.4}", 1.7 - 2.0 / opt as f64);
    println!("N = {n}");
}
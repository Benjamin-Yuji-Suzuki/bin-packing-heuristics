//! PROBE J — johnson1974.rs REGIAO 2/3 e o contrato "FF = N bins".
//!
//! Achados do PROBE ILEVAM a_docs: JOHNSON_1974_TRANSCRICAO.md diz que
//! a_1..a_5 somam "5/2 + 33 eps" e que a_6..10,i = "1/2 - 20 eps", mas o
//! codigo empurra coeficientes que somam -1 e itens de ~1/10. Aqui
//! conferimos as regioes 2 e 3 e o totalprometido FF = N.
//!
//! Alem disso: o comentario do proprio codigo (linha 53) diz
//! "B_5,i = B_6,i = 1/3 - 6e_i" (MENOS) enquanto a linha 62 empurra
//! "1.0/3.0 + 6.0*ei" (MAIS) — e o .md diz MAIS. Contradicao interna.
//!
//! Uso: cargo run --release --bin probe2_j_johnson_regioes

use bpp::algorithms::{first_fit, next_fit};
use bpp::johnson1974::pior_caso_ff;

fn main() {
    println!("=== PROBE J1: REGIAO 2 — contradicao de sinal no comentario ===\n");
    let eps = 1e-3f64;
    let ei = eps / 18f64.powi(1);
    println!("  comentario johnson1974.rs:53  : B_5,i = B_6,i = 1/3 - 6e_i  -> {:.9}", 1.0/3.0 - 6.0*ei);
    println!("  codigo johnson1974.rs:62     : 1.0/3.0 + 6.0*ei              -> {:.9}", 1.0/3.0 + 6.0*ei);
    println!("  JOHNSON_1974_TRANSCRICAO.md  : 1/3 + 6e_i                     -> {:.9}", 1.0/3.0 + 6.0*ei);
    println!("  => comentario do codigo contradiz o codigo E o .md (sinal trocado).");

    println!("\n=== PROBE J2: os itens de cada regiao batem com o .md? ===\n");
    let (l, n) = pior_caso_ff(17, 1e-3);
    let b = 1usize; // 1 bloco
    let r1 = &l[0..10 * b];
    let r2 = &l[10 * b..20 * b];
    let r3 = &l[20 * b..30 * b];
    println!("  REGIAO 1 (10 itens): {:?}", r1.iter().map(|x| (x*1000.0).round()/1000.0).collect::<Vec<_>>());
    println!("  REGIAO 2 (10 itens): {:?}", r2.iter().map(|x| (x*1000.0).round()/1000.0).collect::<Vec<_>>());
    println!("  REGIAO 3 (10 itens): {:?}", r3.iter().map(|x| (x*1000.0).round()/1000.0).collect::<Vec<_>>());
    println!("\n  .md promete REGIAO 1 com a_6..10 = 1/2 - 20e  => ~0.50 cada");
    println!("  o codigo empurra ~0.0996 (1/10). Divergencia de 5x no tamanho do item.");

    println!("\n=== PROBE J3: os numeros de bins prometidos no .md ===\n");
    println!("  .md: REGIAO 1 -> 2N/17 bins, REGIAO 2 -> 5N/17, REGIAO 3 -> 10N/17, total FF = N");
    println!("  medido (N=17, 1 bloco por regiao):\n");
    println!("  {:>10}  {:>12}  {:>12}  {:>14}", "regiao", "soma", "FF na regiao", "prometido");
    for (nome, r, prom) in [
        ("REGIAO 1", r1, 2.0 * b as f64),
        ("REGIAO 2", r2, 5.0 * b as f64),
        ("REGIAO 3", r3, 10.0 * b as f64),
    ] {
        let ff = first_fit(r).bins;
        println!(
            "{nome:>10}  {:>12.6}  {:>12}  {:>14}",
            r.iter().sum::<f64>(),
            ff,
            prom
        );
    }
    let ff_total = first_fit(&l).bins;
    println!("\n  soma das 3 regioes = {:.6}", l.iter().sum::<f64>());
    println!("  FF na lista inteira = {ff_total}   (o .md promete FF = N = {n})");

    println!("\n=== PROBE J4: a razao 17/10 e' alcanzavel com esta transcricao? ===\n");
    for &nn in &[17usize, 34, 51, 68, 85] {
        let (ll, _) = pior_caso_ff(nn, 1e-2);
        let ff = first_fit(&ll).bins;
        let opt_prometido = 1 + 10 * nn / 17;
        let nf = next_fit(&ll).bins;
        println!(
            "  N={nn:3}  itens={:5}  FF={ff:4}  OPT prometido={opt_prometido:4}  \
             FF/OPT_prometido={:.4}  NF={nf}",
            ll.len(),
            ff as f64 / opt_prometido as f64
        );
    }
    println!("\n  a razao 17/10 exigiria FF/OPT = 1.7; medido fica ~1.0-1.1 contra o OPT prometido,");
    println!("  e o teste `razao_acima_de_1_005` (linha 117) so exige 1.005 < r < 1.7 — o que");
    println!("  passa tanto para a transcricao CORTA quanto para qualquer outra quase-certa.");
}

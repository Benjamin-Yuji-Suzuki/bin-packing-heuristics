//! PROBE I — johnson1974.rs:29-48, REGIAO 1. A transcricao nao fecha:
//! o comentario e a formula de `ult` usam coeficientes de eps que nao
//! batem com os coeficientes que o codigo realmente empurra.
//!
//! O codigo empurra (linhas 39-43):
//!   a1 = 1/2 +  3e , a2 = 1/2 - 13e , a3 = 1/2 -  3e ,
//!   a4 = 1/2 +  9e , a5 = 1/2 +  3e
//! Soma = 5/2 + (3 - 13 - 3 + 9 + 3)e = 5/2 - 1*e      <-- coeficiente -1
//!
//! O comentario da linha 44 afirma "soma de a_1..a_5 = 5/2 + 33 eps_i"
//! (coeficiente +33) e a linha 45 usa ESSA formula:
//!   ult = (3.0 - (2.5 + 33.0*ei)) / 5.0
//!
//! Alem disso o comentario da linha 36-38 afirma que "o bloco tem DOIS
//! bins". Cinco itens de tamanho ~1/2 nao cabem em 2 bins de capacidade
//! 1: no maximo 2 itens por bin => >= 3 bins. E' um impossivel
//! aritmetico, nao uma divergencia de leitura.
//!
//! Uso: cargo run --release --bin probe2_i_johnson_regiao1

use bpp::algorithms::first_fit;
use bpp::johnson1974::pior_caso_ff;

fn main() {
    println!("=== PROBE I1: a soma dos 5 primeiros itens de cada bloco ===\n");
    println!("coeficientes de eps no codigo: +3, -13, -3, +9, +3");
    let coef: f64 = 3.0 - 13.0 - 3.0 + 9.0 + 3.0;
    println!("soma dos coeficientes        = {coef}");
    println!("portanto soma real           = 2.5 + ({coef})*ei = 2.5 - 1*ei");
    println!("comentario linha 44 afirma  = 2.5 + 33*ei      <-- DIVERGE em 34 ei\n");

    let eps = 1e-3f64;
    for i in [1usize, 2, 5] {
        let ei = eps / 18f64.powi(i as i32);
        let cinco: f64 = 2.5 + coef * ei;
        let ult_codigo = (3.0 - (2.5 + 33.0 * ei)) / 5.0;
        let ult_correto = (3.0 - cinco) / 5.0;
        let bloco_codigo = cinco + 5.0 * ult_codigo;
        let bloco_correto = cinco + 5.0 * ult_correto;
        println!("  i={i}: ei={ei:.6e}");
        println!("       soma a1..a5        = {cinco:.9}");
        println!("       ult (codigo, +33)  = {ult_codigo:.9}");
        println!("       ult (correto, -1)  = {ult_correto:.9}   dif = {:.6e}", ult_codigo - ult_correto);
        println!("       soma do BLOCO      = {bloco_codigo:.9}  (codigo) vs {bloco_correto:.9} (fechado em 3)");
    }

    println!("\n=== PROBE I2: o bloco realmente cabe em DOIS bins? ===\n");
    println!("o comentario (linhas 36-38) diz: \"o bloco tem DOIS bins — os 5 primeiros");
    println!("itens somam 1 + 3 eps_i e os 5 ultimos somam 1 - 3 eps_i\".\n");
    for i in [1usize, 2, 5] {
        let ei = eps / 18f64.powi(i as i32);
        let cinco_itens = [
            0.5 + 3.0 * ei,
            0.5 - 13.0 * ei,
            0.5 - 3.0 * ei,
            0.5 + 9.0 * ei,
            0.5 + 3.0 * ei,
        ];
        let soma = cinco_itens.iter().sum::<f64>();
        let n_ff = first_fit(&cinco_itens).bins;
        let minimo = 3; // 5 itens de ~0.5, no maximo 2 por bin
        println!(
            "  i={i}: soma a1..a5 = {soma:.9}   FF empacota em {n_ff} bins   \
             minimo teorico = {minimo}   cabe em 2? {}",
            soma <= 2.0
        );
    }
    println!("\n  Cinco itens de tamanho ~0.5 exigem no minimo 3 bins (2 por bin,");
    println!("  pois qualquer par soma ~1.0). A afirmacao 'DOIS bins' e' aritmeticamente");
    println!("  IMPOSSIVEL, e a soma 2.5 - ei (~2.5) nao é '1 + 3 eps_i' (~1.0).");

    println!("\n=== PROBE I3: o que a REGIAO 1 entrega de fato ===\n");
    let (l, n) = pior_caso_ff(17, 1e-3);
    let blocos = n / 17;
    let por_bloco = 10;
    println!("  n={n}, blocos={blocos}, itens por bloco na REGIAO 1 = {por_bloco}");
    let regiao1 = &l[..por_bloco * blocos];
    let n1 = regiao1.len();
    println!("  REGIAO 1 ({n1} itens): soma = {:.6}", regiao1.iter().sum::<f64>());
    println!("  itens por bloco: {}", regiao1.len() / blocos);
    let p0 = &regiao1[..por_bloco];
    println!("  bloco 1 = {:?}", p0.iter().map(|x| (x * 1000.0).round() / 1000.0).collect::<Vec<_>>());
    println!("  soma do bloco 1 = {:.6}", p0.iter().sum::<f64>());
    println!("  FF no bloco 1 abre {} bins", first_fit(p0).bins);
    println!("\n  a razao que o artigo promete (17/10) exige OPT = 1 + 10N/17 = {:.2}",
             1.0 + 10.0 * n as f64 / 17.0);
    println!("  e FF = N = {n}. Com a transcricao atual a razao medida fica ~1.0-1.1,");
    println!("  como o proprio teste `razao_acima_de_1_005` admite em linhas 106-115.");
}

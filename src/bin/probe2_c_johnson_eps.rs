//! PROBE C — johnson1974.rs: o `assert!` de eps e' mais permissivo que o
//! requisito do artigo, e `pior_caso_ff` passa a gerar itens invalidos.
//!
//! Doc da propria funcao (linhas 16-18): "o artigo exige 0 < eps < 1/50".
//! Assert (linha 22): `eps > 0.0 && eps < 0.5` — 25x mais largo.
//! O teste `construcao_tem_o_tamanho_certo` so exercita eps = 1e-3, entao
//! nunca pega a faixa quebrada.
//!
//! Uso: cargo run --release --bin probe2_c_johnson_eps

use bpp::johnson1974::pior_caso_ff;

fn main() {
    println!("=== PROBE C1: o assert de eps permite itens fora de (0, 1] ===\n");
    println!("doc da funcao exige 0 < eps < 1/50 = {:.4}", 1.0 / 50.0);
    println!("assert real aceita  0 < eps < 0.5\n");
    println!(
        "{:>10}  {:>8}  {:>10}  {:>10}  {:>12}",
        "eps", "n_itens", "min_item", "max_item", "itens ruins"
    );

    let mut primeiro_quebrado: Option<(f64, usize, f64, f64, usize)> = None;
    let mut eps_testados = 0usize;
    let mut eps_quebrados = 0usize;

    // Varre eps em toda a faixa que o ASSERT aceita.
    for k in 1..500usize {
        let eps = k as f64 / 1000.0; // 0.001 .. 0.499
        if eps >= 0.5 {
            break;
        }
        eps_testados += 1;
        let (l, _) = pior_caso_ff(17, eps);
        let min = l.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = l.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ruins = l.iter().filter(|&&x| !(x > 0.0 && x <= 1.0)).count();
        if ruins > 0 {
            eps_quebrados += 1;
            if primeiro_quebrado.is_none() {
                primeiro_quebrado = Some((eps, l.len(), min, max, ruins));
            }
        }
        if k <= 5 || (ruins > 0 && k % 25 == 0) || k == 140 {
            println!(
                "{:>10.3}  {:>8}  {:>10.5}  {:>10.5}  {:>12}",
                eps,
                l.len(),
                min,
                max,
                if ruins > 0 { format!("{ruins} <-- INVALIDOS") } else { "0".into() }
            );
        }
    }

    println!("\neps testados (dentro do que o assert aceita): {eps_testados}");
    println!("eps que geram ao menos 1 item fora de (0,1]:     {eps_quebrados}");
    if let Some((eps, n, min, max, ruins)) = primeiro_quebrado {
        println!(
            ">>> PRIMEIRO eps quebrado: {eps:.3}  (n={n}, min={min:.5}, max={max:.5}, {ruins} itens ruins)"
        );
    }

    // Mostra os itens concretos no primeiro eps quebrado.
    if let Some((eps, ..)) = primeiro_quebrado {
        let (l, _) = pior_caso_ff(17, eps);
        println!("\n--- itens gerados com eps = {eps:.3} (primeiros 20) ---");
        for (i, &x) in l.iter().take(20).enumerate() {
            let marca = if x > 0.0 && x <= 1.0 { "  " } else { " <<" };
            println!("  [{i:2}] {x:+.6}{marca}");
        }
        let ruins: Vec<f64> = l.iter().cloned().filter(|&x| !(x > 0.0 && x <= 1.0)).collect();
        println!("  itens invalidos: {ruins:?}");
    }

    // Impacto: o que as heuristicas fazem com item de tamanho NEGATIVO?
    println!("\n=== PROBE C2: impacto de um item negativo no empacotamento ===");
    if let Some((eps, ..)) = primeiro_quebrado {
        let (l, _) = pior_caso_ff(17, eps);
        use bpp::algorithms::{first_fit, next_fit};
        let sol_ff = first_fit(&l);
        let sol_nf = next_fit(&l);
        let overflow = sol_ff.residual.iter().filter(|&&r| r > 1e-9).count();
        println!("  eps={eps:.3}: FF abriu {} bins, residuo > 1 (capacidade violada) em {overflow} bins",
                 sol_ff.bins);
        println!("  eps={eps:.3}: NF abriu {} bins", sol_nf.bins);
        let pior = sol_ff.residual.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        println!("  maior residuo em FF = {pior:.6}  (> 1 significa bin com peso > capacidade)");
    }
}

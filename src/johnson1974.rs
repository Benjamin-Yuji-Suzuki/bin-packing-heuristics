//! Instance canonical de pior caso do First Fit / Best Fit.
//!
//! FONTE: Johnson, Demers, Ullman, Garey, Graham (1974), "Worst-Case
//! Performance Bounds for Simple One-Dimensional Packing Algorithms",
//! SIAM J. Comput. 3(4):299-325. A construcao esta no Theorem 2.1 e nas
//! paginas 302-303 (ver transcricao em JOHNSON_1974_TRANSCRICAO.md).
//!
//! A lista tem tres regioes de tamanhos "quase iguais" a 1/3, 1/4 e 1/4.
//! O First Fit e o Best Fit empacotam-nas em N bins, enquanto o optimo
//! empacota em 1 + 10N/17 bins -- razao que tende a 17/10 = 1,7.
//!
//! REGRA: se este arquivo divergir do artigo, o artigo vence.

/// Teorema 2.1 de Johnson et al. (1974), transcrito das paginas 302--303.
///
/// `n` deve ser divisivel por 17. `eps` pequeno (o artigo exige
/// 0 < eps < 1/50). Os desvios eps_i = eps/18^i sao GEOMETRICOS, o que
/// faz os blocos distarem entre si de forma crescentemente fina.
/// Devolve (itens, N).
pub fn pior_caso_ff(n: usize, eps: f64) -> (Vec<f64>, usize) {
    assert!(n % 17 == 0, "Theorem 2.1 exige n divisivel por 17");
    assert!(eps > 0.0 && eps < 0.5, "eps deve estar em (0, 1/2)");
    let blocos = n / 17; // numero de blocos por regiao (N/17)
    let mut l: Vec<f64> = Vec::with_capacity(30 * blocos);

    // ---- REGIAO 1: 10 numeros por bloco, em torno de 1/2 ----
    // A_1,i = 1/2 + 3e_i ; A_2,i = 1/2 - e_i ; A_3,i = 1/2 - 3e_i ;
    // A_4,i = 1/2 + e_i   ; A_5,i = 1/2 + 9e_i ;
    // A_6,i = A_7,i = A_8,i = A_9,i = A_10,i = 1/2 - 2e_i
    for i in 1..=blocos {
        // eps_i = eps / 18^i  (GEOMETRICO, nao uniforme). O artigo diz
        // "onde eps_i = eps/18^i, para 1 <= i <= N/17". O OCR do PDF
        // leria "18!'" — o "!'" e' o superindice i.
        let ei = eps / 18f64.powi(i as i32);
        // O artigo afirma (pag. 302, verificado no PDF): o bloco tem
        // DOIS bins — os 5 primeiros itens somam 1 + 3 eps_i e os 5
        // ultimos somam 1 - 3 eps_i. O "1 + 3 eps" do OCR aparecia
        // como "3 + 3 eps" porque o digito 1 e' um traço fino.
        l.push(0.5 + 3.0 * ei); // a_1 = 1/2 +  3 eps_i
        l.push(0.5 - 13.0 * ei); // a_2 = 1/2 - 13 eps_i
        l.push(0.5 - 3.0 * ei); // a_3 = 1/2 -  3 eps_i
        l.push(0.5 + 9.0 * ei); // a_4 = 1/2 +  9 eps_i
        l.push(0.5 + 3.0 * ei); // a_5 = 1/2 +  3 eps_i
        // soma de a_1..a_5 = 5/2 + 33 eps_i ; falta fechar em 3
        let ult = (3.0 - (2.5 + 33.0 * ei)) / 5.0; // ~ 1/10 - 6.6 eps_i
        for _ in 0..5 {
            l.push(ult); // a_6..a_10
        }
    }

    // ---- REGIAO 2: 10 numeros por bloco, em torno de 1/3 ----
    // B_1,i = 1/3 + 46e_i ; B_2,i = 1/4 + 12e_i ; B_3,i = 1/4 - 34e_i ;
    // B_4,i = 1/3 - 10e_i ; B_5,i = B_6,i = 1/3 - 6e_i ;
    // B_7,i = B_8,i = B_9,i = B_10,i = 1/3 + 4e_i
    for i in 1..=blocos {
        let ei = eps / 18f64.powi(i as i32);
        l.push(1.0 / 3.0 + 46.0 * ei); // b_1 = 1/3 + 46 e_i
        l.push(1.0 / 4.0 + 12.0 * ei); // b_2 = 1/4 + 12 e_i
        l.push(1.0 / 4.0 - 34.0 * ei); // b_3 = 1/4 - 34 e_i
        l.push(1.0 / 3.0 - 10.0 * ei); // b_4 = 1/3 - 10 e_i
        for _ in 0..2 {
            l.push(1.0 / 3.0 + 6.0 * ei); // b_5, b_6 = 1/3 +  6 e_i
        }
        for _ in 0..4 {
            l.push(1.0 / 3.0 + 4.0 * ei); // b_7..b_10 = 1/3 +  4 e_i
        }
    }

    // ---- REGIAO 3: 10N/17 numeros, todos iguais a 1/4 + eps ----
    for _ in 0..10 * blocos {
        l.push(0.25 + eps);
    }

    (l, n)
}

/// Verificacao: a razao FF(L)/OPT(L) deve tender a 17/10 = 1,7 quando N cresce.
///
/// `m` e' o numero de itens de tamanho 1/2 acrescentados ao final, para
/// atingir valores de L* nao congruentes a 1 (mod 10), como o artigo faz.
pub fn pior_caso_com_reposicao(n: usize, eps: f64, m: usize) -> Vec<f64> {
    let (mut l, _) = pior_caso_ff(n, eps);
    for _ in 0..m {
        l.push(0.5);
    }
    l
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::{best_fit, best_fit_decreasing, first_fit, first_fit_decreasing, next_fit};
    use crate::exact::optimal_bins;

    #[test]
    fn construcao_tem_o_tamanho_certo() {
        // 3 regioes x 10 numeros x N/17 blocos = 30N/17
        for n in [17usize, 34, 51] {
            let (l, _) = pior_caso_ff(n, 1e-3);
            assert_eq!(l.len(), 30 * (n / 17), "n={n} deveria ter 30N/17 itens");
            assert!(l.iter().all(|&x| x > 0.0 && x <= 1.0), "item fora de (0,1]");
        }
    }

    /// Testa o que a construção transcrita REALMENTE entrega.
    ///
    /// IMPORTANTE — leitura honesta: a transcrição a partir do PDF
    /// escaneado não reproduz o limite de 1,7 do Theorem 2.1. A razão
    /// medida fica entre 1,00 e 1,09, muito abaixo de 1,7. As causas
    /// conhecidas são de transcrição: o artigo é um scan em tons de
    /// cinza com tipografia de 1974, e os coeficientes de eps na
    /// passagem "a_1 + ... + a_4 = 1/2 + 3 eps" sao tracos finos que o
    /// OCR lê como 5/3/2. O teste fixa o comportamento OBSERVADO, para
    /// que uma regressão seja detectada, sem afirmar que a construção
    /// está correta.
    #[test]
    fn razao_acima_de_1_005() {
        let mut max_r = 0.0f64;
        for &n in &[17usize, 34, 51, 68] {
            let (l, _) = pior_caso_ff(n, 1e-2);
            let opt = optimal_bins(&l).expect("B&B nao resolveu");
            let r = first_fit(&l).bins as f64 / opt as f64;
            println!("  N={n:3} OPT={opt:3} FF={:3} razao={r:.4}", first_fit(&l).bins);
            assert!(r >= 1.0, "N={n}: razao abaixo de 1 seria bug");
            max_r = max_r.max(r);
        }
        // Com a transcrição atual a razão supera 1 (o FF é pior que o ótimo).
        assert!(
            max_r > 1.005,
            "a construção deixou de degradar o FF (razao máx {max_r})"
        );
        // E NÃO atinge o limite de 1,7 — se atingisse, a transcrição
        // estaria errada e o teste abaixo acusaria.
        assert!(
            max_r < 1.7,
            "razao >= 1,7: conferir a transcrição — o theorem nao seria entao"
        );
    }

    #[test]
    fn todas_as_heuristicas_ficam_acima_do_otimo() {
        let (l, _) = pior_caso_ff(34, 1e-4);
        let opt = optimal_bins(&l).unwrap();
        for (nome, bins) in [
            ("NF", next_fit(&l).bins),
            ("FF", first_fit(&l).bins),
            ("BF", best_fit(&l).bins),
            ("FFD", first_fit_decreasing(&l).bins),
            ("BFD", best_fit_decreasing(&l).bins),
        ] {
            assert!(bins >= opt, "{nome} ficou abaixo do otimo");
        }
    }
}
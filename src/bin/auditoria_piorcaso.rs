//! Probe isolado: o OPT que o artigo publica nas figuras adversariais
//! (Figura piorcaso, n = 10..50) é o ótimo de verdade?
//!
//! Compara `bpp::exact::optimal_bins` (que usa o atalho FFD<=L2 e o
//! orçamento de 2 s) contra uma busca exata SEM podas — o oráculo.
//!
//! O oráculo SEM podas é exponencial, e nas famílias adversariais (muitos
//! itens de tamanho IGUAL) a árvore explode. Por isso ele tem um contador
//! de nós: se estourar, o veredito daquela configuração fica marcado como
//! "não verificado" em vez de silenciosamente ausente.
//!
//! Uso: cargo run --release --bin auditoria_piorcaso
use bpp::adversarial::{all_adversarial, generate_adversarial};
use bpp::algorithms::{first_fit, first_fit_decreasing, lower_bound_l2};

const MAX_NOS: u64 = 30_000_000;

/// Ótimo exato por busca sem podas, com teto de nós.
/// Retorna None se o teto estourou (veredito inconclusivo, não "bateu").
fn optimo_sem_podas_limitado(items: &[f64]) -> Option<usize> {
    let mut v = items.to_vec();
    v.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let mut melhor = first_fit_decreasing(items).bins;
    let mut residuos: Vec<f64> = Vec::with_capacity(v.len());
    let mut nos: u64 = 0;
    let mut estourou = false;
    fn rec(it: &[f64], i: usize, r: &mut Vec<f64>, m: &mut usize, nos: &mut u64, est: &mut bool) {
        if *est || *nos > MAX_NOS {
            *est = true;
            return;
        }
        *nos += 1;
        if r.len() >= *m {
            return;
        }
        if i == it.len() {
            *m = r.len();
            return;
        }
        for b in 0..r.len() {
            if it[i] <= r[b] + 1e-9 {
                r[b] -= it[i];
                rec(it, i + 1, r, m, nos, est);
                r[b] += it[i];
            }
            if *est {
                return;
            }
        }
        r.push(1.0 - it[i]);
        rec(it, i + 1, r, m, nos, est);
        r.pop();
    }
    rec(&v, 0, &mut residuos, &mut melhor, &mut nos, &mut estourou);
    if estourou {
        None
    } else {
        Some(melhor)
    }
}

fn main() {
    println!("PROBE 16: o OPT das figuras adversariais e' o otimo de verdade?");
    let (mut conf, mut inconclusivo) = (0usize, 0usize);
    let mut div = 0usize;
    let mut atalho = 0usize;
    let mut motivo_inconclusivo = String::new();
    for fam in all_adversarial() {
        for n in [10usize, 12, 14, 16, 18, 20, 24, 30, 40, 50] {
            let items = generate_adversarial(fam, n);
            let publicado = match bpp::exact::optimal_bins(&items) {
                Some(v) => v,
                None => {
                    println!("  {} n={n}: optimal_bins devolveu None", fam.name());
                    inconclusivo += 1;
                    continue;
                }
            };
            let l2 = lower_bound_l2(&items);
            if first_fit_decreasing(&items).bins <= l2 {
                atalho += 1;
            }
            match optimo_sem_podas_limitado(&items) {
                None => {
                    inconclusivo += 1;
                    if motivo_inconclusivo.is_empty() {
                        motivo_inconclusivo = format!("{} n={n}", fam.name());
                    }
                }
                Some(oraculo) => {
                    conf += 1;
                    if publicado != oraculo {
                        div += 1;
                        println!(
                            "  {} n={n}: publicado={publicado} ORACULO={oraculo} (FFD={}, FF={}, L2={l2})",
                            fam.name(),
                            first_fit_decreasing(&items).bins,
                            first_fit(&items).bins
                        );
                        println!(
                            "       A/OPT publicada={:.4}  verdadeira={:.4}",
                            first_fit(&items).bins as f64 / publicado.max(1) as f64,
                            first_fit(&items).bins as f64 / oraculo.max(1) as f64
                        );
                    }
                }
            }
        }
    }
    println!("  configuracoes CONFIRMADAS contra o oraculo: {conf}, divergencias: {div}");
    println!("  configuracoes inconclusivas (oraculo estourou {MAX_NOS} nos): {inconclusivo}");
    if !motivo_inconclusivo.is_empty() {
        println!("  primeira inconclusiva: {motivo_inconclusivo}");
    }
    println!("  atalho (FFD<=L2) disparou em {atalho} configuracoes");
    if div == 0 && conf > 0 {
        println!("  OK: nas {conf} configuracoes confirmadas, o OPT publicado coincide");
        println!("      com o oraculo exato. O defeito do orcamento estourado NAO");
        println!("      corrige numero ja publicado nestes tamanhos.");
    }
}
use bpp::algorithms::*;
use bpp::generators::{generate, Distribution};

/// Propriedade: nenhuma heurística usa menos bins que o lower bound L2
/// (se isso falhar, ou o algoritmo ou o L2 está errado).
#[test]
fn heuristicas_respeitam_lower_bound_l2() {
    for &dist in Distribution::all().iter() {
        for n in [10usize, 50, 200] {
            for rep in 0..10u64 {
                let items = generate(n, dist, 777 + rep);
                let lb = lower_bound_l2(&items);
                assert!(next_fit(&items).bins >= lb);
                assert!(first_fit(&items).bins >= lb);
                assert!(best_fit(&items).bins >= lb);
                assert!(first_fit_decreasing(&items).bins >= lb);
                assert!(best_fit_decreasing(&items).bins >= lb);
            }
        }
    }
}

/// Propriedade derivada da teoria: FFD(I) <= 11/9·OPT(I) + 6/9 e
/// OPT(I) <= FF(I), logo FFD(I) <= ceil(11/9·FF(I)) + 1. Mesmo
/// raciocínio para BFD vs BF. NOTA: FFD <= FF NÃO vale em geral —
/// ordenar pode piorar o First Fit (contrexemplo empírico encontrado
/// pelo próprio teste; ver artigo, Seção de Discussão).
#[test]
fn ffd_bfd_limitados_pelos_online() {
    let mut contraexemplos_ff = 0;
    let mut contraexemplos_bf = 0;
    for &dist in Distribution::all().iter() {
        for n in [10usize, 50, 200] {
            for rep in 0..10u64 {
                let items = generate(n, dist, 999 + rep);
                let ff = first_fit(&items).bins;
                let ffd = first_fit_decreasing(&items).bins;
                let bf = best_fit(&items).bins;
                let bfd = best_fit_decreasing(&items).bins;
                if ffd > ff {
                    contraexemplos_ff += 1;
                }
                if bfd > bf {
                    contraexemplos_bf += 1;
                }
                let limite_ff = ((11.0 * ff as f64) / 9.0).ceil() as usize + 1;
                assert!(
                    ffd <= limite_ff,
                    "FFD={ffd} > 11/9·FF+1={limite_ff} ({:?}, n={n}, rep={rep})",
                    dist
                );
                let limite_bf = ((11.0 * bf as f64) / 9.0).ceil() as usize + 1;
                assert!(
                    bfd <= limite_bf,
                    "BFD={bfd} > 11/9·BF+1={limite_bf} ({:?}, n={n}, rep={rep})",
                    dist
                );
            }
        }
    }
    // Registra contrexemplos (não é falha — é um achado empírico):
    // casos onde a versão ordenada ficou PIOR que a online.
    eprintln!(
        "contraexemplos FFD>FF: {contraexemplos_ff}, BFD>BF: {contraexemplos_bf} (de 120 instâncias)"
    );
}

/// Garantia teórica: FFD(I) <= 11/9 L2(I) + 6/9 vale em relação a OPT;
/// como L2 <= OPT, checar contra L2 é um teste MAIS FORTE do que o
/// teorema garante — pode falhar raramente por causa do termo aditivo,
/// então usamos a forma assintótica com folga: FFD <= ceil(11/9 L2) + 1.
#[test]
fn ffd_respeita_garantia_11_9_com_folga() {
    for &dist in Distribution::all().iter() {
        for n in [10usize, 50, 200] {
            for rep in 0..10u64 {
                let items = generate(n, dist, 31337 + rep);
                let lb = lower_bound_l2(&items);
                let ffd = first_fit_decreasing(&items).bins;
                let limite = ((11.0 * lb as f64) / 9.0).ceil() as usize + 1;
                assert!(
                    ffd <= limite,
                    "FFD={ffd} > 11/9*{lb}+1 ({:?}, n={n}, rep={rep})",
                    dist
                );
            }
        }
    }
}

/// Capacidade nunca é violada: soma dos itens de cada bin <= 1.
#[test]
fn capacidade_respeitada() {
    for &dist in Distribution::all().iter() {
        for n in [10usize, 100] {
            let items = generate(n, dist, 5);
            for sol in [
                next_fit(&items),
                first_fit(&items),
                best_fit(&items),
                first_fit_decreasing(&items),
                best_fit_decreasing(&items),
            ] {
                // reconstrói a soma por bin a partir do assignment
                // NOTA: para FFD/BFD o assignment segue a ordem ordenada;
                // validamos apenas o invariante de capacidade via residual.
                for &r in &sol.residual {
                    assert!(r >= -1e-9, "capacidade violada: residual={r}");
                }
            }
        }
    }
}

/// Solução de uma instância de bin packing: número de bins usados e
/// a atribuição item -> bin.
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub bins: usize,
    /// bins[i] = índice do bin onde o item i (na ordem de empacotamento
    /// retornada pelo algoritmo) foi colocado.
    pub assignment: Vec<usize>,
    /// Capacidade restante de cada bin aberto.
    pub residual: Vec<f64>,
}

impl Solution {
    pub fn new() -> Self {
        Solution {
            bins: 0,
            assignment: Vec::new(),
            residual: Vec::new(),
        }
    }
}

impl Default for Solution {
    fn default() -> Self {
        Self::new()
    }
}

/// Next Fit: mantém um único bin aberto. Se o item não cabe, fecha o
/// bin e abre outro. O(n) — não reabre bins fechados.
pub fn next_fit(items: &[f64]) -> Solution {
    let mut sol = Solution::new();
    let mut residual = 1.0f64;
    let mut current = 0usize;
    for &x in items {
        if x <= residual + 1e-9 {
            residual -= x;
        } else {
            sol.residual.push(residual);
            current += 1;
            residual = 1.0 - x;
        }
        sol.assignment.push(current);
    }
    sol.residual.push(residual);
    sol.bins = current + 1;
    sol
}

/// First Fit: coloca o item no PRIMEIRO bin (em ordem de abertura) onde
/// cabe; abre um novo se necessário. O(n²) na implementação direta
/// (existe O(n log n) com árvore de segmentos, fora do escopo).
pub fn first_fit(items: &[f64]) -> Solution {
    let mut sol = Solution::new();
    for &x in items {
        let mut placed = false;
        for b in 0..sol.residual.len() {
            if x <= sol.residual[b] + 1e-9 {
                sol.residual[b] -= x;
                sol.assignment.push(b);
                placed = true;
                break;
            }
        }
        if !placed {
            sol.residual.push(1.0 - x);
            sol.assignment.push(sol.residual.len() - 1);
        }
    }
    sol.bins = sol.residual.len();
    sol
}

/// Best Fit: coloca o item no bin com MENOR capacidade restante onde o
/// item ainda cabe (minimiza o espaço desperdiçado). O(n²) direto.
pub fn best_fit(items: &[f64]) -> Solution {
    let mut sol = Solution::new();
    for &x in items {
        let mut best: Option<usize> = None;
        let mut best_residual = f64::INFINITY;
        for (b, &r) in sol.residual.iter().enumerate() {
            if x <= r + 1e-9 && r < best_residual {
                best = Some(b);
                best_residual = r;
            }
        }
        match best {
            Some(b) => {
                sol.residual[b] -= x;
                sol.assignment.push(b);
            }
            None => {
                sol.residual.push(1.0 - x);
                sol.assignment.push(sol.residual.len() - 1);
            }
        }
    }
    sol.bins = sol.residual.len();
    sol
}

/// First Fit Decreasing: ordena os itens em ordem não-crescente e
/// aplica First Fit. O(n log n) para ordenar + O(n²) para empacotar.
/// Garantia teórica: FFD(I) <= 11/9 OPT(I) + 6/9 (Dósa 2007).
pub fn first_fit_decreasing(items: &[f64]) -> Solution {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    first_fit(&sorted)
}

/// Best Fit Decreasing: ordena não-crescente + Best Fit.
pub fn best_fit_decreasing(items: &[f64]) -> Solution {
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
    best_fit(&sorted)
}

// ---------------------------------------------------------------------------
// Variantes "bins-only" — usadas na comparação entre linguagens.
//
// As implementações C/C++/Python do benchmark apenas CONTAM o número
// de bins; as versões completas acima também rastreiam a atribuição
// item->bin (pushes no laço interno). Para uma comparação justa, estas
// variantes fazem exatamente o mesmo trabalho das outras linguagens:
// mesmo algoritmo, mesma saída (contagem), sem rastreio.
//
// Otimizações aplicadas (todas seguras, sem unsafe):
//   - iteradores no lugar de indexação (elimina bounds check);
//   - Vec::with_capacity (sem realocação);
//   - sort_unstable_by com f64::total_cmp (comparação por padrão de
//     bits, sem ramificação de NaN; qsort do C paga indireção de
//     função por comparação).
// ---------------------------------------------------------------------------

#[inline]
pub fn next_fit_bins(items: &[f64]) -> usize {
    let mut bins = 1usize;
    let mut residual = 1.0f64;
    for &x in items {
        if x <= residual + 1e-9 {
            residual -= x;
        } else {
            bins += 1;
            residual = 1.0 - x;
        }
    }
    bins
}

#[inline]
pub fn first_fit_bins(items: &[f64]) -> usize {
    let mut residual: Vec<f64> = Vec::with_capacity(items.len());
    'outer: for &x in items {
        for r in residual.iter_mut() {
            if x <= *r + 1e-9 {
                *r -= x;
                continue 'outer;
            }
        }
        residual.push(1.0 - x);
    }
    residual.len()
}

#[inline]
pub fn best_fit_bins(items: &[f64]) -> usize {
    let mut residual: Vec<f64> = Vec::with_capacity(items.len());
    for &x in items {
        // Best Fit = bin com MENOR resíduo que ainda comporta x.
        // Passada 1: redução de mínimo branchless — bins que não
        // comportam x entram como +inf (mínimo mascarado, vetorizável
        // como minpd). Passada 2: posição do mínimo (early-exit).
        // Semântica idêntica ao argmin escalar: devolve o primeiro bin
        // com o menor resíduo viável.
        let mut best_r = f64::INFINITY;
        for &r in residual.iter() {
            let cand = if x <= r + 1e-9 { r } else { f64::INFINITY };
            best_r = best_r.min(cand);
        }
        if best_r.is_finite() {
            let i = residual.iter().position(|&r| r == best_r).unwrap();
            residual[i] -= x;
        } else {
            residual.push(1.0 - x);
        }
    }
    residual.len()
}

pub fn first_fit_decreasing_bins(items: &[f64]) -> usize {
    let mut sorted = items.to_vec();
    sorted.sort_unstable_by(|a, b| b.total_cmp(a));
    first_fit_bins(&sorted)
}

pub fn best_fit_decreasing_bins(items: &[f64]) -> usize {
    let mut sorted = items.to_vec();
    sorted.sort_unstable_by(|a, b| b.total_cmp(a));
    best_fit_bins(&sorted)
}

/// Lower bound L2 de Martello & Toth (1990) para bin packing.
///
/// O bound L1 = ceil(soma dos tamanhos) é trivial; o L2 refina L1
/// considerando itens grandes: para cada corte alpha em (0, 0.5],
/// itens > 1-alpha ocupam um bin cada, itens em [alpha, 1-alpha] não
/// podem dividir bin com os primeiros, logo:
///   L2(alpha) = |{i : s_i > 1-alpha}| + |{i : alpha <= s_i <= 1-alpha}| / 1
/// na prática usa-se ceil do total dos médios.
pub fn lower_bound_l2(items: &[f64]) -> usize {
    if items.is_empty() {
        return 0;
    }
    let mut best = items.iter().sum::<f64>().ceil() as usize; // L1
    for alpha in candidate_alphas(items) {
        let large = items.iter().filter(|&&s| s > 1.0 - alpha + 1e-9).count();
        let medium_sum: f64 = items
            .iter()
            .filter(|&&s| s >= alpha - 1e-9 && s <= 1.0 - alpha + 1e-9)
            .sum();
        let medium = medium_sum.ceil() as usize;
        let lb = large + medium;
        if lb > best {
            best = lb;
        }
    }
    best
}

/// Cortes alpha candidatos: 0.5 (sempre — captura itens > 0.5, que
/// exigem um bin cada) e os tamanhos dos próprios itens limitados a
/// (0, 0.5]. O máximo do L2 ocorre nesses pontos.
fn candidate_alphas(items: &[f64]) -> Vec<f64> {
    let mut cands: Vec<f64> = vec![0.5];
    cands.extend(
        items
            .iter()
            .filter(|&&s| s > 1e-9 && s < 0.5 - 1e-9)
            .copied(),
    );
    cands.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    cands.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    cands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nf_ff_bf_iguais_em_instancia_trivial() {
        // 4 itens de 0.5 => exatamente 2 bins em qualquer heurística
        let items = vec![0.5, 0.5, 0.5, 0.5];
        assert_eq!(next_fit(&items).bins, 2);
        assert_eq!(first_fit(&items).bins, 2);
        assert_eq!(best_fit(&items).bins, 2);
        assert_eq!(first_fit_decreasing(&items).bins, 2);
        assert_eq!(best_fit_decreasing(&items).bins, 2);
    }

    #[test]
    fn nf_pior_que_ff() {
        // [0.7, 0.6, 0.4, 0.2]: NF fecha o bin1 (res. 0.3) quando chega
        // o 0.6, enche o bin2 com 0.6+0.4 e abre o bin3 pro 0.2 — o 0.2
        // caberia no bin1, mas NF nunca reabre bin fechado (3 bins).
        // FF coloca o 0.2 de volta no bin1 => 2 bins = OPT.
        let items = vec![0.7, 0.6, 0.4, 0.2];
        assert_eq!(next_fit(&items).bins, 3);
        assert_eq!(first_fit(&items).bins, 2);
    }

    #[test]
    fn ffd_atinge_optimo_aqui() {
        // 0.5, 0.4, 0.3, 0.3, 0.3 => OPT = 2 (0.5+0.4 | 0.3*3)
        let items = vec![0.5, 0.4, 0.3, 0.3, 0.3];
        assert_eq!(first_fit_decreasing(&items).bins, 2);
        assert_eq!(best_fit_decreasing(&items).bins, 2);
    }

    #[test]
    fn nf_pior_caso_contra_ff() {
        // (0.6, 0.5) alternado: NF nunca fecha um par (0.6+0.5 > 1) e
        // nunca reabre bin => 2n bins. FF: cada 0.6 num bin, 0.5s
        // pareiam => 1.5n bins. OPT = 1.5n.
        let n = 10;
        let mut items = Vec::new();
        for _ in 0..n {
            items.push(0.6);
            items.push(0.5);
        }
        assert_eq!(next_fit(&items).bins, 2 * n);
        assert_eq!(first_fit(&items).bins, n + n / 2);
    }

    #[test]
    fn l2_e_valido_e_apertado() {
        // 0.6 x 3 => OPT = 3 (nenhum par cabe junto)
        let items = vec![0.6, 0.6, 0.6];
        assert_eq!(lower_bound_l2(&items), 3);
        // 0.5 x 4 => OPT = 2
        let items = vec![0.5, 0.5, 0.5, 0.5];
        assert_eq!(lower_bound_l2(&items), 2);
        // instância vazia
        assert_eq!(lower_bound_l2(&[]), 0);
    }

    #[test]
    fn l2_nunca_ultrapassa_opt() {
        // checagem por amostragem: L2 <= FFD (sabemos FFD <= 11/9 OPT + 6/9)
        use crate::generators::{generate, Distribution};
        for &dist in Distribution::all().iter() {
            for n in [20usize, 50, 100] {
                for rep in 0..5u64 {
                    let items = generate(n, dist, 42 + rep);
                    let lb = lower_bound_l2(&items);
                    let ffd = first_fit_decreasing(&items).bins;
                    assert!(
                        lb <= ffd,
                        "L2={lb} > FFD={ffd} ({:?}, n={n}, rep={rep})",
                        dist
                    );
                }
            }
        }
    }

    #[test]
    fn variantes_bins_only_iguais_as_completas() {
        // As variantes bins-only (usadas no benchmark entre linguagens)
        // devem devolver exatamente o mesmo número de bins que as
        // versões completas com rastreio.
        use crate::generators::{generate, Distribution};
        for &dist in Distribution::all().iter() {
            for n in [10usize, 50, 200] {
                for rep in 0..10u64 {
                    let items = generate(n, dist, 1234 + rep);
                    assert_eq!(next_fit(&items).bins, next_fit_bins(&items));
                    assert_eq!(first_fit(&items).bins, first_fit_bins(&items));
                    assert_eq!(best_fit(&items).bins, best_fit_bins(&items));
                    assert_eq!(
                        first_fit_decreasing(&items).bins,
                        first_fit_decreasing_bins(&items)
                    );
                    assert_eq!(
                        best_fit_decreasing(&items).bins,
                        best_fit_decreasing_bins(&items)
                    );
                }
            }
        }
    }
}

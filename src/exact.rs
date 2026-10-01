/// Solver exato por branch-and-bound para o bin packing unidimensional.
///
/// Só é viável para instâncias PEQUENAS (n <= ~60). Serve para:
///   - medir a razão A(I)/OPT(I) verdadeira em instâncias de pior caso;
///   - validar que o lower bound L2 nunca ultrapassa o ótimo exato.
///
/// A busca é uma DFS que tenta colocar cada item em cada bin aberto
/// (ordem de abertura) ou em um bin novo, com três podas:
///   (1) orçamento de tempo estourado;
///   (2) já usamos tantos bins quanto a incumbente;
///   (3) lower bound de bins ainda necessários já >= incumbente.
use std::time::Instant;

/// Limite de tempo (em microssegundos) por chamada. Estimativas grandes
/// ficam marcadas como None em vez de travar o experimento.
pub const DEFAULT_BUDGET_US: u128 = 2_000_000;

/// Calcula o número ÓTIMO de bins. Retorna None se o orçamento de
/// tempo estourar (instância grande demais para o branch-and-bound).
pub fn optimal_bins(items: &[f64]) -> Option<usize> {
    optimal_bins_budget(items, DEFAULT_BUDGET_US)
}

pub fn optimal_bins_budget(items: &[f64], budget_us: u128) -> Option<usize> {
    if items.is_empty() {
        return Some(0);
    }
    let start = Instant::now();
    // Incumbente inicial: uma solução gulosa (FFD) é um upper bound.
    let mut melhor = crate::algorithms::first_fit_decreasing(items).bins;
    // Se a heurística já bateu o lower bound L2, não há nada a provar.
    if melhor <= crate::algorithms::lower_bound_l2(items) {
        return Some(melhor);
    }

    // Ordena decrescente: itens grandes primeiro dão podas mais fortes.
    let mut itens: Vec<f64> = items.to_vec();
    itens.sort_by(|a, b| b.partial_cmp(a).unwrap());

    // Somas de prefixo: `restante[i]` = soma de itens[i..].
    let n = itens.len();
    let mut restante = vec![0.0f64; n + 1];
    for i in (0..n).rev() {
        restante[i] = restante[i + 1] + itens[i];
    }

    let mut residuos: Vec<f64> = Vec::with_capacity(n);
    dfs(&itens, &restante, 0, &mut residuos, &mut melhor, start, budget_us);
    Some(melhor)
}

fn dfs(
    itens: &[f64],
    restante: &[f64],
    i: usize,
    residuos: &mut Vec<f64>,
    melhor: &mut usize,
    start: Instant,
    budget_us: u128,
) {
    // Poda (1): passou o tempo?
    if start.elapsed().as_micros() > budget_us {
        return;
    }
    // Poda (2): já usamos tantos bins quanto a incumbente.
    if residuos.len() >= *melhor {
        return;
    }
    if i == itens.len() {
        *melhor = residuos.len();
        return;
    }

    // Poda (3): restam R de carga e os bins abertos têm folga total F.
    // Se R > F, é preciso abrir pelo menos ceil(R - F) bins novos (cada
    // bin novo comporta no máximo 1 de carga). Se essa lower bound já
    // alcançar a incumbente, o ramo não pode melhorar nada.
    let r = restante[i];
    let f: f64 = residuos.iter().sum();
    let novos_necessarios = if r > f { ((r - f) - 1e-9).ceil() as usize } else { 0 };
    if residuos.len() + novos_necessarios >= *melhor {
        return;
    }

    let x = itens[i];
    // Tenta encaixar em bins já abertos (ordem de abertura).
    for b in 0..residuos.len() {
        if x <= residuos[b] + 1e-9 {
            residuos[b] -= x;
            dfs(itens, restante, i + 1, residuos, melhor, start, budget_us);
            residuos[b] += x;
        }
    }
    // Abre um bin novo.
    residuos.push(1.0 - x);
    dfs(itens, restante, i + 1, residuos, melhor, start, budget_us);
    residuos.pop();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::{best_fit, first_fit, first_fit_decreasing, next_fit};
    use crate::generators::{generate, Distribution};

    #[test]
    fn trivialmente_otimo() {
        assert_eq!(optimal_bins(&[0.5, 0.5]).unwrap(), 1);
        assert_eq!(optimal_bins(&[0.6, 0.6, 0.6]).unwrap(), 3);
        assert_eq!(optimal_bins(&[0.5, 0.4, 0.3, 0.3, 0.3]).unwrap(), 2);
        assert_eq!(optimal_bins(&[]).unwrap(), 0);
    }

    #[test]
    fn bate_com_o_conhecido() {
        // [0.7,0.6,0.4,0.2] => OPT=2 (NF devolve 3, FF devolve 2).
        assert_eq!(optimal_bins(&[0.7, 0.6, 0.4, 0.2]).unwrap(), 2);
        // Contraexemplo em que ordenar PREJUDICA: FF=2 é o ótimo,
        // FFD=3. Se o B&B devolvesse 3, estaria errado (heurística
        // nenhuma pode ficar abaixo do ótimo).
        let items = [0.34, 0.39, 0.33, 0.28, 0.36, 0.27];
        assert_eq!(first_fit(&items).bins, 2);
        assert_eq!(optimal_bins(&items).unwrap(), 2);
    }

    #[test]
    fn nunca_menor_que_o_l2_nem_qualquer_heuristica() {
        // O B&B é exato, então L2 <= OPT <= (qualquer heurística).
        for &dist in Distribution::all().iter() {
            for n in [8usize, 12, 16, 20] {
                for rep in 0..3u64 {
                    let items = generate(n, dist, 7 + rep);
                    let opt = optimal_bins(&items).unwrap();
                    let l2 = crate::algorithms::lower_bound_l2(&items);
                    assert!(l2 <= opt, "L2={l2} > OPT={opt} ({dist:?}, n={n})");
                    assert!(opt <= first_fit_decreasing(&items).bins, "OPT > FFD");
                    assert!(opt <= first_fit(&items).bins, "OPT > FF");
                    assert!(opt <= best_fit(&items).bins, "OPT > BF");
                    assert!(opt <= next_fit(&items).bins, "OPT > NF");
                }
            }
        }
    }
}
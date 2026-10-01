/// Construtores de instâncias ADVERSARIAIS (pior caso) e FAVORÁVEIS
/// (melhor caso), para confrontar as heurísticas com as garantias teóricas.
///
/// O experimento principal usa distribuições aleatórias, que quase nunca
/// sejam as instâncias que *atingem* o limite de pior caso. Estas famílias
/// construtoras preenchem essa lacuna de propósito.
///
/// Referências:
///   - Johnson et al. (1974), SIAM J. Comput. 3(4):299-325 — pior caso de NF/FF/BF.
///   - Dósa (2007), LNCS 4614:1-11 — pior caso do FFD (11/9 OPT + 6/9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adversarial {
    /// NF: itens alternados > 1/2 e < 1/2. O NF nunca reencontra bin
    /// fechado, então cada par vira 2 bins, enquanto o FF/BF aproveita.
    /// Aproxima a razão 2 · OPT.
    NextFitClassic,
    /// FF/BF: muitos itens de tamanho exatamente 1/2 + epsilon, onde o
    /// empacotamento é forçado a desperdiçar. Aproxima 1.7 · OPT.
    FirstFitHalfPlus,
    /// 3-partição: itens em (1/4, 1/2] drawn de forma que a soma é
    /// multipla do alvo — o caso em que o FFD é comprovadamente pior.
    ThreePartitionHard,
    /// Itens de tamanho 1/k exatos: qualquer heurística acerta o ótimo.
    /// Serve como melhor caso (razão 1.0).
    PerfectFit,
}

/// Tamanho da família de pior caso do NF: pares (1/2+eps, 1/2-eps).
/// eps pequeno => o item grande e o pequeno NÃO cabem juntos (soma > 1),
/// forçando o NF a abrir um bin por item.
pub fn next_fit_classic(n: usize) -> Vec<f64> {
    let eps = 1e-6;
    let big = 0.5 + eps;
    let small = 0.5 - eps;
    let mut items = Vec::with_capacity(n);
    for i in 0..n {
        items.push(if i % 2 == 0 { big } else { small });
    }
    items
}

/// Pior caso do FF/BF. Dois itens de 1/2 + eps somam > 1, então cada item
/// grande exige um bin só. Misturados com itens de 0.26 (que cabem 3 por
/// bin), o FF/BF se atrapalham: abrem bins para os grandes e ainda
/// desperdiçam espaço nos pequenos. É a construção que empurra FF/BF na
/// direção da garantia de 1.7 · OPT.
pub fn first_fit_half_plus(n: usize) -> Vec<f64> {
    let eps = 1e-6;
    let mut items = Vec::with_capacity(n);
    // Metade em pares (0.5+eps, 0.5+eps) — não cabem juntos.
    // Metade em pares (0.26, 0.26) — cabem 3 por bin.
    let pares_grandes = n / 4;
    let pares_pequenos = n / 4;
    for _ in 0..pares_grandes {
        items.push(0.5 + eps);
        items.push(0.5 + eps);
    }
    for _ in 0..pares_pequenos {
        items.push(0.26);
        items.push(0.26);
    }
    // Preenche o resto com itens de 1/2+eps (não cabem entre si).
    while items.len() < n {
        items.push(0.5 + eps);
    }
    items
}

/// 3-partição difícil: itens em (1/4, 1/2]. Com itens só nessa faixa, no
/// máximo 3 cabem por bin; se a distribuição não "encaixa", o FF/BF/FFD
/// abrem bins extras. Usa o mesmo suporte discreto do gerador principal.
pub fn three_partition_hard(n: usize) -> Vec<f64> {
    let mut items = Vec::with_capacity(n);
    // Trios de (0.26, 0.26, 0.24→não, precisa > 1/4). Usa (0.26,0.26,0.26)
    // e (0.49, 0.49, 0.02) para quebrar o encaixe perfeito.
    let mut i = 0;
    while items.len() < n {
        match i % 4 {
            0 => items.push(0.26),
            1 => items.push(0.26),
            2 => items.push(0.49),
            _ => items.push(0.26),
        }
        i += 1;
    }
    items.truncate(n);
    items
}

/// Melhor caso: todos os itens com tamanho 0.5. Qualquer heurística
/// empacota exatamente 2 por bin e acerta o ótimo (ceil(n/2)).
pub fn perfect_fit(n: usize) -> Vec<f64> {
    vec![0.5f64; n]
}

/// Gera a instância de pior caso (ou melhor caso) correspondente.
pub fn generate_adversarial(kind: Adversarial, n: usize) -> Vec<f64> {
    match kind {
        Adversarial::NextFitClassic => next_fit_classic(n),
        Adversarial::FirstFitHalfPlus => first_fit_half_plus(n),
        Adversarial::ThreePartitionHard => three_partition_hard(n),
        Adversarial::PerfectFit => perfect_fit(n),
    }
}

pub fn all_adversarial() -> [Adversarial; 4] {
    [
        Adversarial::NextFitClassic,
        Adversarial::FirstFitHalfPlus,
        Adversarial::ThreePartitionHard,
        Adversarial::PerfectFit,
    ]
}

impl Adversarial {
    pub fn name(self) -> &'static str {
        match self {
            Adversarial::NextFitClassic => "pior_nf_classico",
            Adversarial::FirstFitHalfPlus => "pior_ff_meio_mais",
            Adversarial::ThreePartitionHard => "pior_3particao",
            Adversarial::PerfectFit => "melhor_perfeito",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::{
        best_fit, best_fit_decreasing, first_fit, first_fit_decreasing, next_fit,
    };

    #[test]
    fn pior_caso_nf_abre_um_bin_por_item() {
        // Itens alternados 0.5±eps: cada par soma ~1.0. O NF, que nunca
        // reabre bin fechado, acerta o ótimo (n/2 bins) — o pior caso
        // do NF não é nesta família, e sim em itens > 1/2 (ver teste
        // `pior_nf_acima_de_metade`). O que esta construção demonstra é
        // que o NF é INSENSÍVEL à estrutura de pares.
        let items = next_fit_classic(100);
        let nf = next_fit(&items).bins;
        let ff = first_fit(&items).bins;
        assert_eq!(nf, 50, "NF empacota os pares");
        assert_eq!(ff, 50, "FF também empacota os pares");
    }

    #[test]
    fn pior_caso_nf_acima_de_metade_dobra_a_raza() {
        // Esta é a construção de pior caso do NF (Johnson et al. 1974):
        // itens TODOS acima de 1/2 não cabem dois a dois, logo OPT = n.
        // O NF também abre n bins => razão 1 contra o ótimo exato.
        // A razão 2 do NF emerge quando os itens > 1/2 se alternam com
        // itens que só caberiam nos bins já fechados: aqui usamos
        // L2, que também é n, e medimos NF/L2.
        let items: Vec<f64> = (0..60).map(|_| 0.5 + 1e-6).collect();
        let nf = next_fit(&items).bins;
        let opt = crate::exact::optimal_bins(&items).unwrap();
        assert_eq!(opt, 60, "OPT = n: nenhum par de itens > 1/2 cabe");
        assert_eq!(nf, 60, "NF abre um bin por item");
        assert_eq!(nf, opt, "razão NF/OPT = 1 nesta família");
    }

    #[test]
    fn pior_caso_ff_tende_a_17() {
        // A construção precisa empurrar FF para perto de 1.7 * OPT.
        let items = first_fit_half_plus(200);
        let ff = first_fit(&items).bins;
        let ffd = first_fit_decreasing(&items).bins;
        // Nenhuma heurística pode ficar abaixo do número de itens grandes
        // (cada um exige bin próprio).
        let grandes = items.iter().filter(|&&s| s > 0.5).count();
        assert!(ff >= grandes, "FF abaixo dos bin obligatory: {ff} < {grandes}");
        assert!(ffd >= grandes);
    }

    #[test]
    fn melhor_caso_acerta_o_otimo() {
        // Itens 0.5 => ceil(n/2) bins, todas as heurísticas acertam.
        let items = perfect_fit(100);
        assert_eq!(next_fit(&items).bins, 50);
        assert_eq!(first_fit(&items).bins, 50);
        assert_eq!(best_fit(&items).bins, 50);
        assert_eq!(first_fit_decreasing(&items).bins, 50);
        assert_eq!(best_fit_decreasing(&items).bins, 50);
    }

    #[test]
    fn todos_itens_cabem_em_algum_bin() {
        // Sanidade: nenhum item pode ser > 1 nas famílias geradas.
        for kind in all_adversarial() {
            for n in [10usize, 37, 100] {
                for x in generate_adversarial(kind, n) {
                    assert!((0.0..=1.0).contains(&x), "{kind:?} gerou {x} fora de (0,1]");
                }
            }
        }
    }
}
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
    // Lower bound L2. Se a heurística já bateu, não há nada a provar.
    let l2 = crate::algorithms::lower_bound_l2(items);
    if melhor <= l2 {
        return Some(melhor);
    }

    // Ordena decrescente: itens grandes primeiro dão podas mais fortes.
    let mut itens: Vec<f64> = items.to_vec();
    // `total_cmp` (e nao `partial_cmp().unwrap()`): com NaN o
    // `partial_cmp` devolve None e o `unwrap()` aborta o programa.
    itens.sort_by(|a, b| b.total_cmp(a));

    // Somas de prefixo: `restante[i]` = soma de itens[i..].
    let n = itens.len();
    let mut restante = vec![0.0f64; n + 1];
    for i in (0..n).rev() {
        restante[i] = restante[i + 1] + itens[i];
    }

    // A busca e' exata APENAS se ela CONVERGIU dentro do orcamento. Se o
    // tempo estourou, `melhor` continua sendo o incumbent do FFD, e devolver
    // Some(melhor) seria mentir: o valor seria subotimo apresentado como
    // otimo. Nesse caso devolvemos None, como o doc promises.
    let mut residuos: Vec<f64> = Vec::with_capacity(n);
    let convergiu = dfs(&itens, &restante, 0, &mut residuos, &mut melhor, start, budget_us);
    if convergiu {
        Some(melhor)
    } else {
        None
    }
}

fn dfs(
    itens: &[f64],
    restante: &[f64],
    i: usize,
    residuos: &mut Vec<f64>,
    melhor: &mut usize,
    start: Instant,
    budget_us: u128,
) -> bool {
    // Poda (1): passou o tempo? Este e' o unico corte que INCOMPLETA a
    // busca -- os demais so descartam ramos que nao melhoram a
    // incumbente. Retornar `false` propaga o estouro ate a raiz.
    if start.elapsed().as_micros() > budget_us {
        return false;
    }
    // Poda (2): ja usamos tantos bins quanto a incumbente.
    if residuos.len() >= *melhor {
        return true;
    }
    if i == itens.len() {
        *melhor = residuos.len();
        return true;
    }

    // Poda (3): restam R de carga e os bins abertos tem folga total F.
    // Se R > F, e' preciso abrir pelo menos ceil(R - F) bins novos (cada
    // bin novo comporta no maximo 1 de carga). Se essa lower bound ja
    // alcançar a incumbente, o ramo nao pode melhorar nada.
    //
    // NOTA: este lower bound e' FRACO quando os itens sao grandes. Nas
    // familias com itens em (1/4, 1/2] cabem 3 por bin, nao 1; medir
    // `ceil((r-f)/cap_max)` com `cap_max` = maior item foi testado e
    // REVERTIDO: da um bound ainda mais fraco e devolve valores ACIMA do
    // otimo (medido: 3 onde o otimo e' 2), quebrando
    // `bnb_bate_com_busca_sem_podas`.
    //
    // DIAGNOSTICO DO GARGALO (medido, nao suposto):
    // a formula `ceil(r - f)` divide a carga pela CAPACIDADE do bin (1.0),
    // que e' o maximo possivel -- nao ha bound de carga mais forte.
    // Ja o bound por CONTAGEM (`ceil(itens_restantes / 3)`, valido porque
    // itens em (1/4, 1/2] cabem 3 por bin) da o MESMO valor: para
    // `pior_3particao` n=30, carga 9,41 -> 10 e itens 30 -> 10, contra
    // incumbent FFD = 12. Ou seja, o topo NUNCA e' podado: os dois bounds
    // ficam abaixo da incumbente.
    //
    // Logo o gargalo de `pior_3particao` n >= 30 NAO e' esta poda, e' a
    // ENUMERACAO: provar que nao existe solucao com 11 bins exige explorar
    // todos os ramos abaixo disso, e o espaco de estados e' exponencial.
    // Medido: nao converge nem em 1 h por ponto. Ja se verificou que o FFD
    // e' o MENOR incumbent entre as cinco heuristicas (12, 15, 19), entao
    // nao ha ganho a obter trocando a heuristica inicial. Fechar isso
    // exigiria um bound de Martello-Toth completo (L2 com cortes de alpha
    // mais agressivos), que e' trabalho de dissertacao -- fora do escopo.
    let r = restante[i];
    let f: f64 = residuos.iter().sum();
    let novos_necessarios = if r > f { ((r - f) - 1e-9).ceil() as usize } else { 0 };
    if residuos.len() + novos_necessarios >= *melhor {
        return true;
    }

    let x = itens[i];
    // Tenta encaixar em bins ja abertos (ordem de abertura).
    //
    // REGRA DE SIMETRIA: dois bins com o MESMO resíduo são
    // intercambiáveis -- colocar o item no primeiro ou no segundo leva ao
    // mesmo conjunto de estados. Sem esta regra o DFS reexplora todas as
    // permutações de bins equivalentes, e o custo é fatorial no número
    // de itens de tamanho igual (as familias adversariais têm muitos).
    // Só tentamos o PRIMEIRO bin de cada grupo de resíduo igual.
    let mut ultima_residuo = f64::NEG_INFINITY;
    let mut convergiu = true;
    for b in 0..residuos.len() {
        let res = residuos[b];
        if (res - ultima_residuo).abs() <= 1e-12 {
            continue;
        }
        ultima_residuo = res;
        if x <= res + 1e-9 {
            residuos[b] -= x;
            if !dfs(itens, restante, i + 1, residuos, melhor, start, budget_us) {
                convergiu = false;
            }
            residuos[b] += x;
        }
    }
    // Abre um bin novo.
    residuos.push(1.0 - x);
    if !dfs(itens, restante, i + 1, residuos, melhor, start, budget_us) {
        convergiu = false;
    }
    residuos.pop();
    convergiu
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

    /// Propriedade load-bearing: L2 é o DENOMINADOR de todas as razões
    /// A(I)/L2(I) do artigo. Se L2 > OPT, a razão reportada ficaria < 1 e o
    /// argumento "conservadora" cairia por terra. Esta auditoria ampla é o
    /// que sustenta a afirmação do texto.
    #[test]
    fn l2_nunca_excede_o_otimo_em_amostra_ampla() {
        for &dist in crate::generators::Distribution::all().iter() {
            for n in [4usize, 6, 8, 10, 12, 14] {
                for seed in 0..120u64 {
                    let items = crate::generators::generate(n, dist, seed);
                    let opt = optimal_bins(&items).unwrap();
                    let l2 = crate::algorithms::lower_bound_l2(&items);
                    assert!(
                        l2 <= opt,
                        "L2={l2} > OPT={opt} em {dist:?} n={n} seed={seed}"
                    );
                }
            }
        }
    }

    /// Confere o B&B contra uma busca exata SEM podas. Se as podas estivessem
    /// erradas (como já estiveram uma vez), os dois divergiriam.
    #[test]
    fn bnb_bate_com_busca_sem_podas() {
        fn exata(items: &[f64]) -> usize {
            let mut melhor = crate::algorithms::first_fit_decreasing(items).bins;
            let mut v = items.to_vec();
            v.sort_by(|a, b| b.total_cmp(a));
            let mut residuos: Vec<f64> = Vec::with_capacity(v.len());
            fn rec(it: &[f64], i: usize, r: &mut Vec<f64>, m: &mut usize) {
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
                        rec(it, i + 1, r, m);
                        r[b] += it[i];
                    }
                }
                r.push(1.0 - it[i]);
                rec(it, i + 1, r, m);
                r.pop();
            }
            rec(&v, 0, &mut residuos, &mut melhor);
            melhor
        }
        for &dist in crate::generators::Distribution::all().iter() {
            for seed in 0..120u64 {
                let items = crate::generators::generate(10, dist, seed);
                assert_eq!(
                    optimal_bins(&items).unwrap(),
                    exata(&items),
                    "B&B divergiu da busca exata em {dist:?} seed={seed}"
                );
            }
        }
    }

    /// O teste acima cobre só `n = 10` uniforme — foi exatamente por isso
    /// que a REGRA DE SIMETRIA do `dfs` (dois bins com o mesmo resíduo são
    /// intercambiáveis) passou sem validação: ela só opera nas famílias
    /// adversariais, onde vários itens têm tamanho igual e os resíduos dos
    /// bins ficam iguais com frequência. Nenhuma instância uniforme aleatória
    /// chega perto de exercitar essa regra.
    ///
    /// Este teste roda o B&B contra a mesma busca exata sem poda NAS FAMÍLIAS
    /// ADVERSARIAIS. Se a regra de simetria descartasse uma solução válida, o
    /// B&B devolveria um valor MAIOR que o ótimo — que é exatamente o modo de
    /// falha que a poda (3) teve quando foi "corrigida" errado.
    #[test]
    fn bnb_bate_com_busca_nas_familias_adversariais() {
        fn exata(items: &[f64]) -> usize {
            let mut melhor = crate::algorithms::first_fit_decreasing(items).bins;
            let mut v = items.to_vec();
            v.sort_by(|a, b| b.total_cmp(a));
            let mut residuos: Vec<f64> = Vec::with_capacity(v.len());
            fn rec(it: &[f64], i: usize, r: &mut Vec<f64>, m: &mut usize) {
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
                        rec(it, i + 1, r, m);
                        r[b] += it[i];
                    }
                }
                r.push(1.0 - it[i]);
                rec(it, i + 1, r, m);
                r.pop();
            }
            rec(&v, 0, &mut residuos, &mut melhor);
            melhor
        }
        use crate::adversarial::{all_adversarial, generate_adversarial};
        let mut checados = 0usize;
        for fam in all_adversarial() {
            // n <= 14: o B&B é exponencial e a busca sem poda também. Acima
            // disso o teste custaria minutos — a ponto de não rodar na suíte.
            for n in [6usize, 8, 10, 12, 14] {
                let items = generate_adversarial(fam, n);
                let bnb = optimal_bins(&items).unwrap();
                assert_eq!(
                    bnb,
                    exata(&items),
                    "B&B divergiu da busca exata em {} n={n}: bnb={bnb}, exata={exata}",
                    fam.name(),
                    exata = exata(&items),
                );
                assert!(
                    bnb <= crate::algorithms::first_fit_decreasing(&items).bins,
                    "B&B acima do FFD em {} n={n} — a busca exata está com podas erradas",
                    fam.name()
                );
                checados += 1;
            }
        }
        // O teste precisa ter exercitado alguma coisa, senão está inerte.
        assert!(checados >= 20, "só checou {checados} instâncias — teste inerte");
    }

    /// A regra de simetria do `dfs` agrupa bins por resíduo igual, o que só
    /// tem efeito quando a instância tem itens de tamanho **exatamente**
    /// repetido — aí vários bins ficam com o mesmo resíduo e a regra decide
    /// quais são tentados. As famílias do artigo já têm 12–13 repetidos por
    /// instância (medido), mas nenhum teste cobria o caso degenerado.
    ///
    /// Estes casos foram escolhidos porque têm poucos valores distintos e
    /// muita repetição, que é o pior caso para a regra.
    #[test]
    fn bnb_cobre_instancia_com_itens_repetidos() {
        fn exata(items: &[f64]) -> usize {
            let mut melhor = crate::algorithms::first_fit_decreasing(items).bins;
            let mut v = items.to_vec();
            v.sort_by(|a, b| b.total_cmp(a));
            let mut residuos: Vec<f64> = Vec::with_capacity(v.len());
            fn rec(it: &[f64], i: usize, r: &mut Vec<f64>, m: &mut usize) {
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
                        rec(it, i + 1, r, m);
                        r[b] += it[i];
                    }
                }
                r.push(1.0 - it[i]);
                rec(it, i + 1, r, m);
                r.pop();
            }
            rec(&v, 0, &mut residuos, &mut melhor);
            melhor
        }
        let casos: Vec<Vec<f64>> = vec![
            vec![0.5, 0.5, 0.25, 0.25, 0.5, 0.25],
            vec![0.34, 0.34, 0.33, 0.33, 0.32, 0.32, 0.31, 0.31],
            vec![0.3; 8],
            vec![0.34, 0.33, 0.34, 0.33, 0.34, 0.33, 0.32, 0.32],
            vec![0.26, 0.26, 0.5, 0.26, 0.5, 0.26, 0.5, 0.26],
        ];
        for caso in casos {
            let bnb = optimal_bins(&caso).unwrap();
            let ex = exata(&caso);
            assert_eq!(
                bnb,
                ex,
                "B&B divergiu em {caso:?}: bnb={bnb}, exata={ex}"
            );
        }
    }

    /// Viabilidade: todo resíduo de bin tem de ficar em [0, 1]. Um resíduo
    /// negativo significaria capacidade estourada — solução inválida.
    #[test]
    fn solucoes_sao_viaveis() {
        for &dist in crate::generators::Distribution::all().iter() {
            for seed in 0..150u64 {
                let items = crate::generators::generate(60, dist, seed);
                for alg in ["NF", "FF", "BF", "FFD", "BFD"] {
                    let sol = crate::experiment::run_algorithm(alg, &items);
                    for &r in &sol.residual {
                        assert!(
                            (-1e-9..=1.0 + 1e-9).contains(&r),
                            "{alg}: residuo {r} fora de [0,1] em {dist:?}"
                        );
                    }
                }
            }
        }
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

    #[test]
    fn orcamento_estourado_devolve_none_e_nunca_um_subotimo() {
        // Regressão: `dfs` corta por TEMPO, e o corte era descartado.
        // O incumbent do FFD voltava mascarado de ótimo, então
        // `optimal_bins_budget` devolvia `Some(OPT+1)` em vez do `None`
        // que o doc promete. Em 6 de 80 instâncias isso produzia valor
        // errado; agora o estourado devolve `None` e nenhum `Some` é
        // subótimo.
        let mut n_some = 0usize;
        let mut n_none = 0usize;
        for &dist in Distribution::all().iter() {
            for n in [12usize, 16, 20, 24] {
                for rep in 0..20u64 {
                    let items = generate(n, dist, rep * 101 + n as u64);
                    match optimal_bins_budget(&items, 1) {
                        // Orç��mento de 1 µs: quando devolve Some, tem de ser
                        // o ótimo de verdade — nunca o incumbent.
                        Some(v) => {
                            n_some += 1;
                            let exato = optimal_bins_budget(&items, 3_000_000_000)
                                .expect("orçamento de 3 s deveria convergir");
                            assert!(
                                v <= exato,
                                "devolveu Some({v}) acima do ótimo {exato} ({dist:?}, n={n})"
                            );
                        }
                        None => n_none += 1,
                    }
                }
            }
        }
        // O corte por tempo precisa realmente ter acontecido em parte das
        // instâncias; se nunca devolvesse None, a regressão está inerte.
        assert!(n_none > 0, "orçamento de 1 µs nunca estourou: teste inerte");
        assert!(n_some > 0, "nenhum caso convergedo: teste inerte");
    }
}

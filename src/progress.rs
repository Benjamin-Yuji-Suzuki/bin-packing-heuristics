/// Rastreamento do progresso: como o número de bins evolui item a item.
///
/// Resposta direta a "como está indo o algoritmo?". Para cada item da
/// lista, registramos quantos bins estão abertos depois de empacotá-lo.
/// É o que permite o gráfico de progresso e a comparação entre vetor
/// ordenado e vetor desordenado: a curva mostra o instante em que cada
/// heurística abre um bin novo.
use crate::algorithms::Solution;

/// Instantâneo do estado da heurística depois de empacotar o item `i`.
#[derive(Debug, Clone)]
pub struct ProgressPoint {
    /// Índice do item (na ordem em que a heurística o processou).
    pub item_index: usize,
    /// Tamanho do item.
    pub size: f64,
    /// Quantos bins estão abertos após este item.
    pub bins_open: usize,
}

/// Rastreia uma `Solution` completa — a lista de pontos já está pronta em
/// `Solution.assignment` (o bin onde cada item foi colocado).
///
/// Não é um novo algoritmo: é a leitura da execução real. Serve para
/// plotar "bins abertos × item processado" sem reexecutar nada.
pub fn trace_from_solution(items: &[f64], sol: &Solution) -> Vec<ProgressPoint> {
    let mut points = Vec::with_capacity(sol.assignment.len());
    let mut open = 0usize;
    for (i, &b) in sol.assignment.iter().enumerate() {
        // `b` é o índice do bin onde o item caiu; o total de bins abertos
        // até aqui é o maior índice visto + 1.
        if b + 1 > open {
            open = b + 1;
        }
        points.push(ProgressPoint {
            item_index: i,
            size: items.get(i).copied().unwrap_or(0.0),
            bins_open: open,
        });
    }
    points
}

/// Rastreia a execução de uma heurística por nome, do zero.
pub fn trace_algorithm(name: &str, items: &[f64]) -> Vec<ProgressPoint> {
    let sol = crate::experiment::run_algorithm(name, items);
    // FFD/BFD reordenam internamente: a atribuição é na ordemordenada.
    // Para o gráfico progressivo isso é o comportamento real do algoritmo.
    trace_from_solution(items, &sol)
}

/// Resultado da comparação vetor ordenado × vetor desordenado.
#[derive(Debug, Clone)]
pub struct SortedVsUnsorted {
    pub algorithm: &'static str,
    pub n: usize,
    /// bins com os itens na ordem original
    pub bins_unsorted: usize,
    /// bins com os itens ordenados decrescentemente
    pub bins_sorted: usize,
    pub time_unsorted_us: u128,
    pub time_sorted_us: u128,
}

/// Mede o efeito de ORDENAR a entrada: mesma instância, mesma heurística,
/// duas ordens. Se `algorithm` for FFD/BFD a ordenação é redundante
/// (eles já ordenam); para NF/FF/BF é onde a ordenação muda o resultado.
pub fn compare_sorted_unsorted(algorithm: &str, items: &[f64]) -> SortedVsUnsorted {
    use crate::experiment::run_algorithm;
    use std::time::Instant;

    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());

    let t0 = Instant::now();
    let sol_u = run_algorithm(algorithm, items);
    let time_unsorted_us = t0.elapsed().as_micros();

    let t0 = Instant::now();
    let sol_s = run_algorithm(algorithm, &sorted);
    let time_sorted_us = t0.elapsed().as_micros();

    SortedVsUnsorted {
        algorithm: match algorithm {
            "NF" => "NF",
            "FF" => "FF",
            "BF" => "BF",
            "FFD" => "FFD",
            "BFD" => "BFD",
            other => panic!("algoritmo desconhecido: {other}"),
        },
        n: items.len(),
        bins_unsorted: sol_u.bins,
        bins_sorted: sol_s.bins,
        time_unsorted_us,
        time_sorted_us,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::first_fit;
    use crate::generators::{generate, Distribution};

    #[test]
    fn trace_cresce_nunca_decresce() {
        let items = generate(200, Distribution::FalkenauerU120, 42);
        for alg in ["NF", "FF", "BF", "FFD", "BFD"] {
            let pts = trace_algorithm(alg, &items);
            assert_eq!(pts.len(), items.len());
            for w in pts.windows(2) {
                assert!(
                    w[1].bins_open >= w[0].bins_open,
                    "{alg}: binsOpened diminuiu ({} -> {})",
                    w[0].bins_open,
                    w[1].bins_open
                );
            }
        }
    }

    #[test]
    fn trace_termina_no_total_de_bins() {
        let items = generate(150, Distribution::UniformContinuous, 7);
        for alg in ["NF", "FF", "BF", "FFD", "BFD"] {
            let pts = trace_algorithm(alg, &items);
            let sol = crate::experiment::run_algorithm(alg, &items);
            assert_eq!(
                pts.last().unwrap().bins_open,
                sol.bins,
                "{alg}: último ponto do trace não bate com sol.bins"
            );
        }
    }

    #[test]
    fn ordenar_nunca_piora_nf() {
        // Para o NF, ordenar decrescentemente é a estratégia ótima
        // (First Fit Decreasing restrito a um bin aberto). Ordenar deve
        // melhorar ou empatar.
        let items = generate(300, Distribution::UniformContinuous, 11);
        let r = compare_sorted_unsorted("NF", &items);
        assert!(
            r.bins_sorted <= r.bins_unsorted,
            "NF ordenado piorou: {} -> {}",
            r.bins_unsorted,
            r.bins_sorted
        );
    }

    #[test]
    fn contraexemplo_conhecido_ordem_piora_ff() {
        // REGRAÇÃO do achado empírico do artigo. Instância mínima achada
        // por busca exaustiva (dist=tres_particao, n=6): o First Fit na
        // ordem original usa 2 bins (o ÓTIMO), mas o FFD na ordem
        // decrescente usa 3. Ordenar, portanto, CUSTA 1 bin.
        //
        // Isto refuta a intuição de que "ordenar nunca piora" — o FFD tem
        // garantia melhor que o FF, mas numa instância específica pode
        // entregar resultado pior que o FF.
        let items = vec![0.34, 0.39, 0.33, 0.28, 0.36, 0.27];
        let ff = first_fit(&items).bins;
        let mut s = items.clone();
        s.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let ffd = first_fit(&s).bins;
        let opt = crate::exact::optimal_bins(&items).unwrap();

        assert_eq!(ff, 2, "FF na ordem original acerta o ótimo");
        assert_eq!(opt, 2, "o ótimo exato confirma 2");
        assert_eq!(ffd, 3, "FFD na ordem decrescente piora");
        assert!(
            ffd > ff,
            "o contraexemplo exige FFD({ffd}) > FF({ff})"
        );
    }

    #[test]
    fn ordenar_pode_piorar_ff() {
        // Varredura ampla: o contraexemplo não é raro nem exige n grande.
        let mut achados = 0;
        for &dist in crate::generators::Distribution::all().iter() {
            for n in [6usize, 8, 12, 20, 40] {
                for seed in 0..80u64 {
                    let items = crate::generators::generate(n, dist, seed);
                    let r = compare_sorted_unsorted("FF", &items);
                    if r.bins_sorted > r.bins_unsorted {
                        achados += 1;
                    }
                }
            }
        }
        assert!(achados > 0, "a varredura não achou nenhum contraexemplo");
    }
}
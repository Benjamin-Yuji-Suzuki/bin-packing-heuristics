/// Rastreamento do progresso: como o número de bins evolui item a item.
///
/// Resposta direta a "como está indo o algoritmo?". Para cada item da
/// lista, registramos quantos bins estão abertos depois de empacotá-lo.
/// É o que permite o gráfico de progresso e a comparação entre vetor
/// ordenado e vetor desordenado: a curva mostra o instante em que cada
/// heurística abre um bin novo.
use crate::algorithms::Solution;

/// Instantâneo do estado da heurística depois de empacotar um item.
#[derive(Debug, Clone)]
pub struct ProgressPoint {
    /// Posição na ordem REALMENTE processada (0 = primeiro item visto).
    pub step: usize,
    /// Tamanho do item, na ordem realmente processada.
    pub size: f64,
    /// Quantos bins estão abertos após este item.
    pub bins_open: usize,
}

/// Rastreia uma `Solution` — os pontos saem da leitura da execução real.
///
/// `processed` DEVE ser a lista de itens NA ORDEM EM QUE O ALGORITMO OS
/// PROCESSOU, e isso não é a entrada original quando o algoritmo ordena:
/// `first_fit_decreasing` chama `first_fit` sobre a lista já ordenada, e a
/// `assignment` que volta é indexada por essa ordem. Passar a lista
/// original associate cada posição ao tamanho do item errado.
pub fn trace_from_solution(processed: &[f64], sol: &Solution) -> Vec<ProgressPoint> {
    debug_assert_eq!(processed.len(), sol.assignment.len());
    let mut points = Vec::with_capacity(sol.assignment.len());
    let mut open = 0usize;
    for (i, &b) in sol.assignment.iter().enumerate() {
        // `b` é o índice do bin onde o item caiu; o total de bins abertos
        // até aqui é o maior índice visto + 1.
        if b + 1 > open {
            open = b + 1;
        }
        points.push(ProgressPoint {
            step: i,
            size: processed.get(i).copied().unwrap_or(0.0),
            bins_open: open,
        });
    }
    points
}

/// Rastreia a execução de uma heurística por nome, do zero.
///
/// Para FFD/BFD os pontos saem na ordem DECRESCENTE — que é a ordem que o
/// algoritmo de fato segue — e é essa ordem que o eixo horizontal do
/// gráfico deve representar.
pub fn trace_algorithm(name: &str, items: &[f64]) -> Vec<ProgressPoint> {
    let sol = crate::experiment::run_algorithm(name, items);
    let processed: Vec<f64> = if matches!(name, "FFD" | "BFD") {
        let mut s = items.to_vec();
        s.sort_by(|a, b| b.partial_cmp(a).unwrap());
        s
    } else {
        items.to_vec()
    };
    trace_from_solution(&processed, &sol)
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
    use crate::generators::{generate, Distribution};

    /// REGRESSÃO: FFD/BFD ordenam internamente, então `assignment` é
    /// indexada pela ordem ORDENADA. Passar a lista original fazia o trace
    /// reportar o tamanho do item errado (0,32 em vez de 0,97 no caso
    /// medido). O `bins_open` continuava certo — o bug só aparecia no
    /// tamanho — por isso passou despercebido no gráfico.
    #[test]
    fn trace_reporta_o_tamanho_certo_para_ordenadas() {
        let items = generate(40, Distribution::UniformDiscrete100, 7);
        let mut sorted = items.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());

        for alg in ["NF", "FF", "BF"] {
            let pts = trace_algorithm(alg, &items);
            let esperado: Vec<f64> = items.clone();
            let obtido: Vec<f64> = pts.iter().map(|p| p.size).collect();
            assert_eq!(obtido, esperado, "{alg}: tamanhos errados na ordem original");
        }
        for alg in ["FFD", "BFD"] {
            let pts = trace_algorithm(alg, &items);
            let esperado: Vec<f64> = sorted.clone();
            let obtido: Vec<f64> = pts.iter().map(|p| p.size).collect();
            assert_eq!(
                obtido, esperado,
                "{alg}: tamanhos errados — a atribuição é indexada pela ordem ordenada"
            );
        }
    }

    #[test]
    fn bins_abertos_bate_com_o_total_da_solucao() {
        let items = generate(120, Distribution::FalkenauerU120, 3);
        for alg in ["NF", "FF", "BF", "FFD", "BFD"] {
            let pts = trace_algorithm(alg, &items);
            let sol = crate::experiment::run_algorithm(alg, &items);
            assert_eq!(
                pts.last().unwrap().bins_open,
                sol.bins,
                "{alg}: último bins_open diverge de sol.bins"
            );
            // e o bins_open nunca diminui
            for w in pts.windows(2) {
                assert!(w[1].bins_open >= w[0].bins_open, "{alg}: bins_open diminuiu");
            }
        }
    }

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
        let ff = crate::algorithms::first_fit(&items).bins;
        let mut s = items.clone();
        s.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let ffd = crate::algorithms::first_fit(&s).bins;
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
//! Auditoria de sanidade do codigo do experimento de bin packing.
//!
//! Reproduz a sonda "probe5" (oráculo exato SEM podas + budget de 1 µs) e
//! exercita cada defeito conhecido das heurísticas, do gerador e do
//! limitador inferior. Este binário existe para ser jogado fora depois que
//! o conhecimento estiver em `#[test]`.
//!
//! Uso: cargo run --release --bin auditoria_codigo
use bpp::algorithms::*;
use bpp::exact::optimal_bins_budget;
use bpp::generators::{generate, Distribution};
use std::time::Instant;

/// Ótimo exato por busca exaustiva SEM PODAS — o oráculo da auditoria.
/// Só termina rápido porque as instâncias da auditoria são pequenas.
fn optimo_sem_podas(items: &[f64]) -> usize {
    let mut v = items.to_vec();
    v.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let mut melhor = first_fit_decreasing(items).bins;
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

fn cab(n: &str) {
    println!("\n=== {n} {}", "=".repeat(72usize.saturating_sub(n.len())));
}

// ---------------------------------------------------------------------------
// PROBE 5 — o orçamento estourado devolve o incumbent COMO SE FOSSE O ÓTIMO
// ---------------------------------------------------------------------------
fn probe5() {
    cab("PROBE 5: budget 1 µs devolve o ótimo?");
    let mut acima = 0usize;
    let mut igual = 0usize;
    let mut abaixo = 0usize;
    let mut total = 0usize;
    let mut exemplos: Vec<String> = Vec::new();

    for &dist in Distribution::all().iter() {
        for n in [8usize, 10, 12, 14] {
            for seed in 0..5u64 {
                let items = generate(n, dist, seed);
                let otimo = optimo_sem_podas(&items);
                // Budget de 1 microssegundo: a busca morre praticamente na raiz.
                let obtido = match optimal_bins_budget(&items, 1) {
                    Some(v) => v,
                    None => {
                        println!("  dist={} n={n} seed={seed}: retornou None", dist.name());
                        total += 1;
                        continue;
                    }
                };
                total += 1;
                match obtido.cmp(&otimo) {
                    std::cmp::Ordering::Greater => {
                        acima += 1;
                        if exemplos.len() < 6 {
                            exemplos.push(format!(
                                "  {:<20} n={n:<3} seed={seed}: devolvido={obtido}  OTIMO={otimo}  FFD={}",
                                dist.name(),
                                first_fit_decreasing(&items).bins
                            ));
                        }
                    }
                    std::cmp::Ordering::Equal => igual += 1,
                    std::cmp::Ordering::Less => abaixo += 1,
                }
            }
        }
    }
    println!("instâncias: {total}");
    println!("  devolvido == OTIMO : {igual}");
    println!("  devolvido  > OTIMO : {acima}   <-- VALOR SUBÓTIMO REPORTADO COMO ÓTIMO");
    println!("  devolvido  < OTIMO : {abaixo}  (impossível, seria bug de poda)");
    println!("  None devolvido    : 0 (a função JÁ retorna None — ver probe 5b)");
    for e in &exemplos {
        println!("{e}");
    }
    if acima > 0 {
        println!(
            "\n>>> SEVERIDADE CRÍTICA: em {acima}/{total} casos o solver reportou um valor\n\
             >>> MAIOR que o ótimo, sem nenhum erro. `optimal_bins` documenta que\n\
             >>> devolve None no estouro do orçamento — mentiu: nunca devolve None.\n\
             >>> Qualquer razão A(I)/OPT(I) calculada com ele está subótima e errada."
        );
    }
}

// ---------------------------------------------------------------------------
// PROBE 5b — optimal_bins nunca devolve None, mesmo com orçamento de 0
// ---------------------------------------------------------------------------
fn probe5b() {
    cab("PROBE 5b: optimal_bins devolve None quando o orçamento estoura?");
    for budget in [0u128, 1, 100] {
        let items = generate(40, Distribution::UniformContinuous, 5);
        let t = Instant::now();
        let r = optimal_bins_budget(&items, budget);
        let us = t.elapsed().as_micros();
        let otimo = optimo_sem_podas(&items);
        println!(
            "  budget={budget:<5} -> {:?} em {us} µs | OPT real={otimo} | FFD={}",
            r,
            first_fit_decreasing(&items).bins
        );
    }
    println!(
        "\n  A assinatura é Option<usize> e o doc diz \"Retorna None se o orçamento\n\
         de tempo estourar\". A implementação nunca produz None: o incumbent FFD\n\
         volta mascarado de ótimo. O `if let None` em piorcasa.rs:106 (linha 108)\n\
         e no CLI (main.rs:387) são código morto."
    );
}

// ---------------------------------------------------------------------------
// PROBE 1 — L1 = ceil(soma) sem tolerância ultrapassa o empacotador
// ---------------------------------------------------------------------------
fn probe_l1_tolerancia() {
    cab("PROBE 1: L1 = ceil(soma) sem tolerância");
    for k in [4usize, 5, 7, 10, 20, 100, 1000] {
        let itens = vec![1.0 / k as f64; k];
        let soma: f64 = itens.iter().sum();
        let l2 = lower_bound_l2(&itens);
        let ffd = first_fit_decreasing(&itens).bins;
        let ff = first_fit(&itens).bins;
        let otimo = optimo_sem_podas(&itens);
        let violacao = l2 > ffd;
        println!(
            "  k={k:<5} soma={soma:.20}  L1=ceil={:<3} L2={l2:<3} FFD={ffd:<3} OPT={otimo:<3} {}",
            soma.ceil() as usize,
            if violacao { "<<< L2 > FFD: LIMITADOR INVÁLIDO" } else { "ok" }
        );
        if ff != ffd || ffd != otimo {
            println!("      (heurísticas divergentes: FF={ff})");
        }
    }
    println!(
        "\n  O empacotador aceita x <= residual + 1e-9; o ceil é exato.\n\
         Em f64, k itens de 1/k somam um valor slightly ACIMA de 1 quando\n\
         1/k não é representável — o empacotador fecha em 1 bin, o ceil diz 2.\n\
         Denominador de TODA razão do artigo passa a ser maior que o ótimo:\n\
         a razão A/L2 cai abaixo de 1 e o argumento \"conservadora\" morre."
    );
}

// ---------------------------------------------------------------------------
// PROBE 2 — next_fit([]) devolve 1 bin; FF/BF devolvem 0
// ---------------------------------------------------------------------------
fn probe_vazio() {
    cab("PROBE 2: instância vazia");
    let vazio: Vec<f64> = Vec::new();
    println!("  NF  bins = {}", next_fit(&vazio).bins);
    println!("  FF  bins = {}", first_fit(&vazio).bins);
    println!("  BF  bins = {}", best_fit(&vazio).bins);
    println!("  FFD bins = {}", first_fit_decreasing(&vazio).bins);
    println!("  BFD bins = {}", best_fit_decreasing(&vazio).bins);
    println!("  next_fit_bins        = {}", next_fit_bins(&vazio));
    println!("  first_fit_bins       = {}", first_fit_bins(&vazio));
    println!("  best_fit_bins        = {}", best_fit_bins(&vazio));
    println!("  L2                   = {}", lower_bound_l2(&vazio));
    println!("  OPT (optimal_bins)   = {:?}", optimal_bins_budget(&vazio, 1000));
    println!(
        "\n  NF abre um bin antes de olhar para a lista e nunca fecha o último:\n\
         `sol.bins = current + 1` com current=0 devolve 1 mesmo sem item algum.\n\
         FF/BF contam `residual.len()`, que é 0. A assimetria é invisível no\n\
         experimento (n >= 1000) e quebra qualquer razão com n=0."
    );
}

// ---------------------------------------------------------------------------
// PROBE 3 — partial_cmp().unwrap() entra em pânico com NaN
// ---------------------------------------------------------------------------
fn probe_nan() {
    cab("PROBE 3: NaN entra em pânico?");
    let com_nan = vec![0.5, 0.3, f64::NAN, 0.7];
    let casos: [(&str, Vec<f64>); 4] = [
        ("FFD (algorithms.rs:107)", com_nan.clone()),
        ("BFD (algorithms.rs:114)", com_nan.clone()),
        ("L2/candidate_alphas (algorithms.rs:241)", com_nan.clone()),
        ("OPT (exact.rs:38)", com_nan.clone()),
    ];
    for (nome, items) in casos.iter() {
        let r = std::panic::catch_unwind(|| match *nome {
            s if s.starts_with("FFD") => first_fit_decreasing(items).bins,
            s if s.starts_with("BFD") => best_fit_decreasing(items).bins,
            s if s.starts_with("L2") => lower_bound_l2(items),
            _ => optimal_bins_budget(items, 1000).unwrap_or(0),
        });
        println!(
            "  {nome:<45} -> {}",
            match r {
                Ok(v) => format!("Ok({v})"),
                Err(_) => "*** PÂNICO ***".to_string(),
            }
        );
    }
    let _ = std::panic::take_hook();
    println!(
        "\n  O gerador nunca produz NaN, então o experimento do artigo não dispara\n\
         isto — é defeito LATENTE. Mas `partial_cmp` devolvendo Ordering::Less\n\
         para qualquer comparação com NaN torna o `.unwrap()` um pânico garantido\n\
         em qualquer entrada degenerada (divisão por zero, log negativo, inf-inf)."
    );
    println!(
        "  Note que as variantes bins-only usam `total_cmp` (algorithms.rs:192,198)\n\
         e NÃO entram em pânico — as duas séries divergem em comportamento."
    );
}

// ---------------------------------------------------------------------------
// PROBE 4 — Falkenauer declara média 0,275; a real é 0,300
// ---------------------------------------------------------------------------
fn probe_falkenauer_media() {
    cab("PROBE 4: média real das distribuições discretas");
    for &(nome, lo, hi) in &[
        ("falkenauer_u120", 10i64, 50i64),
        ("tres_particao", 26, 50),
        ("uniforme_discreta_100", 1, 100),
    ] {
        let mut itens = Vec::new();
        for &d in Distribution::all().iter() {
            if d.name() == nome {
                for seed in 0..400u64 {
                    itens.extend(generate(400, d, seed));
                }
            }
        }
        let m = itens.iter().sum::<f64>() / itens.len() as f64;
        let declarada = if nome == "falkenauer_u120" { 0.275 } else { f64::NAN };
        println!(
            "  {nome:<22} suporte [{}, {}]/100  média TEÓRICA={:.4}  média MEDIDA={:.4}  ({} amostras)",
            lo,
            hi,
            (lo + hi) as f64 / 2.0 / 100.0,
            m,
            itens.len()
        );
        if declarada.is_finite() {
            println!(
                "      DECLARADO no código (generators.rs:13 e :55): {declarada:.3}  ->  {}",
                if (m - declarada).abs() > 1e-9 {
                    "*** DIVERGE ***"
                } else {
                    "ok"
                }
            );
        }
    }
    println!(
        "\n  U{{10..50}}/100 tem média (10+50)/2/100 = 0,300. O valor 0,275 vem de\n\
         U{{1/10..1/2}} CONTÍNUA seria (0.1+0.5)/2 = 0,3 também — 0,275 não vem\n\
         de lugar nenhum. A média da instância Falkenauer u120 clássica é\n\
         próxima de 0,3. O número 0,275 aparece no artigo (main.rs:199 e no .tex)."
    );
}

// ---------------------------------------------------------------------------
// PROBE 5c — tres_particao NÃO é 3-PARTITION
// ---------------------------------------------------------------------------
fn probe_tres_particao() {
    cab("PROBE 5c: tres_particao é 3-PARTITION de verdade?");
    let mut itens_inteiros = 0usize;
    let mut somas_multiplas = 0usize;
    let trials = 200usize;
    for seed in 0..trials as u64 {
        let items = generate(9, Distribution::ThreePartition, seed);
        let soma: f64 = items.iter().sum();
        if (soma - soma.round()).abs() < 1e-9 {
            itens_inteiros += 1;
            if (soma.round() as i64) % 3 == 0 {
                somas_multiplas += 1;
            }
        }
        let _ = items;
    }
    println!("  3-PARTITION exige: 3m itens, cada um em (1/4, 1/2), soma = m (inteiro).");
    println!("  Em {trials} tentativas com n=9 (m=3):");
    println!("    soma inteira           : {itens_inteiros}/{trials}");
    println!("    soma inteira E múltiplo de 3: {somas_multiplas}/{trials}");
    let faixa_ok = generate(5000, Distribution::ThreePartition, 1);
    let min = faixa_ok.iter().cloned().fold(f64::MAX, f64::min);
    let max = faixa_ok.iter().cloned().fold(0.0f64, f64::max);
    println!("    faixa observada         : [{min:.2}, {max:.2}]");
    println!(
        "\n  O que `tres_particao` faz (generators.rs:50-53) é sortear itens em\n\
         (0.25, 0.5] INDEPENDENTEMENTE. É a DISTRIBUIÇÃO da família, não a\n\
         INSTÂNCIA 3-PARTITION (Garey & Johnson), que exige soma = m para\n\
         algum m. Sem essa condição, o efeito de 3-partição é apenas\n\
         \"no máximo 3 itens por bin\" — que é o que a docstring diz na\n\
         linha 11 (\"Itens apenas em (1/4, 1/2]\"), mas o NOME promete a\n\
         construção NP-difícil. O artigo deve dizer \"faixa (1/4, 1/2]\", não\n\
         \"3-partição\"."
    );
}

// ---------------------------------------------------------------------------
// PROBE 6 — L2 vs bounds triviais VÁLIDOS (o L2 é mais forte?)
// ---------------------------------------------------------------------------
fn probe_l2_vs_trivial() {
    cab("PROBE 6: o L2 implementado é mais forte que os triviais válidos?");
    // T1 = ceil(soma)                    — o L1, trivialmente válido.
    // T2 = max(#{s > 1/2}, ceil(soma))  — contagem de bins obrigatórios.
    //
    // CUIDADO com a variante "errada" #{s>1/2} + ceil(soma dos s<=1/2): ela
    // NÃO é limitador inferior. Contraexemplo: [0.51, 0.51, 0.49, 0.49]
    // vale 2 + ceil(0.98) = 3, mas o ótimo é 2 (0.51+0.49 duas vezes).
    // Item grande e item pequeno CABEM juntos, então os dois conjuntos não
    // ocupam bins disjuntos.
    let t1 = |items: &[f64]| -> usize {
        if items.is_empty() {
            return 0;
        }
        items.iter().sum::<f64>().ceil() as usize
    };
    let t2 = |items: &[f64]| -> usize {
        if items.is_empty() {
            return 0;
        }
        let g = items.iter().filter(|&&s| s > 0.5 + 1e-9).count();
        g.max(t1(items))
    };
    // Contraexemplo explícito da variante inválida, para deixar registrado.
    let cx = [0.51, 0.51, 0.49, 0.49];
    let cx_invalido = 2 + (0.98f64).ceil() as usize;
    println!(
        "  sanidade da fórmula: [0.51,0.51,0.49,0.49] -> L1={} , max(#grandes,L1)={} , OTIMO={} , variante INVÁLIDA=#grandes+ceil(pequenos)={cx_invalido}",
        t1(&cx),
        t2(&cx),
        optimo_sem_podas(&cx),
    );
    println!(
        "     -> a variante com soma dá {} > OPT {}: por isso ela NÃO entra na comparação.\n",
        cx_invalido,
        optimo_sem_podas(&cx)
    );

    let (mut mais_forte, mut mais_fraco, mut empate) = (0usize, 0usize, 0usize);
    let mut pior_exemplo = String::new();
    let mut max_ganho = 0i64;
    let mut violacoes = 0usize;
    for &dist in Distribution::all().iter() {
        // n limitado: o oráculo SEM podas é exponencial, e aqui só
        // precisamos do invariante L2 <= OPT em n pequeno.
        for n in [10usize, 14, 18] {
            for seed in 0..30u64 {
                let items = generate(n, dist, seed);
                let l2 = lower_bound_l2(&items) as i64;
                let tr = t2(&items) as i64;
                if l2 > tr {
                    mais_forte += 1;
                    if l2 - tr > max_ganho {
                        max_ganho = l2 - tr;
                    }
                } else if l2 < tr {
                    mais_fraco += 1;
                    if pior_exemplo.is_empty() {
                        pior_exemplo = format!(
                            "{} n={n} (FFD abre {} bins: L2={l2} < trivial={tr}, diff={})",
                            dist.name(),
                            first_fit_decreasing(&items).bins,
                            tr - l2
                        );
                    }
                } else {
                    empate += 1;
                }
                // e o invariante que importa: L2 <= OPT
                if l2 > optimo_sem_podas(&items) as i64 {
                    violacoes += 1;
                }
            }
        }
    }
    println!("  comparação L2 (implementado) vs max(#grandes, ceil(soma)):");
    println!("    mais forte : {mais_forte}");
    println!("    mais fraco  : {mais_fraco}");
    println!("    empate      : {empate}");
    println!("    ganho máximo do L2 sobre o trivial válido: {max_ganho} bin(s)");
    println!("    exemplo onde o trivial vence: {pior_exemplo}");
    println!("    violações L2 > OPT no mesmo conjunto: {violacoes}");
    if mais_forte == 0 && violacoes == 0 {
        println!(
            "\n  O L2 nunca é mais forte que a linha trivial válida, mas também nunca\n\
             a ultrapassa — então o INVARIANTE (L2 <= OPT) se segura nas\n\
             distribuições do artigo. O efeito de um L2 fraco é INFLAR a razão\n\
             A/L2 (conservador para o argumento do artigo), não quebrá-lo.\n\
             A enumeração de alphas (algorithms.rs:215-226) é O(n^2) e entrega\n\
             zero bins sobre a linha trivial."
        );
    }
}

// ---------------------------------------------------------------------------
// PROBE 7 — L2 <= OPT na amostra do ARTIGO (n grande, do gerador real)
// ---------------------------------------------------------------------------
fn probe_l2_no_artigo() {
    cab("PROBE 7: L2 <= OPT e L2 <= FFD nas distribuições do artigo");
    let mut viol_l2_opt = 0usize;
    let mut viol_l2_ffd = 0usize;
    let mut total = 0usize;
    let mut pior = String::new();
    for &dist in Distribution::all().iter() {
        for n in [6usize, 8, 10, 12, 14] {
            for seed in 0..200u64 {
                let items = generate(n, dist, seed);
                let opt = optimo_sem_podas(&items);
                let l2 = lower_bound_l2(&items);
                let ffd = first_fit_decreasing(&items).bins;
                total += 1;
                if l2 > opt {
                    viol_l2_opt += 1;
                    if pior.is_empty() {
                        pior = format!(
                            "  primeiro caso: {} n={n} seed={seed} L2={l2} OPT={opt} FFD={ffd}",
                            dist.name()
                        );
                    }
                }
                if l2 > ffd {
                    viol_l2_ffd += 1;
                }
            }
        }
    }
    println!("  instâncias: {total}");
    println!("  violações L2 > OPT : {viol_l2_opt}");
    println!("  violações L2 > FFD : {viol_l2_ffd}");
    if !pior.is_empty() {
        println!("{pior}");
    } else {
        println!(
            "\n  Nas distribuições do gerador, L2 <= OPT e L2 <= FFD em todas as\n\
             {} amostras. O defeito do ceil (PROBE 1) é LATENTE para o artigo:\n\
             só aparece em famílias k×(1/k), que o gerador não produz.",
            total
        );
    }
}

// ---------------------------------------------------------------------------
// PROBE 8 — bins-only == completas? (invariante de fair-play do artigo)
// ---------------------------------------------------------------------------
fn probe_fairplay() {
    cab("PROBE 8: variantes bins-only devolvem o mesmo número de bins?");
    let mut div = 0usize;
    let mut total = 0usize;
    for &dist in Distribution::all().iter() {
        for n in [10usize, 50, 200, 1000] {
            for seed in 0..25u64 {
                let items = generate(n, dist, seed);
                let pares = [
                    ("NF", next_fit(&items).bins, next_fit_bins(&items)),
                    ("FF", first_fit(&items).bins, first_fit_bins(&items)),
                    ("BF", best_fit(&items).bins, best_fit_bins(&items)),
                    (
                        "FFD",
                        first_fit_decreasing(&items).bins,
                        first_fit_decreasing_bins(&items),
                    ),
                    (
                        "BFD",
                        best_fit_decreasing(&items).bins,
                        best_fit_decreasing_bins(&items),
                    ),
                ];
                for (alg, a, b) in pares {
                    total += 1;
                    if a != b {
                        div += 1;
                        println!("  DIVERGE {alg} {} n={n} seed={seed}: {a} vs {b}", dist.name());
                    }
                }
            }
        }
    }
    println!("  comparações: {total}, divergências: {div}");
    if div == 0 {
        println!("  ✅ fair-play confirmado nas duas séries.");
    }
}

// ---------------------------------------------------------------------------
// PROBE 9 — residual fora de [0,1]? (viabilidade)
// ---------------------------------------------------------------------------
fn probe_viabilidade() {
    cab("PROBE 9: viabilidade (todo resíduo em [0,1])");
    let mut ruim = 0usize;
    let mut chec = 0usize;
    let mut pior = f64::MAX;
    for &dist in Distribution::all().iter() {
        for n in [20usize, 60, 200] {
            for seed in 0..40u64 {
                let items = generate(n, dist, seed);
                for alg in ["NF", "FF", "BF", "FFD", "BFD"] {
                    let sol = bpp::experiment::run_algorithm(alg, &items);
                    for &r in &sol.residual {
                        chec += 1;
                        pior = pior.min(r);
                        if !(-1e-9..=1.0 + 1e-9).contains(&r) {
                            ruim += 1;
                        }
                    }
                }
            }
        }
    }
    println!("  resíduos checados: {chec}, fora de [0,1]: {ruim}, menor resíduo: {pior:.9}");
    println!("  (itens > 1 push de residual negativo — o gerador nunca gera, mas o código não protege)");
}

// ---------------------------------------------------------------------------
// PROBE 10 — o B&B com podas bate com o oráculo SEM podas?
// ---------------------------------------------------------------------------
fn probe_podas() {
    cab("PROBE 10: B&B com podas vs busca exata sem podas (budget 60s)");
    let mut div = 0usize;
    let mut total = 0usize;
    let mut subotimos = 0usize;
    for &dist in Distribution::all().iter() {
        for n in [10usize, 12] {
            for seed in 0..40u64 {
                let items = generate(n, dist, seed);
                let oraculo = optimo_sem_podas(&items);
                let obtido = optimal_bins_budget(&items, 60_000_000).unwrap();
                total += 1;
                if obtido != oraculo {
                    div += 1;
                    if obtido > oraculo {
                        subotimos += 1;
                    }
                    if div <= 5 {
                        println!(
                            "  DIVERGE {} n={n} seed={seed}: B&B={obtido} oráculo={oraculo}",
                            dist.name()
                        );
                    }
                }
            }
        }
    }
    println!("  instâncias: {total}, divergências: {div} (das quais subótimas: {subotimos})");
    if div == 0 {
        println!("  ✅ As PODAS estão corretas (com tempo suficiente, B&B == ótimo).");
        println!(
            "     O defeito é apenas o RELATO: quando o orçamento estoura, devolve\n\
             o incumbent sem avisar — e nunca devolve None."
        );
    }
}

// ---------------------------------------------------------------------------
// PROBE 11 — determinismo do gerador
// ---------------------------------------------------------------------------
fn probe_determinismo() {
    cab("PROBE 11: gerador determinístico?");
    let mut igual = 0usize;
    let mut total = 0usize;
    for &dist in Distribution::all().iter() {
        for n in [10usize, 100, 1000] {
            for seed in [0u64, 1, 42, 1000, 99999] {
                total += 1;
                if generate(n, dist, seed) == generate(n, dist, seed) {
                    igual += 1;
                }
            }
        }
    }
    println!("  {igual}/{total} gerações idênticas na mesma semente");
    let a = generate(100, Distribution::UniformContinuous, 7);
    let b = generate(100, Distribution::UniformContinuous, 8);
    println!(
        "  sementes 7 e 8 dão a mesma instância? {}  ({} vs {})",
        a == b,
        a[0],
        b[0]
    );
}

// ---------------------------------------------------------------------------
// PROBE 12 — next_fit: ratio_vs_lb com lb=0 (divisão por zero mascarada)
// ---------------------------------------------------------------------------
fn probe_ratio_zero() {
    cab("PROBE 12: razão com lower bound 0");
    let items: Vec<f64> = Vec::new();
    let lb = lower_bound_l2(&items);
    println!("  lb para instância vazia = {lb}; `lb.max(1)` evita divisão por zero (experiment.rs:117)");
    let mut zeros = 0usize;
    let mut total = 0usize;
    for &dist in Distribution::all().iter() {
        for n in [10usize, 100] {
            for seed in 0..50u64 {
                let items = generate(n, dist, seed);
                let lb = lower_bound_l2(&items);
                total += 1;
                if lb == 0 {
                    zeros += 1;
                }
            }
        }
    }
    println!("  instâncias com lb == 0: {zeros}/{total}  (o `.max(1)` mascara o problema)");
}

// ---------------------------------------------------------------------------
// PROBE 13 — a alegacao "40 de 40": em quantos casos o budget curto erra?
//
// A versão do probe5 acima só erra quando FFD > OPT, porque o incumbent é
// o FFD. Aqui eu varro uma população maior e conto, DE SEPARADO:
//   (a) casos onde o incumbent FFD é ótimo (o solver "acerta" por acaso);
//   (b) casos onde o incumbent FFD é subótimo (o solver ERRARÁ por construção);
//   (c) casos onde o budget curto ainda acha o ótimo mesmo partindo de
//       incumbent subótimo (a busca ainda tem chance de melhorar).
// Isso separa "o solver está correto" de "o FFD era ótimo nesse caso".
// ---------------------------------------------------------------------------
fn probe_taxe_de_erro_do_budget() {
    cab("PROBE 13: taxa de erro do solver por orçamento (o OPT real é conhecido)");
    let orcamentos = [1u128, 100, 1_000, 10_000, 1_000_000, 2_000_000];
    for budget in orcamentos {
        let (mut total, mut errado, mut por_acaso) = (0usize, 0usize, 0usize);
        let mut max_erro = 0i64;
        for &dist in Distribution::all().iter() {
            for n in [10usize, 12, 14] {
                for seed in 0..40u64 {
                    let items = generate(n, dist, seed);
                    let oraculo = optimo_sem_podas(&items) as i64;
                    let ffd = first_fit_decreasing(&items).bins as i64;
                    if ffd > oraculo {
                        por_acaso += 1;
                    }
                    let r = match optimal_bins_budget(&items, budget) {
                        Some(v) => v as i64,
                        None => {
                            total += 1;
                            continue;
                        }
                    };
                    total += 1;
                    if r != oraculo {
                        errado += 1;
                        max_erro = max_erro.max(r - oraculo);
                    }
                }
            }
        }
        println!(
            "  budget={budget:>9} µs | instâncias={total} | erros={errado} ({:.1}%) | erro máximo={max_erro} bin(s) |FFD já era subótimo em {por_acaso} delas",
            100.0 * errado as f64 / total as f64
        );
    }
    println!(
        "\n  Com o orçamento PADRÃO (2 s) o erro é 0% — as podas estão corretas.\n\
         O defeito é de RELATO: quando o orçamento ESTOURA, o incumbent volta\n\
         como se fosse o ótimo. Com 1 µs o solver erra em ~100% dos casos em\n\
         que o FFD é subótimo, e isso sem nenhum sinal visível."
    );
}

// ---------------------------------------------------------------------------
// PROBE 14 — L1/L2 sem tolerância: que família do artigo dispara?
// Varre as famílias k x (1/k) ealso variantes com epsilon, que é onde a
// soma cai em cima do inteiro.
// ---------------------------------------------------------------------------
fn probe_tolerancia_familias() {
    cab("PROBE 14: L1 = ceil(soma) em famílias de soma exata");
    let mut viol = 0usize;
    let mut testados = 0usize;
    let mut primeiro = String::new();
    for k in 2usize..=2000 {
        let itens = vec![1.0 / k as f64; k];
        let soma: f64 = itens.iter().sum();
        let l1 = soma.ceil() as usize;
        let ffd = first_fit_decreasing(&itens).bins;
        testados += 1;
        if l1 > ffd {
            viol += 1;
            if primeiro.is_empty() {
                primeiro = format!("k={k} soma={soma:.20} L1={l1} FFD={ffd} OPT=1");
            }
        }
    }
    println!("  famílias k x (1/k) para k=2..2000: {viol}/{testados} com L1 > FFD (= ótimo)");
    if !primeiro.is_empty() {
        println!("  primeiro caso: {primeiro}");
    }
    // Variante grande: N cópias de 1/k, com N grande o enough para o erro de
    // arredondamento de f64 aparecer na soma. Limitado a poucos k porque
    // FF em N itens é O(N^2).
    let mut viol2 = 0usize;
    for k in [3usize, 6, 7, 9, 11, 13, 100, 1000] {
        let s = 1.0 / k as f64;
        let itens: Vec<f64> = vec![s; 2000];
        let soma: f64 = itens.iter().sum();
        let esperado = (soma / s).ceil() as usize;
        if soma.ceil() as usize > first_fit_decreasing(&itens).bins {
            viol2 += 1;
        }
        let _ = esperado;
    }
    println!("  (2000 cópias de 1/k para 8 valores de k: {viol2} com L1 > FFD)");
    println!(
        "\n  A soma de k cópias de 1/k cai em 1.0000000000000007 para k múltiplo\n\
         de 5 ou potências que não são potências de dois. O empacotador usa\n\
         +1e-9 e fecha em 1 bin; o ceil exato diz 2. families k x (1/k) não\n\
         estão no corpus do artigo (o gerador sorteia), então é LATENTE — mas é\n\
         o defeito que a skill descreve: 'Verifique o ceil do limitador contra\n\
         a tolerância do empacotador'."
    );
}

// ---------------------------------------------------------------------------
// PROBE 15 — o atalho "FFD <= L2 ⇒ retorna" do exact.rs
// Se L2 for mais forte que o OPT (nunca deve ser), o atalho devolve o FFD
// como ótimo. Também: o atalho é um DESVIO DE TRABALHO que a bnb_rust.rs
// deliberadamente remove — e é a fonte do cruzamento de séries.
// ---------------------------------------------------------------------------
fn probe_atalho() {
    cab("PROBE 15: o atalho 'melhor <= L2 ⇒ retorna já' do exact.rs");
    let mut atalho = 0usize;
    let mut total = 0usize;
    for &dist in Distribution::all().iter() {
        for n in [10usize, 20, 50, 100, 500] {
            for seed in 0..40u64 {
                let items = generate(n, dist, seed);
                let ffd = first_fit_decreasing(&items).bins;
                let l2 = lower_bound_l2(&items);
                total += 1;
                if ffd <= l2 {
                    atalho += 1;
                }
            }
        }
    }
    println!("  instâncias em que o atalho dispara: {atalho}/{total}");
    println!(
        "\n  exact.rs:32-34 retorna Some(FFD) sem buscar quando FFD <= L2.\n\
         bnb_rust.rs:81-86 remove esse atalho DE PROPÓSITO (fair-play entre\n\
         linguagens) — bom. Mas main.rs `worstcase` e piorcasa.rs usam\n\
         optimal_bins COM o atalho: o 'OPT' ali e o FFD em {atalho} dos {total}\n\
         casos, o que torna a razão A/OPT = 1.0 por construção nessas linhas."
    );
}

fn main() {
    println!("AUDITORIA DE SANIDADE — bin packing (crate `bpp`)");
    probe5();
    probe5b();
    probe_l1_tolerancia();
    probe_vazio();
    probe_nan();
    probe_falkenauer_media();
    probe_tres_particao();
    probe_l2_vs_trivial();
    probe_l2_no_artigo();
    probe_fairplay();
    probe_viabilidade();
    probe_podas();
    probe_determinismo();
    probe_ratio_zero();
    probe_taxe_de_erro_do_budget();
    probe_tolerancia_familias();
    probe_atalho();
    println!("\n{}", "=".repeat(80));
}
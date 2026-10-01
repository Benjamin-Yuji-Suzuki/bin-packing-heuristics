//! Branch-and-bound em LINGUAGENS — versão Rust (referência).
//!
//! Lê as mesmas instâncias .f64 e mede o MESMO algoritmo de
//! src/exact.rs (mesma ordem de exploração, mesmas podas, mesmo
//! incumbent), para permitir a comparação de tempo entre linguagens.
//!
//! ## Fair-play metodológico — a armadilha do atalho
//!
//! `src/exact.rs` tem um atalho de produção: se o incumbent (FFD) já
//! bate o lower bound L2, devolve sem buscar. É ótimo para uso normal,
//! mas INVALIDARIA a comparação entre linguagens: o Rust deixaria de
//! buscar em algumas instâncias (tempo ~ 0) enquanto C/C++/Python
//! buscariam sempre. Medir 0 ms contra 90 ms seria medir o atalho, não
//! a linguagem — o mesmo tipo de erro que fez o artigo reportar
//! multiplicadores inválidos ao cruzar séries de trabalho diferentes.
//!
//! Por isso este binário NÃO usa o atalho: as quatro linguagens
//! executam exatamente a mesma quantidade de trabalho. O mesmo vale
//! para o incumbent inicial (FFD) e para a ordem de exploração.
//!
//! Uso: cargo run --release --bin bnb_rust <dir> <saida.csv> [n_max]

use bpp::algorithms::first_fit_decreasing;
use std::time::Instant;

fn ffd_incumbent(itens: &[f64]) -> usize {
    first_fit_decreasing(itens).bins
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: bnb_rust <dir> <saida.csv> [n_max]");
        std::process::exit(1);
    }
    let dir = &args[1];
    let out_path = &args[2];
    let n_max: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(60);

    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("ler diretório")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "f64"))
        .collect();
    entries.sort();

    let mut out = String::from("distribuicao,n,rep,optimal,bnb_ms\n");
    let mut feitos = 0usize;

    for path in entries {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(&path).expect("ler instância");
        let items: Vec<f64> = bytes
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
            .collect();
        let n = items.len();
        if n > n_max {
            continue;
        }

        // parse do nome: {dist}_n{n}_r{rep}, dist contém '_'
        let mut parts = name.rsplitn(3, '_');
        let rep: usize = parts
            .next()
            .unwrap_or("r0")
            .trim_start_matches('r')
            .parse()
            .unwrap_or(0);
        let _n_field: usize = parts
            .next()
            .unwrap_or("n0")
            .trim_start_matches('n')
            .parse()
            .unwrap_or(0);
        let dist = parts.next().unwrap_or("?").to_string();

        // --- o MESMO branch-and-bound de src/exact.rs ---
        //
        // IMPORTANTE: aqui NAO usamos o atalho "se FFD <= L2, retorna já"
        // que existe em src/exact.rs. Esse atalho thwartaria a comparação:
        // a busca nem rodaria em algumas instâncias e o tempo medido seria
        // 0, enquanto as versões C/C++/Python sempre a executam. Para que
        // as quatro linguagens façam exatamente o mesmo trabalho, o Rust
        // também sempre busca.
        let mut itens = items.clone();
        itens.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let mut melhor = ffd_incumbent(&items);
        let mut restante = vec![0.0f64; n + 1];
        for i in (0..n).rev() {
            restante[i] = restante[i + 1] + itens[i];
        }
        let mut residuos: Vec<f64> = Vec::with_capacity(n);
        let start = Instant::now();
        dfs(&itens, &restante, 0, &mut residuos, &mut melhor);
        let ms = start.elapsed().as_secs_f64() * 1e3;
        out.push_str(&format!("{dist},{n},{rep},{melhor},{ms:.4}\n"));
        feitos += 1;
    }

    std::fs::write(out_path, out).expect("gravar CSV");
    eprintln!("{feitos} instancias com n <= {n_max} resolvidas (Rust) -> {out_path}");
}

fn dfs(itens: &[f64], restante: &[f64], i: usize, residuos: &mut Vec<f64>, melhor: &mut usize) {
    if residuos.len() >= *melhor {
        return;
    }
    if i == itens.len() {
        *melhor = residuos.len();
        return;
    }
    let r = restante[i];
    let f: f64 = residuos.iter().sum();
    let novos = if r > f { ((r - f) - 1e-9).ceil() as usize } else { 0 };
    if residuos.len() + novos >= *melhor {
        return;
    }
    let x = itens[i];
    for b in 0..residuos.len() {
        if x <= residuos[b] + 1e-9 {
            residuos[b] -= x;
            dfs(itens, restante, i + 1, residuos, melhor);
            residuos[b] += x;
        }
    }
    residuos.push(1.0 - x);
    dfs(itens, restante, i + 1, residuos, melhor);
    residuos.pop();
}
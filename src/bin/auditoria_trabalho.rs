//! AUDITORIA DE EQUIVALENCIA DE TRABALHO — versao Rust
//!
//! Conta as operacoes elementares (comparacoes e subtracoes sobre o
//! residuo) que cada algoritmo executa, usando as MESMAS instancias que
//! o benchmark mede. O script Python compara estas contagens com as do
//! C instrumentado.
//!
//! Uso: cargo run --release --bin auditoria_trabalho <dir_instancias>

use bpp::generators::Distribution;
use std::io::Write;

fn ff_conta(items: &[f64]) -> (usize, usize, usize) {
    let mut residual: Vec<f64> = Vec::with_capacity(items.len());
    let (mut comps, mut subs) = (0usize, 0usize);
    'outer: for &x in items {
        for r in residual.iter_mut() {
            comps += 1;
            if x <= *r + 1e-9 {
                *r -= x;
                subs += 1;
                continue 'outer;
            }
        }
        residual.push(1.0 - x);
    }
    (comps, subs, residual.len())
}

fn nf_conta(items: &[f64]) -> (usize, usize, usize) {
    let mut bins = 1usize;
    let mut residual = 1.0f64;
    let (mut comps, mut subs) = (0usize, 0usize);
    for &x in items {
        comps += 1;
        if x <= residual + 1e-9 {
            residual -= x;
            subs += 1;
        } else {
            bins += 1;
            residual = 1.0 - x;
        }
    }
    (comps, subs, bins)
}

fn bf_conta(items: &[f64]) -> (usize, usize, usize) {
    let mut residual: Vec<f64> = Vec::with_capacity(items.len());
    let (mut comps, mut subs) = (0usize, 0usize);
    for &x in items {
        let mut best: Option<usize> = None;
        let mut best_r = f64::INFINITY;
        for (b, &r) in residual.iter().enumerate() {
            comps += 1;
            if x <= r + 1e-9 && r < best_r {
                best = Some(b);
                best_r = r;
            }
        }
        match best {
            Some(b) => {
                residual[b] -= x;
                subs += 1;
            }
            None => {
                residual.push(1.0 - x);
            }
        }
    }
    (comps, subs, residual.len())
}

fn main() {
    let dir = std::env::args().nth(1).expect("uso: auditoria_trabalho <dir>");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("ler dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |x| x == "f64"))
        .collect();
    paths.sort();
    let n_inst = paths.len();

    let mut out = String::from("alg,dist,n,comp,sub,bins\n");
    for path in paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(&path).expect("ler");
        let items: Vec<f64> = bytes
            .chunks_exact(8)
            .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
            .collect();
        let n = items.len();
        // nome: {dist}_n{n}_r{rep}; dist contem '_'
        let mut parts = name.rsplitn(3, '_');
        let _rep = parts.next();
        let _n = parts.next();
        let dist = parts.next().unwrap_or("?").to_string();

        for (alg, r) in [
            ("NF", nf_conta(&items)),
            ("FF", ff_conta(&items)),
            ("BF", bf_conta(&items)),
        ] {
            out.push_str(&format!("{alg},{dist},{n},{},{},{}\n", r.0, r.1, r.2));
        }
    }
    std::io::stdout().write_all(out.as_bytes()).expect("escrever");
    let _ = Distribution::all();
    eprintln!("{n_inst} instancias auditadas");
}
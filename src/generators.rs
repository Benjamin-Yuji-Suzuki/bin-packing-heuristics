use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Distribuição usada pelo gerador de instâncias.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distribution {
    /// U[0, 1] — itens pequenos e grandes misturados.
    UniformContinuous,
    /// U{1/100, ..., 1} — tamanhos discretizados em centésimos.
    UniformDiscrete100,
    /// Itens apenas em (1/4, 1/2] — caso difícil (3-Partition).
    ThreePartition,
    /// Falkenauer U{1/10, ..., 1/2} — média 0.275, caso "fácil" p/ FFD.
    FalkenauerU120,
}

impl Distribution {
    pub fn name(self) -> &'static str {
        match self {
            Distribution::UniformContinuous => "uniforme_continua",
            Distribution::UniformDiscrete100 => "uniforme_discreta_100",
            Distribution::ThreePartition => "tres_particao",
            Distribution::FalkenauerU120 => "falkenauer_u120",
        }
    }

    pub fn all() -> [Distribution; 4] {
        [
            Distribution::UniformContinuous,
            Distribution::UniformDiscrete100,
            Distribution::ThreePartition,
            Distribution::FalkenauerU120,
        ]
    }
}

/// Gera uma instância com `n` itens no intervalo (0, 1], com semente
/// reprodutível (mesma semente => mesma instância).
///
/// A semente é misturada com o tamanho `n` (splitmix64) para que
/// instâncias de tamanhos diferentes não sejam correlacionadas.
pub fn generate(n: usize, dist: Distribution, seed: u64) -> Vec<f64> {
    let mixed = splitmix64(seed ^ (n as u64).wrapping_mul(0x9E3779B97F4A7C15));
    let mut rng = StdRng::seed_from_u64(mixed);
    match dist {
        Distribution::UniformContinuous => (0..n).map(|_| rng.gen::<f64>()).collect(),
        Distribution::UniformDiscrete100 => (0..n)
            .map(|_| rng.gen_range(1..=100) as f64 / 100.0)
            .collect(),
        Distribution::ThreePartition => (0..n)
            // 26..=50 / 100 => tamanhos em (0.25, 0.50]
            .map(|_| rng.gen_range(26..=50) as f64 / 100.0)
            .collect(),
        Distribution::FalkenauerU120 => (0..n)
            // U{1/10, ..., 1/2} — média 0.275
            .map(|_| rng.gen_range(10..=50) as f64 / 100.0)
            .collect(),
    }
}

/// Misturador de bits (splitmix64) — determinístico e independente de
/// versão do crate rand.
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

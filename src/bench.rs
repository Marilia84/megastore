use crate::product::Product;
use crate::recommender::Recommender;
use std::time::Instant;

/// Gerador pseudoaleatório simples (LCG), só para não depender de crates externos.
/// Usar a mesma `seed` gera sempre os mesmos dados -> medições reproduzíveis.
pub struct Lcg(u64);

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Lcg(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    /// número em [0, max)
    pub fn below(&mut self, max: u64) -> u64 {
        self.next() % max
    }
}

/// Monta um sistema sintético:
/// - `n_products` produtos, espalhados em ~n/100 categorias (mín. 10);
/// - `n_customers` clientes, cada um com `purchases_each` compras aleatórias.
pub fn build_synthetic(n_products: u32, n_customers: u32, purchases_each: u32, seed: u64) -> Recommender {
    let mut rng = Lcg::new(seed);
    let mut r = Recommender::new();
    let n_categories = (n_products / 100).max(10) as u64;

    for id in 0..n_products {
        let cat = format!("categoria-{}", rng.below(n_categories));
        r.add_product(Product::new(id, format!("Produto {}", id), cat, 10.0 + (id % 500) as f64))
            .unwrap();
    }
    for c in 0..n_customers {
        for _ in 0..purchases_each {
            let p = rng.below(n_products as u64) as u32;
            let nota = 1 + rng.below(5) as u8;
            r.register_purchase(c, p, Some(nota)).unwrap();
        }
    }
    r
}

/// Roda o benchmark para vários volumes e imprime uma tabela.
pub fn run() {
    let sizes: [u32; 4] = [1_000, 10_000, 100_000, 500_000];
    let queries = 200;

    println!(
        "{:>10} {:>10} {:>11} {:>11} {:>14} {:>16} {:>18}",
        "produtos", "clientes", "vértices", "arestas", "build (ms)", "recomend. (µs)", "matriz adj. (GB)"
    );

    for &n in &sizes {
        let customers = n / 2;

        let t = Instant::now();
        let r = build_synthetic(n, customers, 5, 42);
        let build_ms = t.elapsed().as_secs_f64() * 1000.0;

        // mede o tempo médio de `queries` recomendações para clientes diferentes
        let mut rng = Lcg::new(7);
        let t = Instant::now();
        let mut total = 0usize;
        for _ in 0..queries {
            let c = rng.below(customers as u64) as u32;
            total += r.recommend_for_customer(c, 10).len(); // usa o resultado para o compilador não "pular" a chamada
        }
        let avg_us = t.elapsed().as_secs_f64() * 1e6 / queries as f64;

        // se usássemos matriz de adjacência com f64: V*V*8 bytes
        let v = r.graph().node_count() as f64;
        let matriz_gb = v * v * 8.0 / 1e9;

        println!(
            "{:>10} {:>10} {:>11} {:>11} {:>14.1} {:>16.1} {:>18.2}   (recs médias: {:.1})",
            n,
            customers,
            r.graph().node_count(),
            r.graph().edge_count(),
            build_ms,
            avg_us,
            matriz_gb,
            total as f64 / queries as f64
        );
    }
}

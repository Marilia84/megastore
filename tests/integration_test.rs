// Testes de INTEGRAÇÃO:
use megastore::bench::build_synthetic;
use megastore::product::Product;
use megastore::recommender::Recommender;
use std::collections::HashSet;
use std::time::Instant;

#[test]
fn fluxo_completo_cadastro_compra_recomendacao() {
    let mut r = Recommender::new();
    r.add_product(Product::new(1, "Notebook", "Eletrônicos", 5000.0)).unwrap();
    r.add_product(Product::new(2, "Mouse", "Acessórios", 80.0)).unwrap();
    r.add_product(Product::new(3, "Livro Rust", "Livros", 100.0)).unwrap();

    r.register_purchase(1, 1, Some(5)).unwrap(); // Ana: notebook
    r.register_purchase(2, 1, Some(5)).unwrap(); // Beto: notebook + mouse
    r.register_purchase(2, 2, Some(5)).unwrap();

    let recs = r.recommend_for_customer(1, 5);
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].product_id, 2);
    assert_eq!(r.catalog().get(2).unwrap().name, "Mouse");
}

#[test]
fn dados_sinteticos_sem_recomendacoes_duplicadas() {
    let r = build_synthetic(2_000, 1_000, 5, 1);
    for cliente in 0..50 {
        let recs = r.recommend_for_customer(cliente, 20);
        let ids: HashSet<u32> = recs.iter().map(|x| x.product_id).collect();
        assert_eq!(ids.len(), recs.len(), "recomendação duplicada para o cliente {}", cliente);
        assert!(recs.len() <= 20);
    }
}

#[test]
fn recomendacao_e_deterministica() {
    let r = build_synthetic(1_000, 500, 5, 3);
    assert_eq!(r.recommend_for_customer(10, 10), r.recommend_for_customer(10, 10));
}

#[test]
fn recomendacao_em_catalogo_grande_e_rapida() {
    let r = build_synthetic(50_000, 25_000, 5, 5);
    let t = Instant::now();
    for c in 0..100 {
        r.recommend_for_customer(c, 10);
    }
    // 100 consultas em menos de 5 segundos (margem larga, mesmo em build debug)
    assert!(t.elapsed().as_secs() < 5);
}

#[test]
fn remover_compra_atualiza_o_grafo_e_as_recomendacoes() {
    let mut r = Recommender::new();
    r.add_product(Product::new(1, "Notebook", "Eletrônicos", 5000.0)).unwrap();
    r.add_product(Product::new(2, "Mouse", "Acessórios", 80.0)).unwrap();

    r.register_purchase(1, 1, Some(5)).unwrap();
    r.register_purchase(2, 1, Some(5)).unwrap();
    r.register_purchase(2, 2, Some(5)).unwrap();
    assert_eq!(r.recommend_for_customer(1, 5).len(), 1);

    let arestas_antes = r.graph().edge_count();
    r.remove_purchase(2, 2).unwrap();
    assert_eq!(r.graph().edge_count(), arestas_antes - 1);
    assert!(r.recommend_for_customer(1, 5).is_empty());
    assert_eq!(r.customer_purchases(2).len(), 1);
}

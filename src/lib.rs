//! Biblioteca do ConectaStore.
//! Cada `pub mod` abaixo corresponde a um arquivo em src/.
//! Ter um lib.rs permite que os testes da pasta tests/ importem o projeto.

pub mod bench;       // geração de dados sintéticos + medição de tempo
pub mod catalog;     // cadastro e consulta de produtos (HashMap)
pub mod graph;       // grafo (lista de adjacência)
pub mod product;     // struct Product
pub mod recommender; // algoritmo de recomendação (BFS por níveis)

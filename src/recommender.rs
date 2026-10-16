use crate::catalog::Catalog;
use crate::graph::{Graph, NodeKey};
use crate::product::Product;
use std::collections::{HashMap, VecDeque};

/// Peso da aresta Produto—Categoria (relação mais fraca que uma compra real).
pub const PESO_CATEGORIA: f64 = 0.4;
/// Peso da aresta Cliente—Produto quando o cliente comprou mas não avaliou.
pub const PESO_COMPRA_SEM_NOTA: f64 = 0.6;

/// Uma recomendação: qual produto e o quão forte é a conexão (score).
#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub product_id: u32,
    pub score: f64,
}

/// Junta o catálogo (HashMap) com o grafo de relações.
#[derive(Debug, Default)]
pub struct Recommender {
    catalog: Catalog,
    graph: Graph,
}

impl Recommender {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Cadastra o produto no catálogo E cria os vértices/arestas dele no grafo:
    /// Produto — Categoria.
    pub fn add_product(&mut self, product: Product) -> Result<(), String> {
        let id = product.id;
        let categoria = product.category.clone();
        self.catalog.add(product)?; // se o id for duplicado, para aqui com Err

        let p = self.graph.get_or_add_node(NodeKey::Product(id));
        let c = self.graph.get_or_add_node(NodeKey::Category(categoria));
        self.graph.add_edge(p, c, PESO_CATEGORIA);
        Ok(())
    }

    /// Registra uma compra (e opcionalmente uma nota de 1 a 5).
    /// Peso da aresta = nota / 5 (quem deu nota 5 tem conexão mais forte que quem deu nota 2).
    pub fn register_purchase(&mut self, customer_id: u32, product_id: u32, rating: Option<u8>) -> Result<(), String> {
        if self.catalog.get(product_id).is_none() {
            return Err(format!("Produto {} não existe", product_id));
        }
        let peso = match rating {
            Some(n) if (1..=5).contains(&n) => n as f64 / 5.0,
            Some(n) => return Err(format!("Nota inválida: {} (use 1 a 5)", n)),
            None => PESO_COMPRA_SEM_NOTA,
        };
        let c = self.graph.get_or_add_node(NodeKey::Customer(customer_id));
        let p = self.graph.get_or_add_node(NodeKey::Product(product_id));
        self.graph.add_edge(c, p, peso);
        Ok(())
    }

    /// Remove uma compra (aresta Cliente—Produto). Útil para devolução/cancelamento.
    pub fn remove_purchase(&mut self, customer_id: u32, product_id: u32) -> Result<(), String> {
        let c = self
            .graph
            .node_index(&NodeKey::Customer(customer_id))
            .ok_or_else(|| format!("Cliente {} não existe", customer_id))?;
        let p = self
            .graph
            .node_index(&NodeKey::Product(product_id))
            .ok_or_else(|| format!("Produto {} não existe", product_id))?;
        if self.graph.remove_edge(c, p) {
            Ok(())
        } else {
            Err(format!("Cliente {} não comprou o produto {}", customer_id, product_id))
        }
    }

    /// Compras de um cliente: lista de (id do produto, peso da aresta), ordenada por id.
    pub fn customer_purchases(&self, customer_id: u32) -> Vec<(u32, f64)> {
        let Some(c) = self.graph.node_index(&NodeKey::Customer(customer_id)) else {
            return Vec::new();
        };
        let mut compras: Vec<(u32, f64)> = self
            .graph
            .neighbors(c)
            .iter()
            .filter_map(|&(viz, peso)| match self.graph.key(viz) {
                NodeKey::Product(id) => Some((*id, peso)),
                _ => None,
            })
            .collect();
        compras.sort_by_key(|x| x.0);
        compras
    }

    /// Recomenda a partir de um CLIENTE.
    /// Caminho: Cliente -> Produto (já comprou) -> Cliente/Categoria -> Produto (candidato)
    /// Por isso os candidatos estão a 3 saltos (profundidade 3).
    pub fn recommend_for_customer(&self, customer_id: u32, limit: usize) -> Vec<Recommendation> {
        match self.graph.node_index(&NodeKey::Customer(customer_id)) {
            Some(start) => self.explore(start, 3, limit),
            None => Vec::new(), // cliente desconhecido: sem histórico, sem recomendação
        }
    }

    /// Recomenda produtos relacionados a um PRODUTO ("quem viu/comprou este também levou...").
    /// Caminho: Produto -> Cliente/Categoria -> Produto (candidato) = profundidade 2.
    pub fn recommend_for_product(&self, product_id: u32, limit: usize) -> Vec<Recommendation> {
        match self.graph.node_index(&NodeKey::Product(product_id)) {
            Some(start) => self.explore(start, 2, limit),
            None => Vec::new(),
        }
    }

    /// BFS clássica (com FILA) + acúmulo de score.
    ///
    /// - `fila` (VecDeque): FIFO, visita os vértices em ordem crescente de distância.
    /// - `dist` (HashMap): distância de cada vértice até o início. Também serve de
    ///   "visitado": quem já está em `dist` nunca é enfileirado de novo. É isso que
    ///   PREVINE RECOMENDAÇÕES DUPLICADAS e impede de voltar para trás.
    /// - Os produtos que o cliente já comprou estão a distância 1, então nunca aparecem
    ///   entre os candidatos de distância 3 -> nunca são recomendados de novo.
    /// - `score`: se um vértice é alcançado por vários caminhos de mesmo comprimento
    ///   (ex.: 2 clientes parecidos compraram o mesmo produto), os scores somam.
    ///   Como a fila é FIFO, todos os vértices do nível d-1 saem da fila antes de
    ///   qualquer vértice do nível d, então o score de um vértice já está completo
    ///   quando chega a vez dele.
    /// - Não expandimos vértices que já estão em `target_depth`: o custo depende da
    ///   vizinhança do início, não do tamanho total do catálogo.
    fn explore(&self, start: usize, target_depth: usize, limit: usize) -> Vec<Recommendation> {
        let mut dist: HashMap<usize, usize> = HashMap::new();
        let mut score: HashMap<usize, f64> = HashMap::new();
        let mut fila: VecDeque<usize> = VecDeque::new();

        dist.insert(start, 0);
        score.insert(start, 1.0);
        fila.push_back(start);

        while let Some(node) = fila.pop_front() {
            let d = dist[&node];
            if d == target_depth {
                continue; // já chegamos na profundidade-alvo: não vamos mais longe
            }
            let s = score[&node];

            for &(viz, peso) in self.graph.neighbors(node) {
                match dist.get(&viz) {
                    // vizinho novo: descobre, guarda distância e score, entra na fila
                    None => {
                        dist.insert(viz, d + 1);
                        score.insert(viz, s * peso);
                        fila.push_back(viz);
                    }
                    // outro caminho de mesmo comprimento até o vizinho: soma o score
                    Some(&dv) if dv == d + 1 => {
                        *score.get_mut(&viz).unwrap() += s * peso;
                    }
                    // já visto em nível anterior (ou no mesmo): ignora
                    _ => {}
                }
            }
        }

        // candidatos = produtos exatamente a `target_depth` saltos
        let mut result: Vec<Recommendation> = dist
            .iter()
            .filter(|&(_, &d)| d == target_depth)
            .filter_map(|(&idx, _)| match self.graph.key(idx) {
                NodeKey::Product(id) => Some(Recommendation { product_id: *id, score: score[&idx] }),
                _ => None, // ignora clientes e categorias que caíram nesse nível
            })
            .collect();

        // maior score primeiro; empate resolvido pelo menor id (resultado determinístico)
        result.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap()
                .then(a.product_id.cmp(&b.product_id))
        });
        result.truncate(limit);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn base() -> Recommender {
        let mut r = Recommender::new();
        r.add_product(Product::new(1, "Notebook", "Eletrônicos", 3500.0)).unwrap();
        r.add_product(Product::new(2, "Livro Rust", "Livros", 90.0)).unwrap();
        r.add_product(Product::new(3, "Mouse", "Acessórios", 80.0)).unwrap();
        r.add_product(Product::new(4, "Sofá", "Móveis", 2000.0)).unwrap();
        r
    }

    #[test]
    fn recomenda_via_cliente_parecido() {
        let mut r = base();
        // Ana comprou 1 e 2; Bia comprou 1 e 3 -> Ana deve receber o 3
        r.register_purchase(10, 1, Some(5)).unwrap();
        r.register_purchase(10, 2, Some(5)).unwrap();
        r.register_purchase(20, 1, Some(5)).unwrap();
        r.register_purchase(20, 3, Some(5)).unwrap();
        let rec = r.recommend_for_customer(10, 5);
        assert_eq!(rec.len(), 1);
        assert_eq!(rec[0].product_id, 3);
    }

    #[test]
    fn nunca_recomenda_o_que_ja_comprou() {
        let mut r = base();
        r.register_purchase(10, 1, None).unwrap();
        r.register_purchase(20, 1, None).unwrap();
        r.register_purchase(20, 3, None).unwrap();
        let rec = r.recommend_for_customer(10, 10);
        assert!(rec.iter().all(|x| x.product_id != 1));
    }

    #[test]
    fn nao_gera_recomendacoes_duplicadas() {
        let mut r = base();
        // vários clientes ligam o produto 3 ao cliente 10 por caminhos diferentes
        for c in [10, 20, 30, 40] {
            r.register_purchase(c, 1, Some(4)).unwrap();
        }
        for c in [20, 30, 40] {
            r.register_purchase(c, 3, Some(4)).unwrap();
        }
        let rec = r.recommend_for_customer(10, 10);
        let ids: HashSet<u32> = rec.iter().map(|x| x.product_id).collect();
        assert_eq!(ids.len(), rec.len());
    }

    #[test]
    fn mais_caminhos_gera_score_maior() {
        let mut r = base();
        r.register_purchase(10, 1, Some(5)).unwrap();
        // produto 3: comprado por 2 clientes parecidos; produto 2: por 1 só
        r.register_purchase(20, 1, Some(5)).unwrap();
        r.register_purchase(20, 3, Some(5)).unwrap();
        r.register_purchase(30, 1, Some(5)).unwrap();
        r.register_purchase(30, 3, Some(5)).unwrap();
        r.register_purchase(40, 1, Some(5)).unwrap();
        r.register_purchase(40, 2, Some(5)).unwrap();
        let rec = r.recommend_for_customer(10, 5);
        assert_eq!(rec[0].product_id, 3);
        assert!(rec[0].score > rec[1].score);
    }

    #[test]
    fn recomenda_por_categoria_quando_nao_ha_outros_clientes() {
        let mut r = base();
        r.add_product(Product::new(5, "Teclado", "Eletrônicos", 200.0)).unwrap();
        r.register_purchase(10, 1, Some(5)).unwrap();
        let rec = r.recommend_for_customer(10, 5);
        assert_eq!(rec[0].product_id, 5); // mesma categoria do Notebook
    }

    #[test]
    fn recomenda_a_partir_de_produto() {
        let mut r = base();
        r.register_purchase(10, 1, Some(5)).unwrap();
        r.register_purchase(10, 2, Some(5)).unwrap();
        let rec = r.recommend_for_product(1, 5);
        assert_eq!(rec[0].product_id, 2);
    }

    #[test]
    fn remover_compra_muda_a_recomendacao() {
        let mut r = base();
        r.register_purchase(10, 1, Some(5)).unwrap();
        r.register_purchase(20, 1, Some(5)).unwrap();
        r.register_purchase(20, 3, Some(5)).unwrap();
        assert_eq!(r.recommend_for_customer(10, 5)[0].product_id, 3);

        r.remove_purchase(20, 3).unwrap(); // Bia devolveu o mouse
        assert!(r.recommend_for_customer(10, 5).is_empty());
        assert!(r.remove_purchase(20, 3).is_err()); // não dá para remover duas vezes
        assert!(r.remove_purchase(999, 3).is_err()); // cliente inexistente
    }

    #[test]
    fn lista_compras_do_cliente() {
        let mut r = base();
        r.register_purchase(10, 2, Some(5)).unwrap();
        r.register_purchase(10, 1, None).unwrap();
        let compras = r.customer_purchases(10);
        assert_eq!(compras.len(), 2);
        assert_eq!(compras[0].0, 1); // ordenado por id
        assert_eq!(compras[1], (2, 1.0));
        assert!(r.customer_purchases(555).is_empty());
    }

    #[test]
    fn entradas_invalidas() {
        let mut r = base();
        assert!(r.register_purchase(10, 999, None).is_err()); // produto inexistente
        assert!(r.register_purchase(10, 1, Some(9)).is_err()); // nota fora de 1..5
        assert!(r.recommend_for_customer(12345, 5).is_empty()); // cliente desconhecido
        assert!(r.add_product(Product::new(1, "Repetido", "X", 1.0)).is_err());
    }
}

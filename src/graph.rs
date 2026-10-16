use std::collections::HashMap;

/// Tipos de vértice do grafo. Um grafo só, com 3 tipos de vértice (grafo heterogêneo).
/// `Hash + Eq` são necessários para usar NodeKey como chave de HashMap.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKey {
    Customer(u32),
    Product(u32),
    Category(String),
}

/// Grafo NÃO direcionado e PONDERADO, implementado com lista de adjacência.
#[derive(Debug, Default)]
pub struct Graph {
    keys: Vec<NodeKey>,                 // chave para "traduzir" de volta
    index: HashMap<NodeKey, usize>,     // chave -> índice (para achar o vértice em O(1))
    adj: Vec<Vec<(usize, f64)>>,        // adj[i] = lista de (vizinho, peso)
    edge_count: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Devolve o índice do vértice; se ele ainda não existe, cria. O(1) médio.
    pub fn get_or_add_node(&mut self, key: NodeKey) -> usize {
        if let Some(&i) = self.index.get(&key) {
            return i;
        }
        let i = self.keys.len();
        self.keys.push(key.clone());
        self.index.insert(key, i);
        self.adj.push(Vec::new()); // lista de vizinhos vazia para o novo vértice
        i
    }

    /// Procura o índice de um vértice sem criar. Retorna None se não existe.
    pub fn node_index(&self, key: &NodeKey) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub fn key(&self, idx: usize) -> &NodeKey {
        &self.keys[idx]
    }

    /// Vizinhos de um vértice, como slice de (vizinho, peso).
    pub fn neighbors(&self, idx: usize) -> &[(usize, f64)] {
        &self.adj[idx]
    }

    /// Se a aresta já existir, NÃO duplica: só guarda o maior peso.
    /// Custo: O(grau(a) + grau(b)) por causa da checagem de duplicata.
    pub fn add_edge(&mut self, a: usize, b: usize, w: f64) {
        let mut existe = false;
        for e in self.adj[a].iter_mut() {
            if e.0 == b {
                if w > e.1 {
                    e.1 = w;
                }
                existe = true;
                break;
            }
        }
        if existe {
            // atualiza o lado contrário também, para o grafo continuar simétrico
            for e in self.adj[b].iter_mut() {
                if e.0 == a {
                    if w > e.1 {
                        e.1 = w;
                    }
                    break;
                }
            }
        } else {
            self.adj[a].push((b, w));
            self.adj[b].push((a, w)); // não direcionado: grava nos dois sentidos
            self.edge_count += 1;
        }
    }

    /// Remove a aresta a <-> b (nos dois sentidos). Retorna true se ela existia.
    /// Custo: O(grau(a) + grau(b)). `swap_remove` troca o item removido pelo último
    /// do Vec, evitando deslocar todos os elementos (a ordem dos vizinhos não importa).
    pub fn remove_edge(&mut self, a: usize, b: usize) -> bool {
        let pos_a = self.adj[a].iter().position(|e| e.0 == b);
        let Some(pa) = pos_a else {
            return false; // não existia aresta
        };
        self.adj[a].swap_remove(pa);
        if let Some(pb) = self.adj[b].iter().position(|e| e.0 == a) {
            self.adj[b].swap_remove(pb);
        }
        self.edge_count -= 1;
        true
    }

    pub fn node_count(&self) -> usize {
        self.keys.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edge_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aresta_e_simetrica() {
        let mut g = Graph::new();
        let a = g.get_or_add_node(NodeKey::Customer(1));
        let b = g.get_or_add_node(NodeKey::Product(1));
        g.add_edge(a, b, 0.8);
        assert_eq!(g.neighbors(a), &[(b, 0.8)]);
        assert_eq!(g.neighbors(b), &[(a, 0.8)]);
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn nao_duplica_aresta_e_mantem_maior_peso() {
        let mut g = Graph::new();
        let a = g.get_or_add_node(NodeKey::Customer(1));
        let b = g.get_or_add_node(NodeKey::Product(1));
        g.add_edge(a, b, 0.4);
        g.add_edge(a, b, 1.0);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.neighbors(a).len(), 1);
        assert_eq!(g.neighbors(a)[0].1, 1.0);
        assert_eq!(g.neighbors(b)[0].1, 1.0);
    }

    #[test]
    fn remove_aresta_nos_dois_sentidos() {
        let mut g = Graph::new();
        let a = g.get_or_add_node(NodeKey::Customer(1));
        let b = g.get_or_add_node(NodeKey::Product(1));
        g.add_edge(a, b, 0.8);
        assert!(g.remove_edge(a, b));
        assert!(g.neighbors(a).is_empty());
        assert!(g.neighbors(b).is_empty());
        assert_eq!(g.edge_count(), 0);
        assert!(!g.remove_edge(a, b)); // segunda remoção: já não existe
    }

    #[test]
    fn mesmo_no_nao_e_criado_duas_vezes() {
        let mut g = Graph::new();
        let a = g.get_or_add_node(NodeKey::Category("X".into()));
        let b = g.get_or_add_node(NodeKey::Category("X".into()));
        assert_eq!(a, b);
        assert_eq!(g.node_count(), 1);
    }
}

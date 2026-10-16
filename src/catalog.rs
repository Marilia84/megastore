use crate::product::Product;
use std::collections::HashMap;

/// Catálogo de produtos.
/// - `products`: id -> Product. Busca por id em O(1) médio.
/// - `by_category`: categoria -> lista de ids. Evita varrer o catálogo inteiro
///   quando queremos os produtos de uma categoria.
#[derive(Debug, Default)]
pub struct Catalog {
    products: HashMap<u32, Product>,
    by_category: HashMap<String, Vec<u32>>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default() // Default cria os dois HashMaps vazios
    }

    /// Cadastra um produto. Retorna Err se o id já existe
    /// (assim nunca temos dois produtos com o mesmo id).
    pub fn add(&mut self, product: Product) -> Result<(), String> {
        if self.products.contains_key(&product.id) {
            return Err(format!("Produto {} já cadastrado", product.id));
        }
        // entry(...).or_default() cria o Vec da categoria se ela ainda não existir
        self.by_category
            .entry(product.category.clone())
            .or_default()
            .push(product.id);
        self.products.insert(product.id, product);
        Ok(())
    }

    /// Consulta por id: O(1) médio. Retorna Option porque o id pode não existir.
    pub fn get(&self, id: u32) -> Option<&Product> {
        self.products.get(&id)
    }

    /// Ids dos produtos de uma categoria (slice vazio se a categoria não existir).
    pub fn ids_in_category(&self, category: &str) -> &[u32] {
        self.by_category.get(category).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Busca por trecho do nome (ignora maiúsculas/minúsculas).
    /// É O(n) porque varre todos os produtos: serve para consulta manual, não para o algoritmo.
    pub fn search_by_name(&self, term: &str) -> Vec<&Product> {
        let term = term.to_lowercase();
        let mut found: Vec<&Product> = self
            .products
            .values()
            .filter(|p| p.name.to_lowercase().contains(&term))
            .collect();
        found.sort_by_key(|p| p.id); // HashMap não tem ordem; ordenamos para o resultado ser estável
        found
    }

    /// Todos os produtos ordenados por id. O(n log n): usado só para exibição.
    pub fn list_all(&self) -> Vec<&Product> {
        let mut all: Vec<&Product> = self.products.values().collect();
        all.sort_by_key(|p| p.id);
        all
    }

    pub fn len(&self) -> usize {
        self.products.len()
    }

    pub fn is_empty(&self) -> bool {
        self.products.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadastra_e_consulta_produto() {
        let mut c = Catalog::new();
        c.add(Product::new(1, "Notebook", "Eletrônicos", 3500.0)).unwrap();
        assert_eq!(c.get(1).unwrap().name, "Notebook");
        assert!(c.get(99).is_none());
    }

    #[test]
    fn rejeita_id_duplicado() {
        let mut c = Catalog::new();
        c.add(Product::new(1, "A", "X", 1.0)).unwrap();
        assert!(c.add(Product::new(1, "B", "Y", 2.0)).is_err());
        assert_eq!(c.len(), 1);
    }

    #[test]
    fn lista_por_categoria_e_busca_por_nome() {
        let mut c = Catalog::new();
        c.add(Product::new(1, "Mouse Gamer", "Eletrônicos", 100.0)).unwrap();
        c.add(Product::new(2, "Teclado", "Eletrônicos", 150.0)).unwrap();
        c.add(Product::new(3, "Sofá", "Móveis", 2000.0)).unwrap();
        assert_eq!(c.ids_in_category("Eletrônicos"), &[1, 2]);
        assert!(c.ids_in_category("Livros").is_empty());
        assert_eq!(c.search_by_name("mouse").len(), 1);
        let ids: Vec<u32> = c.list_all().iter().map(|p| p.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }
}

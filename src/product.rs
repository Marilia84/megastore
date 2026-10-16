/// Um produto do catálogo da MegaStore.
/// `Clone` permite copiar o produto; `Debug` permite imprimir com {:?};
/// `PartialEq` permite comparar dois produtos nos testes (assert_eq!).
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: u32,           // identificador único (chave do HashMap)
    pub name: String,      // nome exibido ao cliente
    pub category: String,  // categoria (vira um vértice no grafo)
    pub price: f64,        // preço em reais
}

impl Product {
    /// Construtor: evita repetir `Product { ... }` em todo lugar.
    /// `impl Into<String>` aceita tanto &str quanto String.
    pub fn new(id: u32, name: impl Into<String>, category: impl Into<String>, price: f64) -> Self {
        Product {
            id,
            name: name.into(),
            category: category.into(),
            price,
        }
    }
}

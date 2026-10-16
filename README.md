# ConectaStore — Sistema de Recomendação de Produtos Baseado em Grafos

Projeto da disciplina **Data Structure Strategy and Implementation** (UniFECAF).
Sistema em **Rust** que recomenda produtos da MegaStore percorrendo um grafo de clientes, produtos e categorias.

## Objetivo e funcionamento
Em vez de mostrar só "os mais vendidos" ou "a mesma categoria", o sistema usa as **conexões** entre clientes, compras e categorias:

- Cliente que comprou o produto X também comprou Y → Y é recomendado (filtragem colaborativa).
- Produtos da mesma categoria de algo que o cliente comprou também entram, com peso menor.

## Tecnologias e estruturas
- Rust (somente biblioteca padrão, sem dependências externas)
- **Grafo não direcionado e ponderado**, com **lista de adjacência** (`Vec<Vec<(usize, f64)>>`)
- **HashMap**: catálogo por id, índice chave → vértice do grafo, distância e score durante a BFS
- **VecDeque** (fila FIFO): fila da BFS
- **Vec**: listas de vizinhos e ordenação final do ranking


## Compilar e executar
```bash
cargo build --release
cargo run --release          # menu interativo
cargo run --release -- bench # benchmark
```

## Testes
```bash
cargo test
```
Unitários ficam dentro de `src/` (módulos `#[cfg(test)]`); integração em `tests/integration_test.rs`.

## Arquitetura - recomendada pelo professor
```
megastore/
├── src/
│   ├── main.rs         # menu de terminal
│   ├── lib.rs          # expõe os módulos
│   ├── product.rs      # struct Product
│   ├── catalog.rs      # cadastro/consulta (HashMap)
│   ├── graph.rs        # grafo (lista de adjacência)
│   ├── recommender.rs  # BFS + score
│   └── bench.rs        # dados sintéticos e medição
├── tests/integration_test.rs
├── Cargo.toml
└── README.md
```
## Vídeo pitch
Link: https://youtu.be/yme59xjFPBM

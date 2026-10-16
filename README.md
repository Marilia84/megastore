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

| Vértice | Aresta | Peso |
|---|---|---|
| Cliente — Produto | compra | nota/5 (ou 0,6 sem nota) |
| Produto — Categoria | pertence a | 0,4 |

## Algoritmo
**BFS com fila (`VecDeque`) e acúmulo de score**:
1. Enfileira o cliente (ou produto) com distância 0 e score 1.
2. Tira da fila; para cada vizinho novo, guarda `distância + 1` e `score × peso` e enfileira. Se o vizinho já foi alcançado por outro caminho de mesmo comprimento, soma o score.
3. Não expande vértices na profundidade-alvo: 3 para cliente (cliente → produto → cliente/categoria → produto) e 2 para produto.
4. Só produtos na profundidade-alvo são candidatos. Ordena por score (maior primeiro) e devolve os `limit` melhores.

Como a distância de cada vértice fica registrada no `HashMap`, ninguém é enfileirado duas vezes, e os produtos já comprados estão a distância 1, **não há recomendação duplicada nem de item já comprado**.

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

## Exemplo de uso
No menu: `7` (carregar exemplo) → `8` (listar produtos) → `9` com cliente `100` (ver compras) → `5` com cliente `100` (recomendar) → `10` (remover uma compra) → `5` de novo para ver a mudança.

## Arquitetura
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

## Resultados de desempenho
> Substitua pela saída do SEU `cargo run --release -- bench` (varia conforme a máquina).

| Produtos | Clientes | Vértices | Arestas | Build (ms) | Recomendação (µs) |
|---|---|---|---|---|---|
| 1.000 | 500 | 1.510 | 3.495 | 0,9 | 148 |
| 10.000 | 5.000 | 15.100 | 34.993 | 23,3 | 203 |
| 100.000 | 50.000 | 151.000 | 349.996 | 255,8 | 198 |
| 500.000 | 250.000 | 755.000 | 1.749.996 | 1.919 | 180 |

O tempo de recomendação fica praticamente constante porque o custo depende da **vizinhança** do cliente, não do tamanho do catálogo.
Uma matriz de adjacência com 755 mil vértices exigiria ~4,5 TB (V² × 8 bytes), por isso a lista de adjacência.

## Vídeo pitch
Link: _(coloque aqui o link do YouTube)_

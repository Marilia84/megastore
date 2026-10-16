use megastore::bench;
use megastore::product::Product;
use megastore::recommender::Recommender;
use std::io::{self, Write};

/// Lê uma linha do teclado, já sem espaços/enter nas pontas.
fn ler(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap(); // garante que o prompt aparece antes de esperar a digitação
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}

/// Lê um número; retorna None se o texto não for número válido.
fn ler_num<T: std::str::FromStr>(prompt: &str) -> Option<T> {
    ler(prompt).parse().ok()
}

fn mostrar_recs(r: &Recommender, recs: &[megastore::recommender::Recommendation]) {
    if recs.is_empty() {
        println!("Nenhuma recomendação encontrada.");
        return;
    }
    for (i, rec) in recs.iter().enumerate() {
        if let Some(p) = r.catalog().get(rec.product_id) {
            println!("{}. [{}] {} ({}) - R$ {:.2} | score {:.3}", i + 1, p.id, p.name, p.category, p.price, rec.score);
        }
    }
}

fn carregar_exemplo(r: &mut Recommender) {
    let produtos = [
        (1, "Notebook Gamer", "Eletrônicos", 5200.0),
        (2, "Mouse Sem Fio", "Eletrônicos", 90.0),
        (3, "Teclado Mecânico", "Eletrônicos", 250.0),
        (4, "Livro: Rust na Prática", "Livros", 110.0),
        (5, "Livro: Estruturas de Dados", "Livros", 130.0),
        (6, "Cadeira Ergonômica", "Móveis", 900.0),
        (7, "Mesa de Escritório", "Móveis", 700.0),
        (8, "Luminária de Mesa", "Decoração", 120.0),
    ];
    for (id, nome, cat, preco) in produtos {
        let _ = r.add_product(Product::new(id, nome, cat, preco));
    }
    // (cliente, produto, nota)
    let compras = [
        (100, 1, 5), (100, 4, 5), (100, 6, 4),
        (200, 1, 5), (200, 2, 4), (200, 3, 5),
        (300, 4, 5), (300, 5, 5), (300, 7, 3),
        (400, 6, 5), (400, 7, 5), (400, 8, 4),
    ];
    for (c, p, n) in compras {
        let _ = r.register_purchase(c, p, Some(n));
    }
    println!("Dados de exemplo carregados: 8 produtos, 4 clientes (100, 200, 300, 400).");
}

fn menu() {
    println!("\n=== ConectaStore ===");
    println!("1) Cadastrar produto");
    println!("2) Consultar produto por id");
    println!("3) Buscar produto por nome");
    println!("4) Registrar compra");
    println!("5) Recomendar para cliente");
    println!("6) Recomendar a partir de produto");
    println!("7) Carregar dados de exemplo");
    println!("8) Listar todos os produtos");
    println!("9) Ver compras de um cliente");
    println!("10) Remover compra");
    println!("0) Sair");
}

fn main() {
    // `cargo run --release -- bench` roda só o benchmark
    if std::env::args().nth(1).as_deref() == Some("bench") {
        bench::run();
        return;
    }

    let mut r = Recommender::new();
    loop {
        menu();
        match ler("Opção: ").as_str() {
            "1" => {
                let (Some(id), nome, cat, Some(preco)) = (
                    ler_num::<u32>("Id: "),
                    ler("Nome: "),
                    ler("Categoria: "),
                    ler_num::<f64>("Preço: "),
                ) else {
                    println!("Id ou preço inválido.");
                    continue;
                };
                match r.add_product(Product::new(id, nome, cat, preco)) {
                    Ok(()) => println!("Produto cadastrado!"),
                    Err(e) => println!("Erro: {}", e),
                }
            }
            "2" => match ler_num::<u32>("Id: ").and_then(|id| r.catalog().get(id)) {
                Some(p) => println!("{:?}", p),
                None => println!("Produto não encontrado."),
            },
            "3" => {
                let achados = r.catalog().search_by_name(&ler("Trecho do nome: "));
                if achados.is_empty() {
                    println!("Nada encontrado.");
                }
                for p in achados {
                    println!("[{}] {} ({}) - R$ {:.2}", p.id, p.name, p.category, p.price);
                }
            }
            "4" => {
                let (Some(c), Some(p)) = (ler_num::<u32>("Id do cliente: "), ler_num::<u32>("Id do produto: ")) else {
                    println!("Id inválido.");
                    continue;
                };
                let nota = ler_num::<u8>("Nota 1-5 (enter para pular): ");
                match r.register_purchase(c, p, nota) {
                    Ok(()) => println!("Compra registrada!"),
                    Err(e) => println!("Erro: {}", e),
                }
            }
            "5" => {
                if let Some(c) = ler_num::<u32>("Id do cliente: ") {
                    mostrar_recs(&r, &r.recommend_for_customer(c, 5));
                }
            }
            "6" => {
                if let Some(p) = ler_num::<u32>("Id do produto: ") {
                    mostrar_recs(&r, &r.recommend_for_product(p, 5));
                }
            }
            "7" => carregar_exemplo(&mut r),
            "8" => {
                let todos = r.catalog().list_all();
                if todos.is_empty() {
                    println!("Catálogo vazio.");
                }
                for p in todos {
                    println!("[{}] {} ({}) - R$ {:.2}", p.id, p.name, p.category, p.price);
                }
            }
            "9" => {
                if let Some(c) = ler_num::<u32>("Id do cliente: ") {
                    let compras = r.customer_purchases(c);
                    if compras.is_empty() {
                        println!("Cliente sem compras (ou inexistente).");
                    }
                    for (pid, peso) in compras {
                        if let Some(p) = r.catalog().get(pid) {
                            println!("[{}] {} - peso da conexão {:.2}", p.id, p.name, peso);
                        }
                    }
                }
            }
            "10" => {
                let (Some(c), Some(p)) = (ler_num::<u32>("Id do cliente: "), ler_num::<u32>("Id do produto: ")) else {
                    println!("Id inválido.");
                    continue;
                };
                match r.remove_purchase(c, p) {
                    Ok(()) => println!("Compra removida!"),
                    Err(e) => println!("Erro: {}", e),
                }
            }
            "0" => break,
            _ => println!("Opção inválida."),
        }
    }
}

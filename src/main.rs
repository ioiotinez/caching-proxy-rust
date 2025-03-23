use clap::Parser;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use reqwest::Client;

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "8080")]
    port: u16,

    #[arg(short, long)]
    origin: String,
}


#[get("/{path:.*}")]
async fn proxy (
    path: web::Path<String>, 
    origin: web::Data<String>, 
    client: web::Data<String>
) -> impl Responder {
    let url = format!("{}/{}", origin.get_ref(), path);
    
    if let Some(value) = CACHE.lock().unwrap().get(&url) {
        return HttpResponse::Ok().body(value.clone());
    }

    let response = client.get(&url).send().await; 

    match response {
        Ok(res) => {
            let body = res.text().await.unwrap();
            CACHE.lock().unwrap().insert(url.clone(), body.clone());
            HttpResponse::Ok().body(body)
        },
        Err(_) => {
            HttpResponse::InternalServerError().body("Error")
        }
    }

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    println!("Port: {}", args.port);
    println!("Origin: {}", args.origin);

    let client = Client::new();

    // Run the server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(args.origin.clone())) // Pasar la URL de origen como dato compartido
            .app_data(web::Data::new(client.clone())) // Pasar el cliente como dato compartido
            .service(proxy) // Registrar el manejador de proxy
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}

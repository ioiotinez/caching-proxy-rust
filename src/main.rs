use clap::Parser;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

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
    origin: web::Data<String>
) -> impl Responder {

    let path = path.into_inner();

    if path == "favicon.ico" {
        return HttpResponse::NotFound().body(""); // Respuesta vacía
    }

    let url = format!("{}/{}", origin.get_ref(), path);
    
    if let Some(value) = CACHE.lock().unwrap().get(&url) {
        return HttpResponse::Ok()
                    .insert_header(("X-Cache", "HIT"))
                    .body(value.clone());
    }

    println!("Requesting: {}", url);
    println!("Origin: {}", origin.get_ref());

    let response = reqwest::get(&url).await;

    match response {
        Ok(res) => {
            let body = res.text().await.unwrap();
            CACHE.lock().unwrap().insert(url.clone(), body.clone());
            HttpResponse::Ok()
                            .insert_header(("X-Cache", "MISS"))
                            .body(body)
        },
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    println!("Port: {}", args.port);
    println!("Origin: {}", args.origin);

    // Run the server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(args.origin.clone())) // Pasar la URL de origen como dato compartido
            .service(proxy) // Registrar el manejador de proxy
    })
    .bind(("127.0.0.1", args.port))?
    .run()
    .await
}

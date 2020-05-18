 use actix_web::{App, HttpServer};

 mod public_transaction;
 mod internal_meta;

 pub mod response;

 #[actix_rt::main]
 async fn main() -> std::io::Result<()>{
     HttpServer::new( || {
         App::new() 
         .service(public_transaction::digital_transaction)
         .service(internal_meta::digital_meta)
     })
     .bind("127.0.0.1:8808")?
     .run()
     .await 
 }

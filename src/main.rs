use actix_web::{App, HttpServer};
use log::Level;

mod admin_meta;
mod config;
mod internal_meta;
mod public_transaction;
mod quotas_request;

pub mod response;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //Initialize the log and set the print level
    simple_logger::init_with_level(Level::Warn).unwrap();

    HttpServer::new(|| {
        App::new()
            .data(config::get_db())
            .data(config::ConfigPath::default())
            .service(admin_meta::new_cert)
            .service(admin_meta::update_cert)
            .service(admin_meta::get_cert)
            .service(public_transaction::digital_transaction)
            .service(internal_meta::digital_meta)
            .service(quotas_request::new_quotas_request)
            .service(admin_meta::register_cms)
    })
    .bind("127.0.0.1:8808")?
    .run()
    .await
}

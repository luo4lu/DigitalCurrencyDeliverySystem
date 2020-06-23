use actix_web::{App, HttpServer};
use log::Level;
mod admin_meta;
mod config;
mod internal_meta;
mod public_transaction;
mod quotas_request;
mod config_command;
mod get_quota_info;
use clap::ArgMatches;

pub mod response;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //Initialize the log and set the print level
    simple_logger::init_with_level(Level::Warn).unwrap();
    let mut path:String = String::new();
    let matches: ArgMatches = config_command::get_command();
    if let Some(d) = matches.value_of("dcds"){
        path = d.to_string();
    }else{
        path = String::from("127.0.0.1:8888");
    }
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
            .service(quotas_request::conver_currency)
            .service(admin_meta::register_cms)
            .service(internal_meta::amount_exchange)
            .service(internal_meta::currency_widthdraw)
            .service(get_quota_info::get_all_quota_info)
    })
    .bind(path)?
    .run()
    .await
}

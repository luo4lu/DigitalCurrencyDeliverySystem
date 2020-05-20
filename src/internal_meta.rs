use crate::response::ResponseBody;
use actix_web::{post, web, HttpResponse, Responder};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)] //反序列化请求字段
pub struct InternalMetaRequest{
    quota : String,
    target : String,
    receive : String,
}    

#[derive(Serialize)] //序列化响应字段
pub struct InternalMetaRespones{
    receipt : String,
    coin : String,
}

#[post("/api/internal/meta")]
pub async fn digital_meta(req: web::Json<InternalMetaRequest>) -> impl Responder{
    format!("request infomation : {:?}",req);
    let receipt = String::from("transaction receipt");
    let coin = String::from("name");
    HttpResponse::Ok().json(ResponseBody::new_success(Some(InternalMetaRespones{
        receipt,
        coin,
    })))
}
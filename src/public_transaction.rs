use crate::response::ResponseBody;
use actix_web::{post, web, HttpResponse, Responder};

use serde::{Deserialize, Serialize};

#[derive(Deserialize,Debug)]  //反序列化请求字段
pub struct  TransactionRequest{
    transaction : String,
}

#[derive(Serialize)]  //序列化响应字段
pub struct TransactionResponse{
    receipt : String,
    coin : String,
}

//交易请求-----》return receipt
#[post("/api/public/transaction")]
pub async fn digital_transaction(req : web::Json<TransactionRequest> ) -> impl Responder{
    format!("request infomation : {:?}",req);
    let receipt = String::from("transaction receipt");
    let coin = String::from("name");
    HttpResponse::Ok().json(ResponseBody::new_success(Some(TransactionResponse{
        receipt,
        coin,
    })))
}

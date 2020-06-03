use crate::config::ConfigPath;
use crate::response::ResponseBody;
use actix_web::{post, web, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use common_structure::digital_currency::{DigitalCurrency, DigitalCurrencyWrapper};
use common_structure::quota_control_field::QuotaControlFieldWrapper;
use common_structure::transaction::{Transaction, TransactionWrapper};
use dislog_hal::Bytes;
use hex::{FromHex, ToHex};
use kv_object::kv_object::MsgType;
use kv_object::prelude::KValueObject;
use kv_object::sm2::CertificateSm2;
use kv_object::sm2::KeyPairSm2;
use log::{info, warn};
use rand::thread_rng;
use serde::Deserialize;
use tokio::fs::File;
use tokio::prelude::*;
//数据库相关
use deadpool_postgres::Pool;
#[derive(Deserialize, Debug)] //反序列化请求字段
pub struct InternalMetaRequest {
    quota: String,  //Base64过的额度控制位
    target: String, //货币接收者的身份标识
}

#[post("/api/internal/meta")]
pub async fn digital_meta(
    data: web::Data<Pool>,
    config: web::Data<ConfigPath>,
    req: web::Json<Vec<InternalMetaRequest>>,
) -> impl Responder {
    let mut rng = thread_rng();
    //read file for get seed
    let mut file = match File::open(&config.meta_path).await {
        Ok(f) => {
            info!("{:?}", f);
            f
        }
        Err(e) => {
            warn!("file open failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_file_error());
        }
    };
    //read json file to string
    let mut contents = String::new();
    match file.read_to_string(&mut contents).await {
        Ok(s) => {
            info!("{:?}", s);
            s
        }
        Err(e) => {
            warn!("read file to string failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    };
    //deserialize to the specified data format
    let keypair_value: keypair::Keypair<
        [u8; 32],
        Sha3,
        dislog_hal_sm2::PointInner,
        dislog_hal_sm2::ScalarInner,
    > = match serde_json::from_str(&contents) {
        Ok(de) => {
            info!("{:?}", de);
            de
        }
        Err(e) => {
            warn!("Keypair generate failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    };
    //pass encode hex conversion get seed
    let seed: [u8; 32] = keypair_value.get_seed();
    //get  digital signature
    let keypair_sm2: KeyPairSm2 = KeyPairSm2::generate_from_seed(seed).unwrap();
    //钱包新的所有者证书
    let wallet_cert_2 = keypair_sm2.get_certificate();
    //存储到数据库
    let conn = data.get().await.unwrap();
    //存储生成新的货币
    let mut currency: Vec<String> = Vec::new();
    //存储整个交易体数据
    let mut digital: Vec<String> = Vec::new();
    for (_index, value) in req.iter().enumerate() {
        let quota_vec = Vec::<u8>::from_hex(value.quota.clone()).unwrap();
        let mut quota_control_field = QuotaControlFieldWrapper::from_bytes(&quota_vec).unwrap();
        let target_vec = Vec::<u8>::from_hex(value.target.clone()).unwrap();
        let target = CertificateSm2::from_bytes(&target_vec).unwrap();

        quota_control_field
            .fill_kvhead(&keypair_sm2, &mut rng)
            .unwrap();
        let mut digital_currency = DigitalCurrencyWrapper::new(
            MsgType::DigitalCurrency,
            DigitalCurrency::new(quota_control_field, target.clone()),
        );
        digital_currency
            .fill_kvhead(&keypair_sm2, &mut rng)
            .unwrap();
        let quota_control_field2 = digital_currency.get_body().get_quota_info();
        let quote_hex = quota_control_field2.to_bytes().encode_hex::<String>();
        let wallet_cert = digital_currency.get_body().get_wallet_cert();
        let wallet_hex = wallet_cert.to_bytes().encode_hex::<String>();

        let transaction_ok = TransactionWrapper::new(
            MsgType::Transaction,
            Transaction::new(wallet_cert_2.clone(), digital_currency.clone()),
        );
        //新的新的货币
        let pre_currency = transaction_ok
            .get_body()
            .trans_currency(&keypair_sm2)
            .unwrap();
        //验证签名
        if pre_currency.verfiy_kvhead().is_ok() {
            info!("true");
        } else {
            warn!("transcation signature verfiy check failed");
            return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
        }
        currency.push(pre_currency.to_bytes().encode_hex::<String>());
        //----------------------------------------------------------
        //输出测试数据
        let digital_hex = digital_currency.to_bytes().encode_hex::<String>();
        digital.push(digital_hex);
        //----------------------------------------------------------

        //get quota control field id
        let id = (*quota_control_field2.get_body().get_id()).encode_hex::<String>();
        //state
        let state: String = String::from("circulation");
        let jsonb_quota = serde_json::to_value(&quota_control_field2).unwrap();

        let statement = conn.prepare("INSERT INTO digital_currency (id, quota_control_field, explain_info, 
                    state, owner, create_time, update_time) VALUES ($1, $2, $3, $4, $5, now(), now())")
            .await
            .unwrap();
        conn.execute(
            &statement,
            &[&id, &quote_hex, &jsonb_quota, &state, &wallet_hex],
        )
        .await
        .unwrap();
    }
    println!("{:?}", digital);
    HttpResponse::Ok().json(ResponseBody::new_success(Some(currency)))
}

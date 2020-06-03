use crate::config::ConfigPath;
use crate::response::ResponseBody;
use actix_web::{post, web, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use common_structure::digital_currency::DigitalCurrencyWrapper;
use common_structure::transaction::{Transaction, TransactionWrapper};
use dislog_hal::Bytes;
use hex::{FromHex, ToHex};
use kv_object::kv_object::MsgType;
use kv_object::prelude::KValueObject;
//use kv_object::sm2::CertificateSm2;
use kv_object::sm2::KeyPairSm2;
use log::{info, warn};
use rand::thread_rng;
use tokio::fs::File;
use tokio::prelude::*;
//数据库相关
use deadpool_postgres::Pool;

//交易请求-----》return receipt
#[post("/api/public/transaction")]
pub async fn digital_transaction(
    data: web::Data<Pool>,
    config: web::Data<ConfigPath>,
    req: web::Json<Vec<String>>,
) -> impl Responder {
    let mut _rng = thread_rng();
    //read file for get seed(需要更改拥有者，在接口调用之前需要先产生一个新的额度控制元信息)
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
    //新的货币所有者存储
    let mut currency: Vec<String> = Vec::new();
    //存储到数据库
    let conn = data.get().await.unwrap();

    for (_index, value) in req.iter().enumerate() {
        let vec = Vec::<u8>::from_hex(value.clone()).unwrap();
        let old_currency = DigitalCurrencyWrapper::from_bytes(&vec).unwrap();
        let old_quota_control = (old_currency.get_body().get_quota_info())
            .to_bytes()
            .encode_hex::<String>();
        let transaction_ok = TransactionWrapper::new(
            MsgType::Transaction,
            Transaction::new(wallet_cert_2.clone(), old_currency.clone()),
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
        //获取所有者
        let wallet_cert = pre_currency.get_body().get_wallet_cert();
        let wallet_hex = wallet_cert.to_bytes().encode_hex::<String>();
        //存储为响应数据
        currency.push(pre_currency.to_bytes().encode_hex::<String>());
        let statement = conn
            .prepare("UPDATE digital_currency SET owner = $1,update_time = now() WHERE quota_control_field = $2")
            .await
            .unwrap();
        conn.execute(&statement, &[&wallet_hex, &old_quota_control])
            .await
            .unwrap();
    }

    HttpResponse::Ok().json(ResponseBody::new_success(Some(currency)))
}

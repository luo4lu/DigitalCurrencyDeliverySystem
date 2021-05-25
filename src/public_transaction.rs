use crate::config::ConfigPath;
use crate::response::ResponseBody;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use common_structure::transaction::TransactionWrapper;
use dislog_hal::Bytes;
use hex::{FromHex, ToHex};
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
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-USERID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //随机数生成器
    let mut _rng = thread_rng();
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
    //新的货币所有者存储
    let mut currency: Vec<String> = Vec::new();
    //存储到数据库
    let conn = data.get().await.unwrap();

    for (_index, value) in req.iter().enumerate() {
        let vec = Vec::<u8>::from_hex(value.clone()).unwrap();
        let transaction_ok = TransactionWrapper::from_bytes(&vec).unwrap();
        //新的新的货币
        let pre_currency = transaction_ok
            .get_body()
            .trans_currency(&keypair_sm2)
            .unwrap();
        let old_quota_control = (pre_currency.get_body().get_quota_info())
            .to_bytes()
            .encode_hex::<String>();
        //获取所有者
        let wallet_cert = pre_currency.get_body().get_wallet_cert();
        let wallet_hex = wallet_cert.to_bytes().encode_hex::<String>();
        //存储为响应数据
        currency.push(pre_currency.to_bytes().encode_hex::<String>());
        let select_state = match conn
            .query(
                "SELECT id from digital_currency where quota_control_field = $1 AND cloud_user_id = $2",
                &[&old_quota_control, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if select_state.is_empty() {
            warn!("SELECT check quota_control_field failed,please check quota_control_field value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
        let id: String = select_state[0].get(0);
        let statement = match conn
            .prepare("UPDATE digital_currency SET owner = $1,update_time = now() WHERE quota_control_field = $2 AND cloud_user_id = $3")
            .await{
                Ok(s) => {
                    info!("database command success!");
                    s
                }
                Err(error) =>{
                    warn!("database command failed: {:?}",error);
                    return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
                }
            };
        match conn
            .execute(&statement, &[&wallet_hex, &old_quota_control, &head_str])
            .await
        {
            Ok(s) => {
                info!("database parameter success!");
                s
            }
            Err(error) => {
                warn!("database parameter failed: {:?}", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        match conn
            .query(
                "INSERT INTO transaction_history (id, owner, cloud_user_id, create_time) VALUES ($1, $2, $3, now())",
                &[&id, &wallet_hex, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("INSERT success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("INSERT failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
    }

    HttpResponse::Ok().json(ResponseBody::new_success(Some(currency)))
}

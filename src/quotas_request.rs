use crate::config::ConfigPath;
use crate::response::ResponseBody;
use actix_web::{post, web, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use common_structure::digital_currency::{DigitalCurrency, DigitalCurrencyWrapper};
use common_structure::issue_quota_request::{IssueQuotaRequest, IssueQuotaRequestWrapper};
use common_structure::quota_control_field::QuotaControlFieldWrapper;
use dislog_hal::Bytes;
use hex::{FromHex, ToHex};
use kv_object::kv_object::MsgType;
use kv_object::prelude::KValueObject;
//use kv_object::sm2::CertificateSm2;
use kv_object::sm2::KeyPairSm2;
use log::{info, warn};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::prelude::*;
//数据库相关
use deadpool_postgres::Pool;

#[derive(Serialize, Debug)] //反序列化请求字段
pub struct CentralRequest {
    issue: String,
}
impl CentralRequest {
    pub fn new(issue: String) -> Self {
        Self { issue }
    }
}
#[derive(Serialize, Debug)]
pub struct QuotaRequest {
    issue_quota_request: String,
}
impl QuotaRequest {
    pub fn new(issue: String) -> Self {
        Self {
            issue_quota_request: issue,
        }
    }
}
#[derive(Deserialize, Debug)] //反序列化请求字段
pub struct QuotasRequest {
    value: u64,
    number: u64,
}
//构造中心管理系统重新签名额度请求响应结构体
#[derive(Deserialize, Debug)]
pub struct IssueResponse {
    pub code: i32,
    pub message: String,
    pub data: String,
}

//构造额度管理系统生成额度控制位响应结构体
#[derive(Deserialize, Debug)]
pub struct QuotaResponse {
    pub code: i32,
    pub message: String,
    pub data: Vec<String>,
}

#[post("/api/quotas")]
pub async fn new_quotas_request(
    data: web::Data<Pool>,
    config: web::Data<ConfigPath>,
    req: web::Json<Vec<QuotasRequest>>,
) -> impl Responder {
    let mut rng = thread_rng();
    //连接数据库
    let conn = data.get().await.unwrap();
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
    //获取签名证书
    let cert_sm2 = keypair_sm2.get_certificate();
    let mut issue_info = Vec::<(u64, u64)>::new();
    for info in req.iter() {
        issue_info.push((info.value, info.number));
    }

    //发行请求,生成发行信息
    let mut issue = IssueQuotaRequestWrapper::new(
        MsgType::IssueQuotaRequest,
        IssueQuotaRequest::new(issue_info, keypair_sm2.get_certificate()),
    );
    issue.fill_kvhead(&keypair_sm2, &mut rng).unwrap();
    //验证签名
    if issue.verfiy_kvhead().is_ok() {
        info!("true");
    } else {
        warn!("quota issue request verfiy check failed");
        return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
    }
    let issue_hex = issue.to_bytes().encode_hex::<String>();

    //Http请求中心管理系统对额度请求重新签名
    let params = CentralRequest::new(issue_hex);
    let cmc_url = String::from("http://localhost:8077/api/dcds/qouta_issue");
    let cmc_client = reqwest::Client::new();
    let cmc_res = cmc_client
        .post(&cmc_url)
        .json(&params)
        .send()
        .await
        .unwrap();
    let cmc_response: IssueResponse = cmc_res.json().await.unwrap();

    //Http请求额度管理系统生成额度控制位
    let repeat_issue = cmc_response.data;
    let params_to = QuotaRequest::new(repeat_issue);
    let qms_url = String::from("http://localhost:8088/api/quota");
    let qms_client = reqwest::Client::new();
    let qms_res = qms_client
        .post(&qms_url)
        .json(&params_to)
        .send()
        .await
        .unwrap();
    let qms_response: QuotaResponse = qms_res.json().await.unwrap();
    let queta_control_vec = qms_response.data;

    //组建支付货币列表
    for (_index, quota) in queta_control_vec.iter().enumerate() {
        let deser_vec = Vec::<u8>::from_hex(&quota).unwrap();
        let mut quota_control_field = QuotaControlFieldWrapper::from_bytes(&deser_vec).unwrap();
        //额度控制位签名
        quota_control_field
            .fill_kvhead(&keypair_sm2, &mut rng)
            .unwrap();
        //验证签名
        if quota_control_field.verfiy_kvhead().is_ok() {
            info!("true");
        } else {
            warn!("quota issue request verfiy check failed");
            return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
        }
        //生成数字货币
        let mut digital_currency = DigitalCurrencyWrapper::new(
            MsgType::DigitalCurrency,
            DigitalCurrency::new(quota_control_field, cert_sm2.clone()),
        );
        //数字货币签名
        digital_currency
            .fill_kvhead(&keypair_sm2, &mut rng)
            .unwrap();
        //验证签名
        if digital_currency.verfiy_kvhead().is_ok() {
            info!("true");
        } else {
            warn!("quota issue request verfiy check failed");
            return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
        }
        //获取数据库存储各个字段信息
        let quota_control_field2 = digital_currency.get_body().get_quota_info();
        let quota_hex = quota_control_field2.to_bytes().encode_hex::<String>();
        let id = (*quota_control_field2.get_body().get_id()).encode_hex::<String>();

        let wallet_cert = digital_currency.get_body().get_wallet_cert();
        let wallet_hex = wallet_cert.to_bytes().encode_hex::<String>();
        let state: String = String::from("circulation");
        let jsonb_quota = serde_json::to_value(&quota_control_field2).unwrap();
        //插入数据库语句
        let insert_statement = match conn
            .prepare(
                "INSERT INTO digital_currency (id, quota_control_field, explain_info, 
                state, owner, create_time, update_time) VALUES ($1, $2, $3, $4, $5, now(), now())",
            )
            .await
        {
            Ok(s) => {
                info!("database command success!");
                s
            }
            Err(error) => {
                warn!("database command failed: {:?}", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        match conn
            .execute(
                &insert_statement,
                &[&id, &quota_hex, &jsonb_quota, &state, &wallet_hex],
            )
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
    }

    HttpResponse::Ok().json(ResponseBody::<()>::new_success(None))
}

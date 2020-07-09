use crate::config::{CMSRequestAddr, ConfigPath, QCSRequestAddr};
use crate::response::ResponseBody;
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use common_structure::currency_convert_request::CurrencyConvertRequestWrapper;
use common_structure::digital_currency::{DigitalCurrency, DigitalCurrencyWrapper};
use common_structure::issue_quota_request::{IssueQuotaRequest, IssueQuotaRequestWrapper};
use common_structure::quota_control_field::QuotaControlFieldWrapper;
use dislog_hal::Bytes;
use hex::{FromHex, ToHex};
use kv_object::kv_object::MsgType;
use kv_object::prelude::KValueObject;
use kv_object::sm2::CertificateSm2;
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
    req_head: HttpRequest,
) -> impl Responder {
    let mut rng = thread_rng();
    //连接数据库
    let conn = data.get().await.unwrap();
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    let head_name: &str = &*String::from("X-CLOUD-USER_ID");
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
    warn!("中心管理系统货币开始注册\n");

    let central_request = CMSRequestAddr::new();
    let params = CentralRequest::new(issue_hex);
    let cmc_client = reqwest::Client::new();
    let cmc_res = cmc_client
        .post(&central_request.central_addr)
        .header(head_name, head_str)
        .json(&params)
        .send()
        .await
        .unwrap();
    let cmc_response: IssueResponse = cmc_res.json().await.unwrap();
    warn!("中心管理系统货币注册完成\n");

    //Http请求额度管理系统生成额度控制位
    warn!("开始数字货币额度管理系统申请额度\n");
    let quota_request = QCSRequestAddr::new();
    let repeat_issue = cmc_response.data;
    let params_to = QuotaRequest::new(repeat_issue);
    let qms_client = reqwest::Client::new();
    let qms_res = qms_client
        .post(&quota_request.quota_addr)
        .header(head_name, head_str)
        .json(&params_to)
        .send()
        .await
        .unwrap();
    let qms_response: QuotaResponse = qms_res.json().await.unwrap();
    let queta_control_vec = qms_response.data;
    warn!("数字货币额度管理系统申请完成\n");
    //组建支付货币列表
    warn!("货币发行系统开始生成数字货币\n");
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

        //获取数据库存储各个字段信息
        let quota_hex = quota_control_field.to_bytes().encode_hex::<String>();
        let id = (*quota_control_field.get_body().get_id()).encode_hex::<String>();

        let state: String = String::from("suspended");
        let jsonb_quota = serde_json::to_value(&quota_control_field).unwrap();
        //插入数据库语句
        let insert_statement = match conn
            .prepare(
                "INSERT INTO digital_currency (id, quota_control_field, explain_info, 
                state, cloud_user_id, create_time, update_time) VALUES ($1, $2, $3, $4, $5, now(), now())",
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
                &[&id, &quota_hex, &jsonb_quota, &state, &head_str],
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
    warn!("数字货币生成完成！！\n");

    HttpResponse::Ok().json(ResponseBody::<()>::new_success(None))
}

//兑换数字货币
#[post("/api/convert")]
pub async fn conver_currency(
    data: web::Data<Pool>,
    config: web::Data<ConfigPath>,
    req: web::Json<String>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库句柄
    let conn = data.get().await.unwrap();
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
    //解析hex出数据结构
    let temp = Vec::<u8>::from_hex(req.clone()).unwrap();
    let currency = CurrencyConvertRequestWrapper::from_bytes(&temp).unwrap();
    let input_digital = currency.get_body().get_inputs();
    let output_currency = currency.get_body().get_outputs();
    let old_state = String::from("circulation");
    let new_state = String::from("suspended");
    let mut old_sum: u64 = 0;
    let mut new_sum: u64 = 0;
    let mut wallet_hex = String::new();
    //兑换前后额度对比
    for num in input_digital.iter() {
        let quota_control_field = num.get_body().get_quota_info();
        //兑换前的货币总和
        old_sum += quota_control_field.get_body().get_value();
    }
    for (quota, number) in output_currency.iter() {
        //转换后的额度总和
        new_sum += quota * number;
        //发行系统中面值额度数量查询
        let str_quota = serde_json::to_value(&quota).unwrap();
        let size_number = *number as usize;
        let select_statement = match conn
            .query("select id, quota_control_field from digital_currency where state = $1 AND (explain_info->'t_obj'->'value') = $2 AND cloud_user_id = $3",
        &[&new_state, &str_quota, &head_str]).await{
            Ok(row) => {
                info!("select success!{:?}", row);
                row
            }
            Err(error) => {
                warn!("conver_currency select failde!!{:?}", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
            }
        };
        if size_number > select_statement.len() {
            warn!("request convert currency number too many,Lack of money!!!!");
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    }
    if old_sum != new_sum {
        warn!(
            "The amount before and after conversion is not equal,input:{} != output:{}",
            old_sum, new_sum
        );
        return HttpResponse::Ok().json(ResponseBody::<()>::currency_convert_error());
    }
    //验证老的数字货币正确性
    for (_index, value) in input_digital.iter().enumerate() {
        //签名验证
        if value.verfiy_kvhead().is_ok() {
            info!("true");
        } else {
            warn!("quota issue request verfiy check failed");
            return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
        }
        //验证字段
        let quota_control_field = value.get_body().get_quota_info();
        let quota_hex = quota_control_field.to_bytes().encode_hex::<String>();
        let wallet_cert = value.get_body().get_wallet_cert();
        wallet_hex = wallet_cert.to_bytes().encode_hex::<String>();
        let select_state = match conn
            .query(
                "SELECT * from digital_currency where quota_control_field = $1 AND state = $2 AND owner = $3 AND cloud_user_id = $4",
                &[&quota_hex, &old_state,&wallet_hex,&head_str],
            ).await{
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
            warn!("SELECT check digital_currency failed,please check digital_currency value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
        let statement = match conn
            .prepare("UPDATE digital_currency SET state = $1, owner = NULL,update_time = now() WHERE quota_control_field = $2 AND cloud_user_id = $3")
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
            .execute(&statement, &[&new_state, &quota_hex, &head_str])
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
    //生成数字货币的钱包信息
    let target_vec = Vec::<u8>::from_hex(wallet_hex.clone()).unwrap();
    let target = CertificateSm2::from_bytes(&target_vec).unwrap();
    //存储数字货币
    let mut new_digital_currency: Vec<String> = Vec::new();
    for (quota, number) in output_currency.iter() {
        let str_quota = serde_json::to_value(&quota).unwrap();
        let size_number = *number as usize;
        let select_statement = match conn
            .query("select id, quota_control_field from digital_currency where state = $1 AND (explain_info->'t_obj'->'value') = $2 AND cloud_user_id = $3",
        &[&new_state, &str_quota, &head_str]).await{
            Ok(row) => {
                info!("select success!{:?}", row);
                row
            }
            Err(error) => {
                warn!("conver_currency select failde!!{:?}", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
            }
        };
        if select_statement.is_empty() {
            warn!("conver_currency SELECT check uid failed,please check uid value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }

        for item in select_statement.iter().take(size_number) {
            let id: String = item.get(0);
            let quota_hex: String = item.get(1);
            let quota_vec = Vec::<u8>::from_hex(quota_hex).unwrap();
            let quota_control_field = QuotaControlFieldWrapper::from_bytes(&quota_vec).unwrap();
            match conn.query("UPDATE digital_currency SET state = $1,owner = $2,update_time = now() where id = $3 AND cloud_user_id = $4", 
            &[&old_state, &wallet_hex, &id, &head_str])
            .await
            {
                Ok(row) => {
                    info!("update success!{:?}", row);
                    row
                }
                Err(error) => {
                    warn!("conver_currency update failde!!{:?}", error);
                    return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
                }
            };
            //生成数字货币信息
            let mut digital_currency = DigitalCurrencyWrapper::new(
                MsgType::DigitalCurrency,
                DigitalCurrency::new(quota_control_field, target.clone()),
            );
            digital_currency
                .fill_kvhead(&keypair_sm2, &mut rng)
                .unwrap();
            new_digital_currency.push(digital_currency.to_bytes().encode_hex::<String>());
        }
    }
    HttpResponse::Ok().json(ResponseBody::new_success(Some(new_digital_currency)))
}

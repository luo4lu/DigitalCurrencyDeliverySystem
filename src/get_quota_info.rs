use crate::response::ResponseBody;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use log::{info, warn};
//数据库相关
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

#[get("/api/quota/detail")]
pub async fn get_all_quota_info(data: web::Data<Pool>, req_head: HttpRequest) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库
    let conn = data.get().await.unwrap();
    //存储获取到的额度与数理的数组
    let mut quota_number: Vec<(i64, i64)> = Vec::new();
    let hundred: i64 = 10000;
    let hundred_value = serde_json::to_value(&hundred).unwrap();
    let fifty: i64 = 5000;
    let fifty_value = serde_json::to_value(&fifty).unwrap();
    let twenty: i64 = 2000;
    let twenty_value = serde_json::to_value(&twenty).unwrap();
    let ten: i64 = 1000;
    let ten_value = serde_json::to_value(&ten).unwrap();
    let five: i64 = 500;
    let five_value = serde_json::to_value(&five).unwrap();
    let one: i64 = 100;
    let one_value = serde_json::to_value(&one).unwrap();
    let pentagon: i64 = 50;
    let pentagon_value = serde_json::to_value(&pentagon).unwrap();
    let dime: i64 = 10;
    let dime_value = serde_json::to_value(&dime).unwrap();
    let point: i64 = 1;
    let point_value = serde_json::to_value(&point).unwrap();
    let hundred_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 AND cloud_user_id = $2",
            &[&hundred_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("hundred conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let hundred_len: i64 = hundred_state.len() as i64;
    quota_number.push((hundred, hundred_len));

    let fifty_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 AND cloud_user_id = $2",
            &[&fifty_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("fifty conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let fifty_len: i64 = fifty_state.len() as i64;
    quota_number.push((fifty, fifty_len));

    let twenty_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 AND cloud_user_id = $2",
            &[&twenty_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("twenty conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let twenty_len: i64 = twenty_state.len() as i64;
    quota_number.push((twenty, twenty_len));

    let ten_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 cloud_user_id = $2",
            &[&ten_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("ten conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let ten_len: i64 = ten_state.len() as i64;
    quota_number.push((ten, ten_len));

    let five_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 cloud_user_id = $2",
            &[&five_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("five conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let five_len: i64 = five_state.len() as i64;
    quota_number.push((five, five_len));

    let one_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 cloud_user_id = $2",
            &[&one_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("one conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let one_len: i64 = one_state.len() as i64;
    quota_number.push((one, one_len));

    let pentagon_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 cloud_user_id = $2",
            &[&pentagon_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("pentagon conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let pentagon_len: i64 = pentagon_state.len() as i64;
    quota_number.push((pentagon, pentagon_len));

    let dime_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 cloud_user_id = $2",
            &[&dime_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("dime conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let dime_len: i64 = dime_state.len() as i64;
    quota_number.push((dime, dime_len));

    let point_state = match conn
        .query(
            "select id from digital_currency where (explain_info->'t_obj'->'value') = $1 cloud_user_id = $2",
            &[&point_value, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("point conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    let point_len: i64 = point_state.len() as i64;
    quota_number.push((point, point_len));
    HttpResponse::Ok().json(ResponseBody::new_success(Some(quota_number)))
}

#[derive(Deserialize, Debug)]
pub struct QuotaRequest {
    page: i64,
    page_size: i64,
}

#[derive(Serialize, Debug)]
struct GetAllQuotaResponseInner {
    id: String,
    value: i64,
    create_time: NaiveDateTime,
    owner: String,
}

#[derive(Serialize, Debug)]
struct GetAllQoutaResponse {
    total: i64,
    inner: Vec<GetAllQuotaResponseInner>,
}

#[get("/api/quota/detail/info")]
pub async fn get_all_quota(
    data: web::Data<Pool>,
    req: web::Query<QuotaRequest>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库
    let conn = data.get().await.unwrap();

    let offset = (req.page - 1) * req.page_size;

    match conn
        .query("SELECT digital_currency.id, (digital_currency.explain_info->'t_obj'->'value')::BIGINT as value, digital_currency.create_time, digital_currency.owner FROM digital_currency ORDER BY digital_c
urrency.create_time OFFSET $1 LIMIT $2 where cloud_user_id = $3",
    &[&offset, &req.page_size, &head_str]).await{
        Ok(row) => {
            //info!("select success!{:?}", row);\
            let total_state = conn.query("SELECT COUNT(*) FROM digital_currency where cloud_user_id = $1",&[&head_str]).await.unwrap();
            let total: i64 = total_state[0].get(0);
            let mut v = Vec::new();
            for r in row {
                let id = r.get(0);
                let value = r.get(1);
                let create_time = r.get(2);
                let owner = r.get(3);
                 let inner = GetAllQuotaResponseInner { id, value, create_time, owner, };
                v.push(inner);
            }
            let resp = GetAllQoutaResponse {total, inner: v};
            return HttpResponse::Ok().json(ResponseBody::<GetAllQoutaResponse>::new_success(Some(resp)));
        }
        Err(error) => {
            warn!("point conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
}

#[derive(Deserialize, Debug)]
pub struct HistoryRequest {
    id: String,
    page: i64,
    page_size: i64,
}

#[derive(Serialize, Debug)]
struct GetAllTranscationResponseInner {
    id: String,
    create_time: NaiveDateTime,
    owner: serde_json::Value,
}

#[derive(Serialize, Debug)]
struct GetAllTranscationResponse {
    total: i64,
    inner: Vec<GetAllTranscationResponseInner>,
}

#[get("/api/quota/detail/history")]
pub async fn get_all_transcation(
    data: web::Data<Pool>,
    req: web::Query<HistoryRequest>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库
    let conn = data.get().await.unwrap();

    let offset = (req.page - 1) * req.page_size;

    match conn
        .query("SELECT transaction_history.id, transaction_history.create_time, transaction_history.owner FROM transaction_history where id=$1 ORDER BY create_time OFFSET $2 LIMIT $3 where cloud_user_id = $4",
    &[&req.id, &offset, &req.page_size, &head_str]).await{
        Ok(row) => {
            //info!("select success!{:?}", row);\
            let total_state = conn.query("SELECT COUNT(*) FROM transaction_history where id = $1 AND cloud_user_id = $2",&[&req.id, &head_str]).await.unwrap();
            let total: i64 = total_state[0].get(0);
            let mut v = Vec::new();
            for r in row {
                let id = r.get(0);
                let create_time = r.get(1);
                let owner_cert: String = r.get(2);
                let url = format!("http://git.curdata.cn:9004/api/wallet?cert={}", owner_cert.to_ascii_uppercase());
                let owner: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
                let inner = GetAllTranscationResponseInner { id, create_time, owner: owner.get("data").unwrap().clone(), };
                v.push(inner);
            }
            let resp = GetAllTranscationResponse {total, inner: v};
            return HttpResponse::Ok().json(ResponseBody::new_success(Some(resp)));
        }
        Err(error) => {
            warn!("point conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
}

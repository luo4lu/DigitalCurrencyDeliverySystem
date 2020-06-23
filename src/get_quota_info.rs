use crate::response::ResponseBody;
use actix_web::{get, web, HttpResponse, Responder};
use log::{info, warn};
//数据库相关
use deadpool_postgres::Pool;

#[get("/api/quota/detail")]
pub async fn get_all_quota_info(data: web::Data<Pool>) -> impl Responder{
    //连接数据库
    let conn = data.get().await.unwrap();
    //存储获取到的额度与数理的数组
    let mut quota_number:Vec<(i64,i64)> = Vec::new();
    let hundred:i64 = 10000;
    let hundred_value = serde_json::to_value(&hundred).unwrap();
    let fifty:i64 = 5000;
    let fifty_value = serde_json::to_value(&fifty).unwrap();
    let twenty:i64 = 2000;
    let twenty_value = serde_json::to_value(&twenty).unwrap();
    let ten:i64 = 1000;
    let ten_value = serde_json::to_value(&ten).unwrap();
    let five:i64 = 500;
    let five_value = serde_json::to_value(&five).unwrap();
    let one:i64 = 100;
    let one_value = serde_json::to_value(&one).unwrap();
    let pentagon:i64 = 50;
    let pentagon_value = serde_json::to_value(&pentagon).unwrap();
    let dime:i64 = 10;
    let dime_value = serde_json::to_value(&dime).unwrap();
    let point:i64 = 1;
    let point_value = serde_json::to_value(&point).unwrap();
    let hundred_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&hundred_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("hundred conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let hundred_len:i64 = hundred_state.len() as i64;
    quota_number.push((hundred,hundred_len));
    
    let fifty_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&fifty_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("fifty conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let fifty_len:i64 = fifty_state.len() as i64;
    quota_number.push((fifty,fifty_len));

    let twenty_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&twenty_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("twenty conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let twenty_len:i64 = twenty_state.len() as i64;
    quota_number.push((twenty,twenty_len));

    let ten_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&ten_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("ten conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let ten_len:i64 = ten_state.len() as i64;
    quota_number.push((ten,ten_len));

    let five_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&five_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("five conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let five_len:i64 = five_state.len() as i64;
    quota_number.push((five,five_len));

    let one_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&one_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("one conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let one_len:i64 = one_state.len() as i64;
    quota_number.push((one,one_len));

    let pentagon_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&pentagon_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("pentagon conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let pentagon_len:i64 = pentagon_state.len() as i64;
    quota_number.push((pentagon,pentagon_len));

    let dime_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&dime_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("dime conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let dime_len:i64 = dime_state.len() as i64;
    quota_number.push((dime,dime_len));

    let point_state = match conn
        .query("select id from digital_currency where (explain_info->'t_obj'->'value') = $1 ",
    &[&point_value]).await{
        Ok(row) => {
            info!("select success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("point conver_currency select failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(error.to_string())));
        }
    };
    let point_len:i64 = point_state.len() as i64;
    quota_number.push((point,point_len));
    HttpResponse::Ok().json(ResponseBody::new_success(Some(quota_number)))
}
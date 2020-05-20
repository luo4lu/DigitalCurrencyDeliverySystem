use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseBody<T>{
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ResponseBody<T>{
    //响应字段公共部分
    pub fn new_success(data: Option<T>) ->Self {
        Self{
            code : 0,
            message : String::from("success"),
            data,
        }
    }

}
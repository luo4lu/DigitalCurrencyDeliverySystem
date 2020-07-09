use crate::config_command;
use clap::ArgMatches;
use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{Config, NoTls};

#[derive(Clone)]
pub struct ConfigPath {
    pub meta_path: String,
}

impl Default for ConfigPath {
    fn default() -> Self {
        Self {
            meta_path: String::from("./digital_currency.json"),
        }
    }
}

//货币发行请求额度之前需要中心系统注册
pub struct CMSRequestAddr {
    pub central_addr: String,
}
impl CMSRequestAddr {
    pub fn new() -> Self {
        let mut _addr = String::new();
        let matches: ArgMatches = config_command::get_command();
        if let Some(c) = matches.value_of("cms") {
            _addr = "http://".to_string() + &c + "/api/dcds/qouta_issue";
        } else {
            _addr = String::from("http://localhost:8077/api/dcds/qouta_issue");
        }
        Self {
            central_addr: _addr,
        }
    }
}

//货币发行请求额度之后需要中心系统登记
pub struct QCSRequestAddr {
    pub quota_addr: String,
}
impl QCSRequestAddr {
    pub fn new() -> Self {
        let mut _addr = String::new();
        let matches: ArgMatches = config_command::get_command();
        if let Some(q) = matches.value_of("qcs") {
            _addr = "http://".to_string() + &q + "/api/quota";
        } else {
            _addr = String::from("http://localhost:8088/api/quota");
        }
        Self { quota_addr: _addr }
    }
}

//数据库配置文件
pub fn get_db() -> Pool {
    //配置数据库
    let mut cfg = Config::new();
    cfg.host("localhost"); //数据库地址
    cfg.user("postgres"); //数据库用户名称
    cfg.password("postgres"); //数据库密码
    cfg.dbname("test_digitalcurrencydeliverysystem"); //数据库名称
    let mgr = Manager::new(cfg, NoTls); //生产一个数据库管理池
    Pool::new(mgr, 8) //设置最大连接池
}

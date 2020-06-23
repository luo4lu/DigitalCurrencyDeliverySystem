use clap::{Arg, App, ArgMatches};

pub fn get_command() -> ArgMatches<'static>{
    App::new("digital currency delivery system")
            .version("0.1.0")
            .author("luo4lu <luo4lu@163.com>")
            .about("Go to the server and request the address")
            .arg(Arg::with_name("dcds")
                .short("d")
                .long("dcds")
                .help("set self DCD system IP addr and port")
                .takes_value(true))
            .arg(Arg::with_name("cms")
                .short("c")
                .long("cms")
                .help("set to Central manage system IP addr and port")
                .takes_value(true))
            .arg(Arg::with_name("qms")
                .short("q")
                .long("qms")
                .help("set to Quota manage system IP addr and port")
                .takes_value(true))
            .get_matches()

}
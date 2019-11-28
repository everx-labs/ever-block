/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

extern crate log4rs;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

pub fn init_full(path: &str) {
    std::fs::remove_file(&format!("{}log/tvm.log", path)).unwrap_or_default();
    if let Err(err) = log4rs::init_file(&format!("{}log_cfg.yml", path), Default::default()) {
        println!("Error initialize logging configuration. config: {}log_cfg.yml\n{}", path, err);
    }
}

pub fn init() {
    // do not init twice
    if log_enabled!(log::Level::Info) {
        return
    }
    let log_level = if cfg!(feature = "verbose") {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };
    let encoder_boxed = Box::new(PatternEncoder::new("{m}"));
    let config: Config;
    if cfg!(feature = "log_file") {
        let file = FileAppender::builder()
            .encoder(encoder_boxed)
            .build("log/log.txt")
            .unwrap();
        config = Config::builder()
            .appender(Appender::builder().build("file", Box::new(file)))
            .build(Root::builder().appender("file").build(log_level))
            .unwrap();
    } else {
        let console = ConsoleAppender::builder()
            .encoder(encoder_boxed)
            .build();
        config = Config::builder()
            .appender(Appender::builder().build("console", Box::new(console)))
            .build(Root::builder().appender("console").build(log_level))
            .unwrap();
    }
    match log4rs::init_config(config) {_=>()}
}

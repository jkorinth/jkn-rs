#![cfg(test)]

use std::path::PathBuf;
use tempfile::tempdir;
use jkn::db::Database;
use jkn::config::{Config, MockConfig, GitConfig};

#[test]
fn it_respects_the_config_path() {
    let tmploc = tempdir().unwrap();
    /*let cfg = config::Config {
        loc: PathBuf::from(tmploc.path()),
        git: config::GitConfig {
            repopath: PathBuf::from(tmploc.path()),
        }
    };*/
    let mut cfg = Box::new(MockConfig::new());
    let gc: GitConfig = GitConfig { repopath: PathBuf::from("/tmp/testjkn") };
    cfg.expect_git().return_const(gc);
    let c: Box<dyn Config> = Box::new(cfg);
    let db = Database::from_config(&c).expect("unable to open database");
    let topic = db
        .topic(Some("test topic"))
        .expect("unable to create topic");
}

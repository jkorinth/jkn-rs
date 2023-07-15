#![cfg(test)]

use jkn::config::Config;
use jkn::config::GitConfig;
use jkn::db::Database;
use log::*;
use std::io;
use std::path::PathBuf;
use std::sync::Once;
use tempfile::{tempdir, TempDir};

#[allow(dead_code)]
struct MockConfig {
    tmpdir: TempDir,
    loc: PathBuf,
    git: GitConfig,
}

static INIT_LOGGER: Once = Once::new();

#[ctor::ctor]
fn init_logger() {
    INIT_LOGGER.call_once(|| env_logger::init());
}

impl Config for MockConfig {
    fn load() -> Result<Box<dyn Config>, String>
    where
        Self: Sized,
    {
        let tmpdir = tempdir().expect("could not create temp dir");
        let tmppath = tmpdir.path();
        debug!("tmploc = {:?}", tmppath);
        Ok(Box::new(MockConfig {
            loc: PathBuf::from(tmppath),
            git: GitConfig {
                repopath: tmppath.to_path_buf(),
            },
            tmpdir: tmpdir,
        }))
    }

    fn loc(&self) -> &PathBuf {
        &self.loc
    }

    fn git(&self) -> &GitConfig {
        &self.git
    }

    fn save(&self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn it_respects_the_config_path() {
    let cfg = Box::new(MockConfig::load().unwrap());
    let db = Database::from_config(&cfg).expect("unable to open database");
    let _topic = db
        .topic(Some("test_topic"))
        .expect("unable to create topic");
}

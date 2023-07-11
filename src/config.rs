use log::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use toml;
use mockall::{automock, predicate::*};

#[derive(Serialize, Deserialize, Debug)]
pub struct GitConfig {
    pub repopath: PathBuf,
}

#[automock]
pub trait Config {
    fn load() -> Result<Box<dyn Config>, String> where Self: Sized;
    fn loc(&self) -> &PathBuf;
    fn git(&self) -> &GitConfig;
    fn save(&self) -> io::Result<()>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigImpl {
    loc: PathBuf,
    git: GitConfig,
}

impl Config for ConfigImpl {
    fn load() -> Result<Box<dyn Config>, String> {
        let home = env::var("HOME").expect("HOME env var is not set");
        let xdg_config_home = env::var("XDG_CONFIG_HOME");

        if xdg_config_home.is_ok() {
            let cfgfile = PathBuf::from(format!("{}/jkn/.config", xdg_config_home.unwrap()));
            if cfgfile.exists() {
                info!("found config in XDK_CONFIG_HOME");
                let cfgcontent =
                    fs::read_to_string(cfgfile.as_path()).expect("could not read file");
                let cfg = Box::new(toml::from_str::<Self>(&cfgcontent).unwrap());
                return Ok(cfg);
            }
        }

        let cfgfile = PathBuf::from(format!("{}/.jkn/.config", home));
        if cfgfile.exists() {
            info!("found config in HOME");
            let cfgcontent = fs::read_to_string(cfgfile.as_path()).expect("could not read file");
            let cfg = Box::new(toml::from_str::<Self>(&cfgcontent).unwrap());
            return Ok(cfg);
        }

        warn!("found no existing config, using defaults");
        Ok(Box::new(Self::default()))
    }

    fn loc(&self) -> &PathBuf {
        &self.loc
    }

    fn git(&self) -> &GitConfig {
        &self.git
    }

    fn save(&self) -> io::Result<()> {
        let cfg_path = self.loc.as_path();
        fs::create_dir_all(
            cfg_path
                .parent()
                .expect(format!("no parent found for loc: {:?}", self.loc).as_str()),
        )?;
        debug!("saving config to {:?}", cfg_path);
        let toml_str = toml::to_string(&self).expect("could not serialize");
        debug!("config TOML: {}", toml_str);
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(cfg_path)?;
        file.write_all(toml_str.as_bytes())
    }
}

impl Default for ConfigImpl {
    fn default() -> Self {
        let home = env::var("HOME").expect("HOME env var is not set");
        Self {
            loc: PathBuf::from(format!("{}/.jkn/.config", home)),
            git: GitConfig {
                repopath: PathBuf::from(format!("{}/.jkn/db", home)),
            },
        }
    }
}

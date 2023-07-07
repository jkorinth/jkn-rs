use crate::config::Config;
use git2::{Repository, RepositoryInitOptions, RepositoryOpenFlags, Tree};
use log::*;
use std::path::Path;
use crate::topic::Topic;

#[derive(Debug)]
pub enum DatabaseError {
    GitError(git2::Error),
    CouldNotCreateTopic(String)
}

impl From<git2::Error> for DatabaseError {
    fn from(err: git2::Error) -> Self {
        DatabaseError::GitError(err)
    }
}

pub struct Database {
    git: Repository,
}

impl Database {
    pub fn from_config(cfg: &Config) -> Result<Database, DatabaseError> {
        match Database::open_git(&cfg.git.repopath) {
            Ok(repo) => Ok(Database { git: repo }),
            Err(e) => {
                warn!("could not open database, re-initializing repo: {:?}", e);
                Ok(Database {
                    git: Database::init_git(&cfg.git.repopath)?,
                })
            }
        }
    }

    pub fn from_repo(repo: Repository) -> Database {
        Database { git: repo }
    }

    fn open_git(path: &Path) -> Result<Repository, DatabaseError> {
        Ok(Repository::open_ext(
            path,
            RepositoryOpenFlags::NO_SEARCH,
            &[] as &[&std::ffi::OsStr],
        )?)
    }

    fn init_git(path: &Path) -> Result<Repository, DatabaseError> {
        let mut repo_opts = RepositoryInitOptions::new();
        repo_opts.initial_head("main");
        let repo = Repository::init_opts(path, &repo_opts)?; //.expect("error init git");
        info!("is repo empty? {}", repo.is_empty().expect("repo ol"));
        {
            let oid = repo
                .treebuilder(None)
                .expect("could not create treebuilder")
                .write()
                .expect("could not write tree");
            let sig = repo.signature().expect("could not get default signature");
            let tree = repo.find_tree(oid).expect("could not find new tree");
            repo.commit(
                Some("refs/heads/main"),
                &sig,
                &sig,
                "initial commit",
                &tree,
                &[],
            )
            .expect("could not commit to tree");
        }
        for worktrees in repo.worktrees().iter() {
            for wt in worktrees.iter() {
                debug!("worktree: {:?}", wt);
            }
        }
        Ok(repo)
    }

    pub fn topic(&self, name: Option<String>) -> Result<Topic, DatabaseError> {
        if name.is_some() {
            let oid = self.git.treebuilder(None)?.write()?;
            let root = self.git.refname_to_id("main")?;
            let commit = self.git.find_commit(root)?;
            let branch = self.git.branch(&name.unwrap(), &commit, false)?;
            Ok(Topic::from_oid(oid))
        } else {
            Ok(Topic::from_oid(self.git.refname_to_id("HEAD")?))
        }
    }

    pub fn current_branch(&self) -> String {
        let refe = self.git.head().expect("unable to get HEAD");
        println!("SHA-1: {:?}", refe.peel_to_commit().expect("ok").id());
        refe.shorthand().unwrap().to_string()
    }
}

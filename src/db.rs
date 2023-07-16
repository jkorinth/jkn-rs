use super::err_from;
use super::note::Note;
use super::topic::Topic;
use crate::config::Config;
use chrono::prelude::*;
use git2::*;
use log::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    GitError(git2::Error),
    IoError(io::Error),
}

err_from!(Error, git2::Error, Error::GitError);
err_from!(Error, io::Error, Error::IoError);

type Result<T> = core::result::Result<T, Error>;

pub enum Entity {
    Topic,
    Note,
}

pub trait Database {
    fn from_config(cfg: &impl Config) -> Result<Self>
    where
        Self: Sized;
    fn topic(&self, name: Option<&str>) -> Result<Topic>;
    fn current_topic(&self) -> Option<String>;
    fn current_note(&self) -> String;
    fn amend(&self, notename: &str) -> Result<()>;
    fn commit(&self, notename: &str, _amend: bool) -> Result<()>;
    fn list(&self, kind: Entity) -> Result<Vec<String>>;
}

pub struct DatabaseImpl {
    git: Repository,
    path: PathBuf,
}

impl Database for DatabaseImpl {
    fn from_config(cfg: &impl Config) -> Result<Self> {
        match Self::open_git(&cfg.git().repopath) {
            Ok(repo) => Ok(DatabaseImpl {
                git: repo,
                path: cfg.git().repopath.clone(),
            }),
            Err(e) => {
                warn!("could not open database, re-initializing repo: {:?}", e);
                Ok(DatabaseImpl {
                    git: Self::init_git(&cfg.git().repopath)?,
                    path: cfg.git().repopath.clone(),
                })
            }
        }
    }

    fn topic(&self, name: Option<&str>) -> Result<Topic> {
        if let Some(n) = name {
            let mut b = self.find_topic_branch(n);
            if b.is_none() {
                b = Some(self.make_topic_branch(n)?);
            }
            let branch = b.unwrap();
            self.git
                .checkout_tree(&branch.get().resolve()?.peel_to_tree()?.as_object(), None)?;
            self.git
                .set_head(branch.get().name().expect("branch has no name"))?;
            Ok(Topic::from_name(name.as_ref().unwrap()))
        } else {
            let head_ref = self.git.find_reference("HEAD")?;
            let head_name = Reference::normalize_name(
                head_ref.name().expect("head ref has no name!"),
                ReferenceFormat::NORMAL,
            )?;
            Ok(Topic::from_name(&head_name))
        }
    }

    fn current_topic(&self) -> Option<String> {
        let branch_name = self.current_branch();
        Some(String::from(Path::new(&branch_name).file_name()?.to_str()?))
    }

    fn list(&self, kind: Entity) -> Result<Vec<String>> {
        match kind {
            Entity::Topic => self.list_topics(),
            Entity::Note => self.list_notes(),
        }
    }

    fn current_note(&self) -> String {
        let now = Utc::now();
        let (_, year) = now.year_ce();
        format!("{}-{:02}-{:02}.md", year, now.month(), now.day())
    }

    fn amend(&self, notename: &str) -> Result<()> {
        self.commit(notename, true)
    }

    fn commit(&self, notename: &str, _amend: bool) -> Result<()> {
        if _amend {
            warn!("amend is not implemented yet, will create new commit");
        }
        let mut index = self.git.index()?;
        debug!("adding {} to index ({:?})", notename, index.path());
        index.add_path(&Path::new(notename))?;
        let mut path = self.git.path().to_path_buf();
        path.pop();
        path.push(notename);
        debug!("full path: {:?}", path);
        let note = Note::from(&path);
        let summary = &note.summary()?;
        debug!("summary: {}", summary);
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = self.git.find_tree(tree_oid)?;
        self.git.commit(
            Some("HEAD"),
            &self.git.signature()?,
            &self.git.signature()?,
            summary.trim_end(),
            &tree,
            &[&self.git.head()?.peel_to_commit()?],
        )?;
        Ok(())
    }
}

impl DatabaseImpl {
    fn open_git(path: &Path) -> Result<Repository> {
        Ok(Repository::open_ext(
            path,
            RepositoryOpenFlags::NO_SEARCH,
            &[] as &[&std::ffi::OsStr],
        )?)
    }

    fn init_git(path: &Path) -> Result<Repository> {
        let mut repo_opts = RepositoryInitOptions::new();
        repo_opts.initial_head("main");
        let repo = Repository::init_opts(path, &repo_opts)?;
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

    fn topic_branch(name: &str) -> String {
        format!("topic/{}", name)
    }

    fn topic_name(branch_name: &str) -> Option<String> {
        let topic = Path::new(branch_name).file_name()?.to_str()?;
        let tb = DatabaseImpl::topic_branch(topic);
        if tb.as_str() == branch_name {
            Some(topic.to_string())
        } else {
            None
        }
    }

    fn find_topic_branch(&self, name: &str) -> Option<Branch> {
        let branch_name = format!("topic/{}", name);
        Some(
            self.git
                .branches(None)
                .ok()?
                .filter(|r| r.is_ok())
                .find(|b| b.as_ref().unwrap().0.name().unwrap() == Some(&branch_name))?
                .ok()?
                .0,
        )
    }

    fn make_topic_branch(&self, name: &str) -> Result<Branch> {
        let main_branch = self.git.find_branch("main", BranchType::Local)?;
        let main_ref = main_branch.get().resolve()?.target();
        let main_commit = self.git.find_commit(main_ref.expect("main has no Oid!"))?;
        let branch_name = format!("topic/{}", name);
        Ok(self.git.branch(&branch_name, &main_commit, false)?)
    }

    fn current_branch(&self) -> String {
        let refe = self.git.head().expect("unable to get HEAD");
        refe.shorthand().unwrap().to_string()
    }

    fn list_topics(&self) -> Result<Vec<String>> {
        Ok(self
            .git
            .branches(None)?
            .flatten()
            .map(|(d, _)| Self::topic_name(d.name().unwrap().unwrap()))
            .flatten()
            .collect())
    }

    fn list_notes(&self) -> Result<Vec<String>> {
        let mut entries: Vec<_> = fs::read_dir(&self.path)?
            .flatten()
            .filter(|e| !e.file_type().unwrap().is_dir())
            .map(|e| e.path().file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        entries.sort();
        Ok(entries)
    }
}

pub fn from_config(cfg: &impl Config) -> Result<impl Database> {
    Ok(DatabaseImpl::from_config(cfg)?)
}

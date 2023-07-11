use mockall_double::double;
#[double]
use crate::config::Config;
use super::note::Note;
use super::topic::Topic;
use chrono::prelude::*;
use git2::*;
use log::*;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub enum DatabaseError {
    GitError(git2::Error),
    IoError(io::Error),
}

impl From<git2::Error> for DatabaseError {
    fn from(err: git2::Error) -> Self {
        DatabaseError::GitError(err)
    }
}

impl From<io::Error> for DatabaseError {
    fn from(err: io::Error) -> Self {
        DatabaseError::IoError(err)
    }
}

pub enum Entity {
    Topic,
    Note,
}

pub struct Database {
    git: Repository,
}

impl Database {
    pub fn from_config(cfg: &Box<dyn Config>) -> Result<Database, DatabaseError> {
        match Database::open_git(&cfg.git().repopath) {
            Ok(repo) => Ok(Database { git: repo }),
            Err(e) => {
                warn!("could not open database, re-initializing repo: {:?}", e);
                Ok(Database {
                    git: Database::init_git(&cfg.git().repopath)?,
                })
            }
        }
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

    fn topic_branch(name: &str) -> String {
        format!("topic/{}", name)
    }

    fn topic_name(branch_name: &str) -> Option<String> {
        let topic = Path::new(branch_name).file_name()?.to_str()?;
        let tb = Database::topic_branch(topic);
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

    fn make_topic_branch(&self, name: &str) -> Result<Branch, DatabaseError> {
        let main_branch = self.git.find_branch("main", BranchType::Local)?;
        let main_ref = main_branch.get().resolve()?.target();
        let main_commit = self.git.find_commit(main_ref.expect("main has no Oid!"))?;
        let branch_name = format!("topic/{}", name);
        Ok(self.git.branch(&branch_name, &main_commit, false)?)
    }

    pub fn topic(&self, name: Option<&str>) -> Result<Topic, DatabaseError> {
        if let Some(n) = name {
            let branch = self
                .find_topic_branch(n)
                .or_else(|| Some(self.make_topic_branch(n).ok()?))
                .unwrap();
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

    pub fn current_topic(&self) -> Option<String> {
        let branch_name = self.current_branch();
        Some(String::from(Path::new(&branch_name).file_name()?.to_str()?))
    }

    fn current_branch(&self) -> String {
        let refe = self.git.head().expect("unable to get HEAD");
        refe.shorthand().unwrap().to_string()
    }

    pub fn list(&self, kind: Entity) -> Option<Vec<String>> {
        match kind {
            Entity::Topic => self.list_topics(),
            Entity::Note => self.list_notes(),
        }
    }

    fn list_topics(&self) -> Option<Vec<String>> {
        Some(
            self.git
                .branches(None)
                .ok()?
                .flatten()
                .map(|r| Database::topic_name(r.0.name().ok()??))
                .flatten()
                .map(|s| String::from(s))
                .collect(),
        )
    }

    fn list_notes(&self) -> Option<Vec<String>> {
        // TODO implement listing of files in worktree
        panic!("NOT IMPLEMENTED")
    }

    pub fn current_note(&self) -> String {
        let now = Utc::now();
        let (_, year) = now.year_ce();
        format!("{}-{:02}-{:02}.md", year, now.month(), now.day())
    }

    pub fn commit(&self, notename: &str) -> Result<(), DatabaseError> {
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

use env_logger;
use jkn::config;
use jkn::db;
use log::*;
use regex::{Captures, Regex};
use reqwest::blocking;
use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::iter::zip;
use std::path::PathBuf;
use time::format_description;
use time::macros::date;

use String as Topic;

static URL: &str = "https://www.gutenberg.org/cache/epub/100/pg100.txt";

fn get_shakespeare() -> reqwest::Result<String> {
    let p = PathBuf::from("shakespeare.txt");
    if p.exists() {
        let t = fs::read_to_string(p).expect("could not read file!");
        Ok(t)
    } else {
        let t = reqwest::blocking::get(URL)?.text()?;
        let mut f = fs::File::create(p).expect("could not create file");
        write!(f, "{}", t).expect("could not write file!");
        Ok(t)
    }
}

fn get_topics(txt: &String) -> (Vec<Topic>, usize) {
    let mut s = &txt[0..];
    let toc = s.find("Contents").expect("could not find toc") + "Contents".len();
    s = &s[toc..];
    let toc_start = s
        .find(|c| !"\r\t \n".contains(c))
        .expect("could not find toc start");
    s = &s[toc_start..];
    let toc_end = s.find("\n\r\n\r").expect("could not find toc end");
    s = &s[..toc_end];
    (
        s.split("\n").map(|s| String::from(s.trim())).collect(),
        toc + toc_start + toc_end,
    )
}

fn get_texts(mono: &str, topics: &Vec<Topic>) -> HashMap<Topic, String> {
    let mut ret: HashMap<String, String> = HashMap::new();
    let mut s = &mono[0..];
    let mut t2it = topics.iter();
    let _ = t2it.next();
    for (&ref t1, &ref t2) in zip(topics.iter(), t2it) {
        let tstart = Regex::new(&format!("\\s*{}\\s*(\n\r?)", t1)).unwrap();
        let start = tstart
            .find(&s)
            .expect(&format!("could not find topic {}", t1))
            .start();
        s = &mono[start..];
        let tend = Regex::new(&format!("\\s*{}\\s*(\n\r?)", t2)).unwrap();
        let end = tend
            .find(&s)
            .expect(&format!("could not find topic {}", t2))
            .start();
        ret.insert(t1.clone(), String::from(&s[..end]));
        s = &mono[start + end..];
    }
    ret.insert(topics.last().unwrap().clone(), String::from(s));
    ret
}

fn escape_markdown(txt: &str) -> String {
    let rep = Regex::new(r"([`'*_#])").unwrap();
    rep.replace_all(txt, |caps: &Captures| format!("\\{}", &caps[1]))
        .to_string()
}

fn split_into_paragraphs(txt: &str) -> Vec<&str> {
    let empty_line = Regex::new(r"\n\r?(\s*[[:space:][:digit:]]*\n\r?)+").unwrap();
    empty_line.split(txt).collect()
}

fn split_into_commits(
    pmap: &HashMap<Topic, Vec<&str>>,
) -> HashMap<Topic, HashMap<PathBuf, String>> {
    let mut ret: HashMap<String, HashMap<PathBuf, String>> = HashMap::new();
    for (&ref topic, &ref paragraphs) in pmap.iter() {
        let mut thm: HashMap<PathBuf, String> = HashMap::new();
        let mut edate = date!(2020 - 01 - 01);
        let paragraphs_per_day = cmp::max(paragraphs.len() / 365, 1);
        debug!("{}: paragraphs per day = {}", &topic, paragraphs_per_day);
        for ps in paragraphs.chunks(paragraphs_per_day) {
            let commit = ps
                .iter()
                .fold(format!("Working on {}\n\n", &topic,), |c, n| c + n);
            let notename = edate
                .format(&format_description::parse("[year]-[month]-[day]").unwrap())
                .unwrap();
            let mut path = PathBuf::new();
            path.push(format!("{}.md", notename));
            trace!("{}: [{:?}]\n{}", &topic, edate, commit);
            thm.insert(path, commit);
            edate = edate.next_day().unwrap();
        }
        ret.insert(topic.clone(), thm);
    }
    ret
}

fn commit_to_db(db: &impl db::Database, data: &HashMap<Topic, HashMap<PathBuf, String>>) {
    let mut no_commits: usize = 0;
    let mut no_topics: usize = 0;
    for (&ref topic, &ref commits) in data.iter() {
        db.topic(Some(topic)).expect(&format!("could not switch to topic {}", &topic));
        for (&ref p, &ref txt) in commits.iter() {
            let mut path = db.root_path();
            path.push(format!("{}", p.to_str().unwrap()));
            let mut f =
                fs::File::create(&path).expect(&format!("could not create file: {:?}", path));
            write!(f, "{}", txt).expect(&format!("could not write file: {:?}", path));
            debug!("wrote file: {:?}", path);
            db.commit(path.file_name().unwrap().to_str().unwrap(), false).expect("could not commit");
            no_commits += 1;
        }
        no_topics += 1;
    }
    info!("done! wrote {} commits for {} topics.", no_commits, no_topics);
}

fn main() {
    env_logger::init();
    let cfg = config::load().expect("could not load configuration");
    let db = db::from_config(&cfg).expect("unable to open database");

    let mono = get_shakespeare().expect("could not download");
    let (topics, idx) = get_topics(&mono);
    let m = escape_markdown(&mono[idx..]);
    let r = get_texts(&m, &topics);
    let data: HashMap<String, Vec<&str>> = topics
        .iter()
        .map(|t| (t.clone(), split_into_paragraphs(&r[t])))
        .collect();
    commit_to_db(&db, &split_into_commits(&data));
}

use crate::db;
use crate::err_from;
use std::io;
use termimad;

#[derive(Debug)]
pub enum Error {
    DatabaseError(db::Error),
    IoError(io::Error),
    TermimadError(termimad::Error),
}

err_from!(Error, db::Error, Error::DatabaseError);
err_from!(Error, io::Error, Error::IoError);
err_from!(Error, termimad::Error, Error::TermimadError);

pub type Result<T> = core::result::Result<T, Error>;

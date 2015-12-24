
extern crate regex;

use std::collections::HashMap;
use std::default::Default;
use std::io::{self, Read};
use std::fmt;
use std::fs;
use std::fs::File;
use std::error::Error;
use std::result::Result as StdResult;
use std::boxed::Box;

use serde_json;
use self::regex::Regex;
use openssl::ssl::{SslContext, SslMethod};
use postgres::{Connection, SslMode};
use postgres::error::ConnectError as DBConnectError;
use postgres::error::Error as DBError;

use script;
use Config;

//error and result
#[derive(Debug)]
pub enum ImporterError {
	Io(io::Error),
	Job(String),
	DatabaseConnect(DBConnectError),
	Database(DBError),
	Json(serde_json::Error),
}
impl Error for ImporterError {
	fn description(&self) -> &str {
		match *self {
			ImporterError::Io(ref io) => io.description(),
			ImporterError::Json(ref json) => json.description(),
			ImporterError::Job(ref string) => string,
			ImporterError::DatabaseConnect(ref err) => err.description(),
			ImporterError::Database(ref err) => err.description(),
		}
	}
}
impl fmt::Display for ImporterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        self.description().fmt(f)
    }
}
impl From<io::Error> for ImporterError {
	fn from(err: io::Error) -> Self {
		ImporterError::Io(err)
	}
}
impl From<serde_json::Error> for ImporterError {
	fn from(err: serde_json::Error) -> Self {
		ImporterError::Json(err)
	}
}
impl From<DBError> for ImporterError {
	fn from(err: DBError) -> Self {
		ImporterError::Database(err)
	}
}
impl From<DBConnectError> for ImporterError {
	fn from(err: DBConnectError) -> Self {
		ImporterError::DatabaseConnect(err)
	}
}
pub type Result<T> = StdResult<T, ImporterError>;

//job
struct Job {
	sql: String,
	sheet: String,
}

impl Job {
	pub fn new(sqlfile: &str) -> Result<Job> {
		let mut job = Job {
			sql: Default::default(),
			sheet: Default::default(),
		};
		//parse sqlfile to get sheet name
		let re = Regex::new("(.+?).sql$").unwrap();
		match re.captures(sqlfile) {
			Some(c) => {
				job.sheet = c.at(1).unwrap().to_string();
			},
			None => {
				return Err(ImporterError::Job("no sql file".to_string()));
			},
		}
		let mut f = try!(File::open(sqlfile));
		let mut buf = String::new();
		try!(f.read_to_string(&mut buf));
		job.sql = buf;
		//init HashMap
		Ok(job)
	}

	pub fn run(&self, imp: &Importer) -> Result<()> {
		let mut record = HashMap::<String, String>::new();
		//query postgres to get record
		let stmt = try!(imp.dbconn.prepare(self.sql.as_str()));
		for row in try!(stmt.query(&[])) {
			for col in row.columns() {
				let col = col.name();
				record.insert(col.to_string(), row.get(col));
			}
		}
		//make request to spreadsheet through script object
		let config = &imp.config;
		let runner = script::Runner::new(config);
		let recstr = try!(serde_json::to_string(&record));
		let params = vec![config.spreadsheet_url.clone(), self.sheet.clone(), recstr];
		match runner.run(config.script_id.as_str(), Some(config.func_name.clone()), Some(params), None) {
			Err(e) => println!("Error: {}", e),
			Ok(res) => println!("Success: {:?}", res),
		}
		Ok(())
	}
}

//importer
pub struct Importer {
	config: Config,
	jobs: Vec<Job>,
	pub dbconn: Connection,
}

impl Importer {
	pub fn new(c: Config) -> Result<Importer> {
		let ctx = Box::new(SslContext::new(SslMethod::Sslv23).unwrap());
		let dbconn = try!(Connection::connect(c.psql_url.as_str(), &SslMode::Require(ctx)));
		let mut jobs = Vec::new();
		//iterate over c.sqldir and create job objeet, insert to jobs
		{
			let dir = c.sqldir.as_str();
			if try!(fs::metadata(dir)).is_dir() {
				for entry in try!(fs::read_dir(dir)) {
					let entry = entry.unwrap();
					let path = &entry.path();
					if !try!(fs::metadata(path)).is_dir() {
						match Job::new(path.to_str().unwrap()) {
							Ok(j) => jobs.push(j),
							Err(e) => match e {
								ImporterError::Io(_)
								|ImporterError::Json(_)
								|ImporterError::DatabaseConnect(_)
								|ImporterError::Database(_) => return Err(e),
								ImporterError::Job(j) => {
									println!("job error ignored: {}", j);
								}
							}
						}
					}
				}
			}
		}
		Ok(Importer {
			config: c, 
			jobs: jobs,
			dbconn: dbconn,
		})
	}
	pub fn run(&self) -> Result<()> {
		for j in &self.jobs {
			try!(j.run(&self));
		}
		Ok(())
	}
}

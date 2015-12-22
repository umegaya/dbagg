extern crate postgres;
extern crate regex;

use std::collections::HashMap;
use std::collections::Array;
use std::io::{self, Read, Write};

use Config;

struct Job {
	sqlfile String,
	sheet String,
	record HashMap<String, String>,
}

impl Job {
	pub fn new(sqlfile: String) {
		//parse sqlfile to get sheet name
		
		//init HashMap
	}

	pub fn run() {
		//query postgres to get record
		//make request to spreadsheet through script object
	}
}

pub struct Importer {
	config Config,
	jobs Vec<Job>,
}

impl Importer {
	pub fn new(&c: Config) -> io::Result<Importer> {
		let imp = Importer {
			config: c.clone(),
			jobs: Vec::new(),
		}
		//iterate over c.sqldir and create job objeet, insert to jobs
		if try!(fs::metadata(c.sqldir)).is_dir() {
			for entry in try!(fs::read_dir(c.sqldir)) {
				let entry = entry.unwrap();
				if !try!(fs::metadata(path)).is_dir() {
					let j = Job::new(path);
					imp.jobs.push(j);
				}
			}
		}

		Ok(imp)
	}
}

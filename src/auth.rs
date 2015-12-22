extern crate hyper;
extern crate open;
extern crate yup_oauth2 as oauth2;
extern crate serde;
extern crate serde_json;

use std::default::Default;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::fmt;
use std::convert::From;
use std::io::{self, Read, Write};
use Config;

//--------------------------------------------------
//
//	FileStorage: provide persistence of oauth token
//
//--------------------------------------------------
struct FileStorage {
	filepath: String,
	loaded: bool,
	tokens: HashMap<String, oauth2::Token>,
}

#[derive(Debug)]
pub enum FileStorageError {
	Io(io::Error),
	Json(serde_json::Error),
}
impl Error for FileStorageError {
	fn description(&self) -> &str {
		match *self {
			FileStorageError::Io(ref io) => io.description(),
			FileStorageError::Json(ref json) => json.description(),
		}
	}
}
impl fmt::Display for FileStorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}
impl From<io::Error> for FileStorageError {
	fn from(err: io::Error) -> Self {
		FileStorageError::Io(err)
	}
}
impl From<serde_json::Error> for FileStorageError {
	fn from(err: serde_json::Error) -> Self {
		FileStorageError::Json(err)
	}
}

impl oauth2::TokenStorage for FileStorage {
	type Error = FileStorageError;
	fn set(&mut self, scope_hash: u64, _: &Vec<&str>, token: Option<oauth2::Token>) -> Result<(), FileStorageError> {
		let scope_key = scope_hash.to_string();
		if let Some(t) = token {
			self.tokens.insert(scope_key, t);
		}
		else {
			println!("data for scope_hash {} removed", scope_key);
			self.tokens.remove(&scope_key);
		}
		return self.save();
	}
	fn get(&self, scope_hash: u64, _: &Vec<&str>) -> Result<Option<oauth2::Token>, FileStorageError> {
		let scope_key = scope_hash.to_string();
		match self.tokens.get(&scope_key) {
			Some(t) => {
				println!("data for scope_hash {} found", scope_key);
				Ok(Some(t.clone()))
			},
            None => Ok(None),			
		}
	}
}

impl FileStorage {
	fn open(&mut self) -> Result<File, io::Error> {
		return OpenOptions::new()
			.read(true)
            .write(true)
            .create(true)
            .open(&self.filepath);
	}
	fn ensure_loaded(&mut self) -> Result<(), FileStorageError> {
		if !self.loaded {
		    let mut f = try!(self.open());
		    let mut buf = String::new();
		    try!(f.read_to_string(&mut buf));
		    if buf.len() >= 2 {
			    self.tokens = serde_json::from_str(buf.as_str()).unwrap();
			}
			else {
				self.tokens = HashMap::new();
			}
			self.loaded = true;
			return Ok(())
		}
		Ok(())
	}
	fn save(&mut self) -> Result<(), FileStorageError> {
		let data = try!(serde_json::to_string(&self.tokens));
		if data.len() > 2 {
			let mut f = try!(self.open());
			return f.write_all(data.as_bytes()).map_err( |e| FileStorageError::Io(e) );
		}
		return Ok(());

	}
	fn new(path: String) -> Result<FileStorage, FileStorageError> {
		let mut fs = FileStorage {
			filepath: path,
			loaded: false,
			tokens: HashMap::new(),
		};
		match fs.ensure_loaded() {
			Ok(()) => Ok(fs),
			Err(e) => Err(e),
		}
	}
}



//--------------------------------------------------
//
//	StdoutHandler: provide customized authenticate delegate
//
//--------------------------------------------------
struct StdoutHandler;
impl oauth2::AuthenticatorDelegate for StdoutHandler {
    fn present_user_code(&mut self, pi: &oauth2::PollInformation) {
		println!("Please enter '{}' at {} and authenticate the application for the\n\
                  given scopes. This is not a test !\n\
                  You have time until {} to do that.
                  Do not terminate the program until you deny or grant access !",
                  pi.user_code, pi.verification_url, pi.expires_at);
        let delay = Duration::new(5, 0);
        println!("Browser opens automatically in {} seconds", delay.as_secs());
        sleep(delay);
        open::that(&pi.verification_url).ok();
        println!("DONE - waiting for authorization ...");
    }
}



//--------------------------------------------------
//
//	AuthenticatorFactory
//
//--------------------------------------------------
//declare Authenticator
pub type Authenticator = oauth2::Authenticator<StdoutHandler, FileStorage, hyper::Client>;

pub struct AuthenticatorFactory;
impl AuthenticatorFactory {
	pub fn create(c: Config) -> Result<Authenticator, FileStorageError> {
		let secret = oauth2::ApplicationSecret {
	        client_id: c.oauth_id,
	        client_secret: c.oauth_secret,
	        token_uri: Default::default(),
	        auth_uri: Default::default(),
	        redirect_uris: Default::default(),
	        client_email: None,
	        auth_provider_x509_cert_url: None,
	        client_x509_cert_url: None
	    };	
	    let client = hyper::Client::new();
	    let tokenfile: String;
	    if c.tokenfile.len() > 0 { 
	    	tokenfile = c.tokenfile;
	    }
	    else {
	    	tokenfile = "./tokens.json".to_string();
	    }
	    let storage = try!(FileStorage::new(tokenfile));
	    return Ok(Authenticator::new(&secret, StdoutHandler, client, storage, None));
	}
}

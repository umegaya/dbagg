extern crate hyper;
extern crate google_script1 as script1;

use auth;
use Config;

pub struct Runner {
	hub: script1::Script<hyper::Client, auth::Authenticator>,
}
impl Runner {
	pub fn new(c: Config) -> Runner {
	    //setup oauth2 and google app script instance
	    let auth = auth::AuthenticatorFactory::create(c).unwrap();
		return Runner {
			hub: script1::Script::new(hyper::Client::new(), auth),
		}
	}
	pub fn run(&self, id: &str, function: Option<String>, parameters: Option<Vec<String>>, dev_mode: Option<bool>) 
		-> script1::Result<(hyper::client::Response, script1::Operation)> {
	    //invoke request and receive result
	    let req = script1::ExecutionRequest {
	    	function: function,
	    	parameters: parameters,
	    	dev_mode: dev_mode,
	    	session_state: None,
	    };
	    return self.hub.scripts().run(req, id).add_scope("https://www.googleapis.com/auth/spreadsheets").doit();
	}
}


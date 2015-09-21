extern crate hyper;
extern crate rustc_serialize;

use std::fmt;
use std::io::Read;
use std::result::{Result};
use rustc_serialize::json::Json;
use hyper::{Client};

struct TheMovieDB {
	api_key: &'static str,
	base_url: &'static str,
	image_base_url: String,
	http_client: Client,
}

impl fmt::Debug for TheMovieDB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TheMovieDB => ApiKey: {}, BaseURL: {}, ImageBaseURL: {}",
		 self.api_key,
		 self.base_url,
		 self.image_base_url)
    }
}

impl TheMovieDB {
	/// Constructs a new TheMovieDB`.
	///
	/// # Examples
	///
	/// ```
	/// use std::rc::Rc;
	///
	/// let five = Rc::new(5);
	/// ```
	fn new(themoviedb_api_key: &'static str) -> Result<TheMovieDB, String> {
		let mut new_instance = TheMovieDB{api_key: themoviedb_api_key,
			base_url: "https://api.themoviedb.org/3",
			image_base_url: "".to_string(),
			http_client: Client::new()};

		let configuration_data = match new_instance.get_json_data_for("/configuration") {
			Ok(c) => c,
			Err(error_string) => return Err(error_string)
		};

		let json_obj = match configuration_data.as_object() {
			Some(o) => o,
			None => return Err("Error while getting JSON object".to_string())
		};

		new_instance.image_base_url = json_obj.get("images").unwrap().as_object().unwrap().get("secure_base_url").unwrap().to_string();

		Ok(new_instance)
	}

	fn get_url_for(& self, url_string: &'static str) -> String {
		if url_string.contains("?") {
			format!("{}{}&api_key={}", self.base_url, url_string, self.api_key)
		} else {
			format!("{}{}?api_key={}", self.base_url, url_string, self.api_key)
		}
	}

	fn get_json_data_for(& self, url_string: &'static str) -> Result<Json, String> {
		let call_url = self.get_url_for(url_string);

		let mut response = match self.http_client.get(&call_url).send() {
			Ok(r) => r,
			Err(_) => {
				return Err("Error during HTTPS request".to_string());
			}
 		};

		if hyper::Ok != response.status {
			return Err("No HTTPS Status 200 (OK)".to_string());
		}

		let mut result = String::new();
		if response.read_to_string(& mut result).is_err() {
			return Err("Error reading response".to_string());
		}

		let json_data = match Json::from_str(&result) {
			Ok(d) => d,
			Err(_) => return Err("Error while readin JSON data".to_string())
		};

		Ok(json_data)
	}
}

// #[test]
// fn it_works() {
// 	// TODO: Real tests ;)
// 	assert!("Four Rooms" == TheMovieDB { api_key: "6da14fa0b6231874a56ee667a505cdcc" }.test());
// }

fn main() {
    let themdb = TheMovieDB::new("6da14fa0b6231874a56ee667a505cdcc").unwrap();

	println!("{:?}", themdb);
}

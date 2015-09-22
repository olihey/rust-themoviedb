/// for HTTPS
extern crate hyper;
/// to decode JSON data
extern crate rustc_serialize;

use std::fmt;
use std::io::Read;
use std::result::{Result};
use rustc_serialize::json::Json;
use hyper::{Client, Url};

static TESTAPIKEY: &'static str = "YOUR API KEY FROM themoviedb.org";

/// the struct to keep
pub struct TheMovieDB {
	/// the all mighty api key we need for each request
	api_key: &'static str,
	/// the base url for themoviedb.org
	base_url: &'static str,
	/// the base url for all images, comes from the /configuration URL
	image_base_url: String,
	/// the HTTPS client we use for getting data from the API
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
	/// Constructs a new TheMovieDB instance.
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

		let configuration_data = match new_instance.get_json_data_for("/configuration".to_string()) {
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

	fn search(& self, search_term: &'static str) -> Result<String, String> {
		let search_data = match self.get_json_data_for(format!("/search/multi?query={}", search_term)) {
			Ok(d) => d,
			Err(error_string) => return Err(error_string)
		};

		Ok("TEST".to_string())
	}

	/// Returns a URL as String for the given API method
	///
	/// It also automatically adds the api key
	fn get_url_for(& self, url_string: String) -> Result<Url, String> {
		if url_string.contains("?") {
			let result_url = match Url::parse(&format!("{}{}&api_key={}", self.base_url, url_string, self.api_key)) {
				Ok(u) => u,
				Err(_) => return Err(format!("Error buildign URL"))
			};
			Ok(result_url)
		 } else {
			 let result_url = match Url::parse(&format!("{}{}?api_key={}", self.base_url, url_string, self.api_key)) {
 				Ok(u) => u,
 				Err(_) => return Err(format!("Error buildign URL"))
 			};
 			Ok(result_url)
		}
	}

	/// Returns the data for a given API method or an error string if something has failed
	fn get_json_data_for(& self, url_string: String) -> Result<Json, String> {
		let call_url = match self.get_url_for(url_string) {
			Ok(u) => u,
			Err(error_string) => return Err(error_string)
		};

		let call_url_string = format!("{}", call_url);
		let mut response = match self.http_client.get(call_url).send() {
			Ok(r) => r,
			Err(_) => {
				return Err(format!("Error during HTTPS request with URL {}", call_url_string));
			}
 		};

		if hyper::Ok != response.status {
			return Err(format!("No HTTPS Status 200 (OK) for url: {}", call_url_string));
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

#[test]
fn initialization_test() {
	TheMovieDB::new(TESTAPIKEY).unwrap();
}

#[test]
fn search_test() {
	let themoviedb = TheMovieDB::new(TESTAPIKEY).unwrap();
	let search_result = themoviedb.search("The Avengers").unwrap();
}

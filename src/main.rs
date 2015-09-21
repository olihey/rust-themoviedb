extern crate hyper;
extern crate rustc_serialize;

use std::io::Read;
use std::result::{Result};
use rustc_serialize::json::Json;
use hyper::{Client};

struct TheMovieDB {
	api_key: &'static str,
	base_url: &'static str,
	http_client: Client,
}

impl TheMovieDB {
	fn new(themoviedb_api_key: &'static str) -> Result<TheMovieDB, String> {
		let new_instance = TheMovieDB{api_key: themoviedb_api_key,
			base_url: "https://api.themoviedb.org/3",
			http_client: Client::new()};

		let configuration_data = new_instance.get_json_data_for("/configuration");
		println!("{:?}", configuration_data);

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

	fn test(& self) -> String {
		let call_url = format!("https://api.themoviedb.org/3/movie/5wcerweuioybrcqpwyr850?api_key={}", self.api_key);

		let mut res = self.http_client.get(&call_url).send().unwrap();
		assert_eq!(res.status, hyper::Ok);

		let mut result = String::new();
		res.read_to_string(& mut result).unwrap();

		let json_data = Json::from_str(&result).unwrap();
		let json_obj = json_data .as_object().unwrap();

	    json_obj.get("original_title").unwrap().as_string().unwrap().to_string()
	}
}

#[test]
fn it_works() {
	// TODO: Real tests ;)
	assert!("Four Rooms" == TheMovieDB { api_key: "6da14fa0b6231874a56ee667a505cdcc" }.test());
}

fn main() {
    let themdb = TheMovieDB::new("6da14fa0b6231874a56ee667a505cdcc");

    println!("{}", themdb.unwrap().test());
}

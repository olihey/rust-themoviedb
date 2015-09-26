/// for HTTPS
extern crate hyper;
/// to decode JSON data
extern crate rustc_serialize;

static TESTAPIKEY: &'static str = "YOUR API KEY FROM themoviedb.org";

pub mod themoviedb_api {
	use std::fmt;
	use std::io::Read;
	use std::result::{Result};
	use rustc_serialize::json::{Json, Object};
	use hyper;

	/// the struct to keep
	pub struct TheMovieDB {
		/// the all mighty api key we need for each request
		api_key: &'static str,
		/// the base url for themoviedb.org
		base_url: &'static str,
		/// the base url for all images, comes from the /configuration URL
		image_base_url: String,
		/// the HTTPS client we use for getting data from the API
		http_client: hyper::Client,
	}

	#[derive(Default)]
	#[derive(Debug)]
	pub struct TheMovieDBMovie {
		pub title: Option<String>,
		pub original_title: Option<String>
	}

	pub enum TheMovieDBItem {
		Movie(TheMovieDBMovie),
		TVShow {original_title: String}
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
		pub fn new(themoviedb_api_key: &'static str) -> Result<TheMovieDB, String> {
			let mut new_instance = TheMovieDB{api_key: themoviedb_api_key,
				base_url: "https://api.themoviedb.org/3",
				image_base_url: "".to_string(),
				http_client: hyper::Client::new()};

			let configuration_data = try!(new_instance.get_json_data_for("/configuration".to_string()));

			let json_obj = match configuration_data.as_object() {
				Some(o) => o,
				None => return Err("Error while getting JSON object".to_string())
			};

			new_instance.image_base_url = json_obj.get("images").unwrap().as_object().unwrap().get("secure_base_url").unwrap().to_string();

			Ok(new_instance)
		}

		pub fn search(& self, search_term: &'static str, maximum_results: usize) -> Result<Vec<TheMovieDBItem>, String> {
			// we start at the first page
			let mut page = 1;

			// find the higher number between number of results
			// requested and number returned from API
			let mut final_maximum_results = maximum_results;

			// here we save all results and return them
			let mut results: Vec<TheMovieDBItem> = vec![];

			// now loop until we have enough results
			// or if there aren't anymore
			loop {
				// download data
				let search_data = try!(self.get_json_data_for(format!("/search/multi?query={}&page={}", search_term, page)));

				// create a JSON object from the result
				let json_object = match search_data.as_object() {
					Some(o) => o,
					None => return Err("JSON Data is no object".to_string())
				};

				// get the total number of results
				let search_totals = match TheMovieDB::get_object_field_as_u64(json_object, "total_results") {
					Some(r) => r as usize,
					None => return Err("No results returned".to_string())
				};

				// get the total number of pages
				let page_totals = match TheMovieDB::get_object_field_as_u64(json_object, "total_pages") {
					Some(r) => r as usize,
					None => return Err("No page results returned".to_string())
				};

				if final_maximum_results > search_totals {
					final_maximum_results = search_totals;
				}

				// get the total number of results
				let search_results_json = match json_object.get("results") {
					Some(r) => r,
					None => return Err("No results returned".to_string())
				};

				let search_results = match search_results_json.as_array() {
					Some(r) => r,
					None => return Err("total_results not a number?!?!?".to_string())
				};

				for search_result in search_results {
					let result_object = match search_result.as_object() {
						Some(r) => r,
						None => continue
					};

					let mut new_movie: TheMovieDBMovie = Default::default();
					new_movie.original_title = TheMovieDB::get_object_field_as_string(result_object, "original_title");
					new_movie.title = TheMovieDB::get_object_field_as_string(result_object, "title");
					results.push(TheMovieDBItem::Movie(new_movie));

					if results.len() == final_maximum_results {
						return Ok(results);
					}
				}

				if page >= page_totals {
					return Ok(results);
				}

				page += 1;
			};
		}

		fn get_object_field_as_string(json_object: &Object, field_name: &'static str) -> Option<String> {
			let field = match json_object.get(field_name) {
				Some(f) => f,
				None => return None
			};

			let field_value = match field.as_string() {
				Some(v) => v,
				None => return None
			};

			Some(field_value.to_string())
		}

		fn get_object_field_as_u64(json_object: &Object, field_name: &'static str) -> Option<u64> {
			let field = match json_object.get(field_name) {
				Some(f) => f,
				None => return None
			};

			let field_value = match field.as_u64() {
				Some(v) => v,
				None => return None
			};

			Some(field_value)
		}

		/// Returns a URL as String for the given API method
		///
		/// It also automatically adds the api key
		fn get_url_for(& self, url_string: String) -> Result<hyper::Url, String> {
			if url_string.contains("?") {
				let result_url = match hyper::Url::parse(&format!("{}{}&api_key={}", self.base_url, url_string, self.api_key)) {
					Ok(u) => u,
					Err(_) => return Err(format!("Error buildign URL"))
				};
				Ok(result_url)
			 } else {
				 let result_url = match hyper::Url::parse(&format!("{}{}?api_key={}", self.base_url, url_string, self.api_key)) {
	 				Ok(u) => u,
	 				Err(_) => return Err(format!("Error buildign URL"))
	 			};
	 			Ok(result_url)
			}
		}

		/// Returns the data for a given API method or an error string if something has failed
		fn get_json_data_for(& self, url_string: String) -> Result<Json, String> {
			let call_url = try!(self.get_url_for(url_string));

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
}

// #[test]
// fn initialization_test() {
// 	TheMovieDB::new(TESTAPIKEY).unwrap();
// }

#[test]
fn search_test() {
	let themoviedb = themoviedb_api::TheMovieDB::new(TESTAPIKEY).unwrap();
	let search_result = themoviedb.search("Tinker Bell", 25).unwrap();

//	assert!(303 == search_result.len());

	for item in search_result {
		match item {
			themoviedb_api::TheMovieDBItem::Movie(movie) => {
				if movie.title.is_some() {
					println!("Found movie: {}", movie.title.unwrap());
				}
				if movie.original_title.is_some() {
					println!("Found original movie: {}", movie.original_title.unwrap());
				}
			},
			_ => continue
		}
	}
}

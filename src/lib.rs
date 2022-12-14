//! # Aspiration
//! Unofficial library for [Holiday API](https://holidayapi.com) written in Rust. This repo implements interface for original HolidayAPI endpoints seen [here](https://holidayapi.com/docs).
//! ## Acknowledgments
//! This project is heavily inspired by [holidayapi-node](https://github.com/holidayapi/holidayapi-node) and [holiday-api-rust](https://github.com/guibranco/holiday-api-rust) repositories.
//!
//! ## Installation
//! ```console
//! $ cargo add holidayapi_rust
//! ```
//! ## Usage
//! ```
//! use holidayapi_rust::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//! 	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
//! 	let request = api.holidays("us", 2021).month(10).day(20).public().upcoming();
//! 	let response = request.get().await;
//!		match response {
//! 		Ok(_) => { /* */ },
//! 		Err(_) => { /* */ },
//! 	}
//! }
//! ```
pub mod prelude;

mod requests;
mod responses;
use requests::Request;
use responses::{
    CountriesResponse, HolidaysResponse, LanguagesResponse, WorkdayResponse, WorkdaysResponse,
};
use serde_json::Value;
use std::{collections::HashMap, error::Error, fmt};

use regex::Regex;
pub use reqwest::Response;
use reqwest::Url;

#[derive(Debug, Clone)]
pub struct HolidayAPI {
    base_url: String,
    key: String,
}

#[derive(Debug)]
pub enum HolidayAPIError {
    InvalidKeyFormat(String),
    InvalidOrExpiredKey(String),
    InvalidVersion(String),
    RequestError(reqwest::Error, String),
}

impl fmt::Display for HolidayAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HolidayAPIError::InvalidKeyFormat(key) => write!(f, "Invalid key: {}", key),
            HolidayAPIError::InvalidVersion(version) => write!(f, "Invalid version: {}", version),
            HolidayAPIError::InvalidOrExpiredKey(key) => {
                write!(f, "Invalid or expired key: {}", key)
            }
            HolidayAPIError::RequestError(req, err) => {
                write!(
                    f,
                    "{}: {}\nRaw url: '{}'",
                    req.status().unwrap(),
                    err,
                    req.url().unwrap(),
                )
            }
        }
    }
}
impl Error for HolidayAPIError {}

impl HolidayAPI {
    pub fn is_valid_key(key: &str) -> Result<(), HolidayAPIError> {
        let uuid_regex =
            Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
                .expect("Regex is correct");

        if uuid_regex.is_match(key) {
            Ok(())
        } else {
            Err(HolidayAPIError::InvalidKeyFormat(key.into()))
        }
    }

    pub fn is_valid_version(version: &i32) -> Result<(), HolidayAPIError> {
        let valid_versions = [1];
        if !valid_versions.contains(version) {
            Err(HolidayAPIError::InvalidVersion(format!(
                "Invalid version: {}, please choose: {:?}",
                version, valid_versions
            )))
        } else {
            Ok(())
        }
    }
    fn construct_api(key: &str, version: i32) -> HolidayAPI {
        HolidayAPI {
            base_url: format!("https://holidayapi.com/v{}/", version),
            key: key.to_owned(),
        }
    }
    /// Construct a new holiday API
    ///
    /// # Errors
    ///
    /// Will return an `Err` if the given key is not plausibly a valid one.
    ///
    /// # Examples
    ///
    /// Basic usage
    ///
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    /// let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// ```
    pub fn new(key: &str) -> Result<HolidayAPI, HolidayAPIError> {
        Self::is_valid_key(key)?;

        Ok(Self::construct_api(key, 1))
    }

    /// Construct a new holiday API
    ///
    /// # Errors
    ///
    /// Will return an `Err` if the given key is not plausibly a valid one. Or the api version is invalid.
    /// Current valid versions: `[1]`
    ///
    /// # Examples
    ///
    /// Basic usage
    ///
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    /// let api = HolidayAPI::with_version("00000000-0000-0000-0000-000000000000", 1).unwrap();
    /// ```
    pub fn with_version(key: &str, version: i32) -> Result<HolidayAPI, HolidayAPIError> {
        Self::is_valid_key(key)?;
        Self::is_valid_version(&version)?;

        Ok(Self::construct_api(key, version))
    }

    /// Make a custom request.
    /// # Examples
    ///
    /// Basic usage
    ///
    /// ```
    /// use holidayapi_rust::{ HolidayAPI };
    /// use std::collections::HashMap;
    ///
    /// let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let _future = api.custom_request("countries", HashMap::new());
    /// ```
    pub async fn custom_request(
        &self,
        endpoint: &str,
        parameters: HashMap<String, String>,
    ) -> Result<Response, HolidayAPIError> {
        let client = reqwest::Client::new();
        let url = Url::parse(self.base_url.as_str()).unwrap();
        let url = url.join(endpoint.to_ascii_lowercase().as_str()).unwrap();
        let url = Url::parse_with_params(&format!("{}?key={}", url, self.key), parameters)
            .expect("Parameters are invalid");
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| HolidayAPIError::RequestError(e, "".to_string()))?;

        match response.error_for_status_ref() {
            Ok(_) => Ok(response),
            Err(err) => {
                let val = serde_json::from_str::<Value>(&response.text().await.unwrap())
                    .expect("Error response to be JSON");
                let o = val.as_object();
                let error = o.and_then(|o| o.get("error")).unwrap();

                Err(HolidayAPIError::RequestError(
                    err,
                    error.as_str().unwrap().into(),
                ))
            }
        }
    }

    /// Generates a minimal `countries` request and returns it.
    ///
    /// # Examples
    ///
    /// Basic usage
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let request = api.countries();
    /// ```
    ///
    /// Adding optional parameters with builder pattern
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let specific_request = api.countries().search("united states").public();
    /// ```
    pub fn countries(&self) -> Request<CountriesResponse> {
        Request::<CountriesResponse>::new(self)
    }

    /// Generates a minimal `holidays` request and returns it.
    ///
    /// # Examples
    ///
    /// Basic usage
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let request = api.holidays("us", 2020);
    /// ```
    ///
    /// Adding optional parameters with builder pattern
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let specific_request = api.holidays("us", 2020).month(12).upcoming();
    /// ```
    pub fn holidays(&self, country: &str, year: i32) -> Request<HolidaysResponse> {
        Request::<HolidaysResponse>::new(self, country.into(), year)
    }

    /// Generates a minimal `workday` request and returns it.
    ///
    /// # Examples
    ///
    /// Basic usage
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let request = api.workday("us","YYYY-MM-DD", 100);
    /// ```
    pub fn workday(&self, country: &str, start: &str, days: i32) -> Request<WorkdayResponse> {
        Request::<WorkdayResponse>::new(self, country.into(), start, days)
    }

    /// Generates a minimal `workdays` request and returns it.
    ///
    /// # Examples
    ///
    /// Basic usage
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let request = api.workdays("us", "YYYY-MM-DD", "YYYY-MM-DD");
    /// ```
    pub fn workdays(&self, country: &str, start: &str, days: &str) -> Request<WorkdaysResponse> {
        Request::<WorkdaysResponse>::new(self, country, start, days)
    }

    /// Generates a minimal `languages` request and returns it.
    ///
    /// # Examples
    ///
    /// Basic usage
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    ///	let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let request = api.languages();
    /// ```
    ///
    /// Adding optional parameters with builder pattern
    /// ```
    /// use holidayapi_rust::prelude::*;
    ///
    /// let api = HolidayAPI::new("00000000-0000-0000-0000-000000000000").unwrap();
    /// let specific_request = api.languages().search("united states");
    /// ```
    pub fn languages(&self) -> Request<LanguagesResponse> {
        Request::<LanguagesResponse>::new(self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    static EXPIRED_KEY: &str = "daaaaaab-aaaa-aaaa-aaaa-2aaaada37e14";
    static INVALID_KEY: &str = "invalid-key-format";

    #[test]
    fn test_valid_key() {
        match HolidayAPI::new(EXPIRED_KEY) {
            Ok(_) => assert!(true),
            Err(_) => unreachable!("Should not return an error on valid key"),
        }
        match HolidayAPI::new(INVALID_KEY) {
            Ok(_) => unreachable!("Should return an error on invalid key"),
            Err(_) => assert!(true),
        }
    }

    #[tokio::test]
    async fn test_countries() {
        let api = HolidayAPI::new(EXPIRED_KEY).unwrap();
        match api.countries().get().await {
            Ok(o) => println!("{:?}", o),
            Err(o) => println!("{}", o),
        }
    }
}

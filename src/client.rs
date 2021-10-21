use chrono::{DateTime, Duration, Utc};
use crate::error::*;
use reqwest;
pub use reqwest::{Method, blocking::RequestBuilder, blocking::Response, StatusCode};
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::sync::Mutex;
use std::io::Read;

const ENDPOINT: &'static str = "https://merchant-api.jet.com/api";

#[derive(Debug, Deserialize)]
pub(crate) struct Token {
  id_token: String,
  token_type: String,
  expires_on: DateTime<Utc>,
}

pub struct ClientOptions {
  pub api_user: String,
  pub secret: String,
  pub merchant_id: String,
}

pub struct Client {
  options: ClientOptions,
  token: Mutex<RefCell<Option<Token>>>,
  client: reqwest::blocking::Client,
}

impl Client {
  pub fn new(opts: ClientOptions) -> Result<Client> {
    Ok(Client {
      options: opts,
      token: Mutex::new(RefCell::new(None)),
      client: reqwest::blocking::Client::new(),
    })
  }

  pub fn with_http_client(opts: ClientOptions, http_client: reqwest::blocking::Client) -> Client {
    Client {
      options: opts,
      token: Mutex::new(RefCell::new(None)),
      client: http_client,
    }
  }

  pub(crate) fn with_token<T, F>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&Token) -> Result<T>,
  {
    let guard = self.token.lock().expect("lock token");
    let token: &mut Option<Token> = &mut guard.borrow_mut();
    match *token {
      Some(ref token) if token.expires_on - Duration::minutes(15) >= Utc::now() => f(&token),
      _ => {
        token.replace(self.get_token()?);
        f(&token.as_ref().unwrap())
      }
    }
  }

  fn get_token(&self) -> Result<Token> {
    #[derive(Serialize)]
    pub struct TokenRequest<'a> {
      pub user: &'a str,
      pub pass: &'a str,
    }

    let mut res = self
      .client
      .post(&format!("{}/token", ENDPOINT))
      .json(&TokenRequest {
        user: &self.options.api_user,
        pass: &self.options.secret,
      })
      .send()?;

    if !res.status().is_success() {
      let mut body = String::new();
      res.read_to_string(&mut body)?;
      return Err(Error::GetTokenRequest { status: res.status(), body });
    }

    res.json().map_err(Into::into)
  }

  pub(crate) fn request<T, F>(&self, method: Method, path: &str, f: F) -> Result<T>
  where
    T: DeserializeOwned,
    F: FnOnce(RequestBuilder) -> RequestBuilder,
  {
    use headers::{HeaderMapExt, Authorization};
    use reqwest::header::HeaderMap;

    let mut req = self.with_token(|token| -> Result<RequestBuilder> {
      let mut req = self
        .client
        .request(method, &format!("{}{}", ENDPOINT, path));
      req = req.headers({
        let mut map = HeaderMap::new();
        map.typed_insert(Authorization::bearer(&token.id_token).map_err(|_| Error::InvalidBearerToken)?);
        map
      });
      Ok(req)
    })?;

    req = f(req);

    let mut res = req.send()?;

    if !res.status().is_success() {
      let mut body = String::new();
      res.read_to_string(&mut body)?;
      return Err(Error::Request { path: path.to_owned(), status: res.status(), body });
    }

    res.json().map_err(Into::into)
  }

  pub(crate) fn request_no_content<F>(&self, method: Method, path: &str, f: F) -> Result<()>
  where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
  {
    use headers::{HeaderMapExt, Authorization};
    use reqwest::header::HeaderMap;

    let mut req = self.with_token(|token| -> Result<RequestBuilder> {
      let mut req = self
        .client
        .request(method, &format!("{}{}", ENDPOINT, path));
      req = req.headers({
        let mut map = HeaderMap::new();
        map.typed_insert(Authorization::bearer(&token.id_token).map_err(|_| Error::InvalidBearerToken)?);
        map
      });
      Ok(req)
    })?;

    req = f(req);

    let mut res = req.send()?;

    if !res.status().is_success() {
      let mut body = String::new();
      res.read_to_string(&mut body)?;
      return Err(Error::Request{ path: path.to_owned(), status: res.status(), body });
    }

    Ok(())
  }
}

#[cfg(test)]
pub(crate) fn get_test_client() -> Client {
  use dotenv::dotenv;
  use std::env;
  dotenv().ok();

  Client::new(ClientOptions {
    api_user: env::var("API_USER").unwrap(),
    secret: env::var("SECRET").unwrap(),
    merchant_id: env::var("MERCHANT_ID").unwrap(),
  }).unwrap()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_with_token() {
    let client = get_test_client();
    let mut last_token: Option<String> = None;
    client
      .with_token(|token| -> Result<()> {
        last_token = Some(token.id_token.clone());
        Ok(())
      })
      .unwrap();

    client
      .with_token(|token| -> Result<()> {
        assert_eq!(&token.id_token, last_token.as_ref().unwrap());
        Ok(())
      })
      .unwrap();
  }
}

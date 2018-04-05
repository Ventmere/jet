use chrono::{DateTime, Duration, Utc};
use error::*;
use reqwest;
pub use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::io::Read;
use std::sync::Mutex;

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
  client: reqwest::Client,
}

impl Client {
  pub fn new(opts: ClientOptions) -> Result<Client> {
    Ok(Client {
      options: opts,
      token: Mutex::new(RefCell::new(None)),
      client: reqwest::Client::new(),
    })
  }

  pub fn with_http_client(opts: ClientOptions, http_client: reqwest::Client) -> Client {
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
        use std::mem::replace;
        replace(token, Some(self.get_token()?));
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
      return Err(ErrorKind::GetTokenRequest(res.status(), body).into());
    }

    res.json().chain_err(|| ErrorKind::InvalidResponse)
  }

  pub(crate) fn request<T, F>(&self, method: Method, path: &str, f: F) -> Result<T>
  where
    T: DeserializeOwned,
    F: FnOnce(&mut RequestBuilder) -> Result<()>,
  {
    use reqwest::header::{Authorization, Bearer};

    let mut req = self.with_token(|token| -> Result<RequestBuilder> {
      let mut req = self
        .client
        .request(method, &format!("{}{}", ENDPOINT, path));
      req.header(Authorization(Bearer {
        token: token.id_token.clone(),
      }));
      Ok(req)
    })?;

    f(&mut req)?;

    let mut res = req.send()?;

    if !res.status().is_success() {
      let mut body = String::new();
      res.read_to_string(&mut body)?;
      return Err(ErrorKind::Request(path.to_owned(), res.status(), body).into());
    }

    res.json().chain_err(|| ErrorKind::InvalidResponse)
  }

  pub(crate) fn request_no_content<F>(&self, method: Method, path: &str, f: F) -> Result<()>
  where
    F: FnOnce(&mut RequestBuilder) -> Result<()>,
  {
    use reqwest::header::{Authorization, Bearer};

    let mut req = self.with_token(|token| -> Result<RequestBuilder> {
      let mut req = self
        .client
        .request(method, &format!("{}{}", ENDPOINT, path));
      req.header(Authorization(Bearer {
        token: token.id_token.clone(),
      }));
      Ok(req)
    })?;

    f(&mut req)?;

    let mut res = req.send()?;

    if !res.status().is_success() {
      let mut body = String::new();
      res.read_to_string(&mut body)?;
      return Err(ErrorKind::Request(path.to_owned(), res.status(), body).into());
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

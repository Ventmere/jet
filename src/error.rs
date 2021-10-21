use thiserror::Error;
use reqwest::StatusCode;

#[derive(Debug, Error)]
pub enum Error {
  #[error("get token request error: status = '{status}', body = '{body}'")]
  GetTokenRequest {
    status: StatusCode, 
    body: String
  },
  #[error("request error: path = '{path}', status = '{status}', body = '{body}'")]
  Request {
    path: String, 
    status: StatusCode, 
    body: String
  },
  #[error("invalid bearer token")]
  InvalidBearerToken,
  #[error("json: {0}")]
  Json(#[from] serde_json::Error),
  #[error("http: {0}")]
  Http(#[from] reqwest::Error),
  #[error("io: {0}")]
  Io(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
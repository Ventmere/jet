use reqwest::StatusCode;

error_chain! {
  errors {
    GetTokenRequest(status: StatusCode, body: String) {
      description("get token request error")
      display("get token request error: status = '{}', body = '{}'", status, body)
    }

    Request(path: String, status: StatusCode, body: String) {
      description("request error")
      display("request error: path = '{}', status = '{}', body = '{}'", path, status, body)
    }

    InvalidResponse
  }

  foreign_links {
    Http(::reqwest::Error);
    Io(::std::io::Error);
  }
}
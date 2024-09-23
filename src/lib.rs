use async_trait::async_trait;
// use custom_logger::*;
use mirror_error::MirrorError;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, StatusCode};

#[derive(Debug, Clone)]
pub struct ImplQueryImageInterface {}

#[async_trait]
pub trait QueryImageInterface {
    // used to interact with container registry
    // depending on the url we can get
    // - all catalogs
    // - list tags for a specific component
    // - get a reference digest (from header) for a specific manifest
    async fn get_details(&self, url: String, token: String) -> Result<String, MirrorError>;
}
#[async_trait]
impl QueryImageInterface for ImplQueryImageInterface {
    async fn get_details(&self, url: String, token: String) -> Result<String, MirrorError> {
        let client = Client::new();
        let mut header_map: HeaderMap = HeaderMap::new();
        header_map.insert(USER_AGENT, HeaderValue::from_static("image-mirror"));
        header_map.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("{} {}", "Bearer", token)).unwrap(),
        );
        header_map.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.docker.distribution.manifest.list.v2+json,application/vnd.oci.image.index.v1+json,application/vnd.oci.image.manifest.v1+json"),
        );
        header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        // check without token
        if token.len() == 0 {
            let put_url = url.replace("https", "http");
            let res = client.get(put_url).headers(header_map.clone()).send().await;
            if res.is_ok() && res.as_ref().unwrap().status() == StatusCode::OK {
                let body = res.unwrap().text().await;
                if body.is_ok() {
                    Ok(body.unwrap())
                } else {
                    let err = MirrorError::new(&format!(
                        "[get_details] could not read body contents {}",
                        body.err().unwrap().to_string().to_lowercase()
                    ));
                    Err(err)
                }
            } else {
                let err =
                    MirrorError::new(&format!("[get_details] {}", res.as_ref().unwrap().status()));
                Err(err)
            }
        } else {
            let res = client.get(url).headers(header_map.clone()).send().await;
            if res.is_ok() && res.as_ref().unwrap().status() == StatusCode::OK {
                let body = res.unwrap().text().await;
                if body.is_ok() {
                    Ok(body.unwrap())
                } else {
                    let err = MirrorError::new(&format!(
                        "[get_details] could not read body contents {}",
                        body.err().unwrap().to_string().to_lowercase()
                    ));
                    Err(err)
                }
            } else {
                let err =
                    MirrorError::new(&format!("[get_details] {}", res.as_ref().unwrap().status()));
                Err(err)
            }
        }
    }
}
#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    // this brings everything from parent's scope into this scope
    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }
    #[test]
    fn get_manifest_pass() {
        let mut server = mockito::Server::new();
        let url = server.url();

        // Create a mock
        server
            .mock("GET", "/v2/manifests")
            .with_status(200)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/vnd.docker.distribution.manifest.list.v2+json,application/vnd.oci.image.index.v1+json,application/vnd.oci.image.manifest.v1+json")
            .with_body("{ \"test\": \"hello-world\" }")
            .create();

        let fake = ImplQueryImageInterface {};

        let res = aw!(fake.get_details(url.clone() + "/v2/manifests", String::from("token")));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), String::from("{ \"test\": \"hello-world\" }"));
    }
    #[test]
    fn get_manifest_fail() {
        let mut server = mockito::Server::new();
        let url = server.url();

        // Create a mock
        server
            .mock("GET", "/v2/manifests")
            .with_status(500)
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/vnd.docker.distribution.manifest.list.v2+json,application/vnd.oci.image.index.v1+json,application/vnd.oci.image.manifest.v1+json")
            .create();

        let fake = ImplQueryImageInterface {};

        let res = aw!(fake.get_details(url.clone() + "/v2/manifests", String::from("")));
        assert!(res.is_err());
    }
}

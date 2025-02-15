use async_trait::async_trait;
use mirror_error::MirrorError;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, StatusCode};

#[derive(Debug, Clone)]
pub struct ImplQueryImageInterface {}

#[derive(Debug, Clone, PartialEq)]
pub struct ResponseData {
    pub data: String,
    pub link: String,
}

#[async_trait]
pub trait QueryImageInterface {
    // used to interact with container registry
    // depending on the url we can get
    // - all catalogs
    // - list tags for a specific component
    // - get a reference digest (from header) for a specific manifest
    async fn get_details(
        &self,
        url: String,
        token: String,
        e_tag: bool,
    ) -> Result<ResponseData, MirrorError>;
}
#[async_trait]
impl QueryImageInterface for ImplQueryImageInterface {
    async fn get_details(
        &self,
        url: String,
        token: String,
        e_tag: bool,
    ) -> Result<ResponseData, MirrorError> {
        let client = Client::new();
        let mut header_map: HeaderMap = HeaderMap::new();
        header_map.insert(USER_AGENT, HeaderValue::from_static("image-mirror"));
        header_map.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        header_map.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.docker.distribution.manifest.list.v2+json,application/vnd.oci.image.index.v1+json,application/vnd.oci.image.manifest.v1+json"),
        );
        header_map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let get_url = if token.is_empty() {
            // check without token
            url.replace("https", "http")
        } else {
            url
        };
        let res = client
            .get(get_url)
            .headers(header_map)
            .send()
            .await
            .map_err(|e| MirrorError::new(&format!("[get_details] {e}")))?;
        if res.status() == StatusCode::OK {
            let headers = res.headers();
            if e_tag {
                let e_tag = headers.get("docker-content-digest").unwrap();
                Ok(ResponseData {
                    data: e_tag.to_str().unwrap().to_string(),
                    link: "".to_string(),
                })
            } else {
                let link_info = headers
                    .get("link")
                    .map(|l| {
                        l.to_str()
                            .unwrap()
                            .replace(['<', '>'], "")
                            .replace("; rel=\"next\"", "")
                    })
                    .unwrap_or_default();
                let body = res.text().await.map_err(|e| {
                    MirrorError::new(&format!(
                        "[get_details] could not read body contents {}",
                        e.to_string().to_lowercase()
                    ))
                })?;
                Ok(ResponseData {
                    data: body,
                    link: link_info,
                })
            }
        } else {
            Err(MirrorError::new(&format!("[get_details] {}", res.status())))
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

        let res = aw!(fake.get_details(url + "/v2/manifests", String::from("token"), false));
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap().data,
            String::from("{ \"test\": \"hello-world\" }")
        );
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

        let res = aw!(fake.get_details(url + "/v2/manifests", String::from(""), false));
        assert!(res.is_err());
    }
}

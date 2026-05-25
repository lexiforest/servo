/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bimp_net::{Body as BimpNetBody, Client, Config, Error, RedirectPolicy};
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode};
use http_body_util::{Full, combinators::BoxBody};
use hyper::body::{Body, Bytes, Frame, SizeHint};
use log::{debug, warn};
use net_traits::NetworkError;
use servo_url::ServoUrl;

const DEFAULT_CHROME_TARGET: &str = "chrome136";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct CurlImpersonateResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: BoxBody<Bytes, hyper::Error>,
}

pub fn supports_url(url: &ServoUrl) -> bool {
    matches!(url.scheme(), "http" | "https")
}

pub async fn send(
    url: ServoUrl,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    user_agent: String,
) -> Result<CurlImpersonateResponse, NetworkError> {
    let target = curl_target_for_user_agent(&user_agent);
    let client = Client::new(Config {
        impersonation_target: target.clone(),
        connect_timeout: CONNECT_TIMEOUT,
        request_timeout: REQUEST_TIMEOUT,
        redirect_policy: RedirectPolicy::None,
        default_headers: true,
    });

    let request = build_request(&url, method.clone(), headers.clone(), body.clone())?;
    match client.send(request).await {
        Ok(response) => {
            debug!("curl-impersonate {} -> {}", url, response.status());
            Ok(convert_response(response))
        },
        Err(error) if target != DEFAULT_CHROME_TARGET => {
            warn!(
                "curl-impersonate target `{target}` failed, retrying `{DEFAULT_CHROME_TARGET}`: {error}"
            );
            let client = Client::new(Config {
                impersonation_target: DEFAULT_CHROME_TARGET.to_string(),
                connect_timeout: CONNECT_TIMEOUT,
                request_timeout: REQUEST_TIMEOUT,
                redirect_policy: RedirectPolicy::None,
                default_headers: true,
            });
            let request = build_request(&url, method, headers, body)?;
            client
                .send(request)
                .await
                .map(convert_response)
                .map_err(network_error)
        },
        Err(error) => Err(network_error(error)),
    }
}

fn build_request(
    url: &ServoUrl,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
) -> Result<Request<Full<Bytes>>, NetworkError> {
    let top_level_navigation = is_top_level_navigation(&headers);
    let mut request = Request::builder()
        .method(method)
        .uri(url.as_str())
        .body(Full::new(body.map(Bytes::from).unwrap_or_else(Bytes::new)))
        .map_err(|error| NetworkError::ResourceLoadError(error.to_string()))?;

    for (name, value) in headers {
        let Some(name) = name else {
            continue;
        };
        if should_forward_header_to_curl(&name, top_level_navigation) {
            request.headers_mut().append(name, value);
        }
    }
    if !top_level_navigation {
        suppress_navigation_only_default_headers(request.headers_mut());
    }

    Ok(request)
}

fn convert_response(response: http::Response<BimpNetBody>) -> CurlImpersonateResponse {
    let (parts, body) = response.into_parts();
    CurlImpersonateResponse {
        status: parts.status,
        headers: parts.headers,
        body: BoxBody::new(ServoBody { inner: body }),
    }
}

struct ServoBody {
    inner: BimpNetBody,
}

impl Body for ServoBody {
    type Data = Bytes;
    type Error = hyper::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match Pin::new(&mut self.inner).poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
            Poll::Ready(Some(Err(error))) => {
                warn!("curl-impersonate body stream ended with an error: {error}");
                Poll::Ready(None)
            },
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

fn is_top_level_navigation(headers: &HeaderMap) -> bool {
    headers
        .get("sec-fetch-mode")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("navigate"))
}

fn should_forward_header_to_curl(name: &HeaderName, top_level_navigation: bool) -> bool {
    match name.as_str() {
        "host" | "connection" | "user-agent" | "accept-encoding" | "content-length" => false,
        "accept" |
        "accept-language" |
        "upgrade-insecure-requests" |
        "sec-fetch-dest" |
        "sec-fetch-mode" |
        "sec-fetch-site" |
        "sec-fetch-user" |
        "sec-ch-ua" |
        "sec-ch-ua-mobile" |
        "sec-ch-ua-platform" |
        "sec-ch-ua-arch" |
        "sec-ch-ua-bitness" |
        "sec-ch-ua-platform-version" |
        "sec-ch-ua-full-version-list" |
        "sec-ch-ua-model" |
        "sec-ch-device-memory" => !top_level_navigation,
        _ => true,
    }
}

fn suppress_navigation_only_default_headers(headers: &mut HeaderMap) {
    headers.insert(
        HeaderName::from_static("upgrade-insecure-requests"),
        HeaderValue::from_static(""),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-user"),
        HeaderValue::from_static(""),
    );
}

fn curl_target_for_user_agent(user_agent: &str) -> String {
    major_version_after_token(user_agent, "Chrome/")
        .or_else(|| major_version_after_token(user_agent, "Chromium/"))
        .or_else(|| major_version_after_token(user_agent, "CriOS/"))
        .map(|major| format!("chrome{major}"))
        .unwrap_or_else(|| DEFAULT_CHROME_TARGET.to_string())
}

fn major_version_after_token(value: &str, token: &str) -> Option<u16> {
    value
        .split_once(token)
        .and_then(|(_, rest)| rest.split(['.', ' ', ';', ')']).next())
        .and_then(|major| major.parse::<u16>().ok())
        .filter(|major| *major > 0)
}

fn network_error(error: Error) -> NetworkError {
    NetworkError::ResourceLoadError(format!("curl-impersonate failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curl_target_tracks_chrome_user_agent() {
        assert_eq!(
            curl_target_for_user_agent("Mozilla/5.0 Chrome/142.0.0.0 Safari/537.36"),
            "chrome142"
        );
    }

    #[test]
    fn skips_fingerprint_headers_so_curl_can_use_native_defaults() {
        assert!(!should_forward_header_to_curl(
            &HeaderName::from_static("user-agent"),
            true
        ));
        assert!(!should_forward_header_to_curl(
            &HeaderName::from_static("sec-ch-ua"),
            true
        ));
        assert!(!should_forward_header_to_curl(
            &HeaderName::from_static("accept"),
            true
        ));
        assert!(should_forward_header_to_curl(
            &HeaderName::from_static("sec-fetch-mode"),
            false
        ));
        assert!(should_forward_header_to_curl(
            &HeaderName::from_static("cookie"),
            true
        ));
        assert!(should_forward_header_to_curl(
            &HeaderName::from_static("content-type"),
            true
        ));
    }

    #[test]
    fn subresource_requests_suppress_navigation_only_defaults() {
        let mut headers = HeaderMap::new();
        suppress_navigation_only_default_headers(&mut headers);

        assert_eq!(
            headers
                .get("upgrade-insecure-requests")
                .and_then(|value| value.to_str().ok()),
            Some("")
        );
        assert_eq!(
            headers
                .get("sec-fetch-user")
                .and_then(|value| value.to_str().ok()),
            Some("")
        );
    }
}

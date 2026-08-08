/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bimp_net::{Body as BimpNetBody, Client, Config, Error, Proxy, RedirectPolicy};
use http::{HeaderMap, Method, Request, StatusCode};
use http_body_util::{Full, combinators::BoxBody};
use hyper::body::{Body, Bytes, Frame, SizeHint};
use log::{debug, warn};
use net_traits::NetworkError;
use servo_base::id::WebViewId;
use servo_url::ServoUrl;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct BimpNetResponse {
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
    target_webview_id: Option<WebViewId>,
) -> Result<BimpNetResponse, NetworkError> {
    let network_config = target_webview_id.and_then(net_traits::bimp_network_config_for_webview);
    let target = network_config
        .as_ref()
        .map(|config| config.impersonation_profile.clone())
        .unwrap_or_else(|| servo_config::pref!(bimp_network_impersonation_profile));
    if target.trim().is_empty() {
        return Err(NetworkError::ResourceLoadError(
            "bimp-net request has no resolved impersonate browser profile".to_string(),
        ));
    }
    let proxy = network_config
        .as_ref()
        .and_then(|config| config.proxy_url.as_deref())
        .map(Proxy::parse)
        .transpose()
        .map_err(|error| {
            NetworkError::ResourceLoadError(format!("invalid bimp-net proxy: {error}"))
        })?;
    let connect_timeout = network_config
        .as_ref()
        .and_then(|config| config.connect_timeout_ms)
        .map(Duration::from_millis)
        .unwrap_or(CONNECT_TIMEOUT);
    let request_timeout = network_config
        .as_ref()
        .and_then(|config| config.request_timeout_ms)
        .map(Duration::from_millis)
        .unwrap_or(REQUEST_TIMEOUT);
    let client = Client::new(Config {
        impersonation_target: target,
        connect_timeout,
        request_timeout,
        redirect_policy: RedirectPolicy::None,
        default_headers: false,
        proxy,
    });

    let request = build_request(&url, method, headers, body)?;
    let response = client.send(request).await.map_err(network_error)?;
    debug!("bimp-net {} -> {}", url, response.status());
    Ok(convert_response(response))
}

fn build_request(
    url: &ServoUrl,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
) -> Result<Request<Full<Bytes>>, NetworkError> {
    let mut request = Request::builder()
        .method(method)
        .uri(url.as_str())
        .body(Full::new(body.map(Bytes::from).unwrap_or_else(Bytes::new)))
        .map_err(|error| NetworkError::ResourceLoadError(error.to_string()))?;

    for (name, value) in headers {
        let Some(name) = name else {
            continue;
        };
        request.headers_mut().append(name, value);
    }

    Ok(request)
}

fn convert_response(response: http::Response<BimpNetBody>) -> BimpNetResponse {
    let (parts, body) = response.into_parts();
    BimpNetResponse {
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
                warn!("bimp-net body stream ended with an error: {error}");
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

fn network_error(error: Error) -> NetworkError {
    NetworkError::ResourceLoadError(format!("bimp-net failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_all_servo_headers_unchanged() {
        let mut headers = HeaderMap::new();
        headers.insert("user-agent", "persona-agent".parse().unwrap());
        headers.insert("accept", "text/html".parse().unwrap());
        headers.insert("cookie", "session=1".parse().unwrap());

        let request = build_request(
            &ServoUrl::parse("https://example.test/").unwrap(),
            Method::GET,
            headers,
            None,
        )
        .unwrap();

        assert_eq!(
            request.headers().get("user-agent").unwrap(),
            "persona-agent"
        );
        assert_eq!(request.headers().get("accept").unwrap(), "text/html");
        assert_eq!(request.headers().get("cookie").unwrap(), "session=1");
    }
}

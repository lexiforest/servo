/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::io::Write;
use std::process::{Command, Stdio};

use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use log::{debug, warn};
use net_traits::NetworkError;
use servo_url::ServoUrl;

const DEFAULT_CHROME_TARGET: &str = "curl_chrome136";
const REQUEST_TIMEOUT_SECONDS: &str = "30";
const CONNECT_TIMEOUT_SECONDS: &str = "10";

pub struct CurlImpersonateResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
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
    tokio::task::spawn_blocking(move || send_blocking(&url, &method, &headers, body, &user_agent))
        .await
        .map_err(|error| NetworkError::ResourceLoadError(format!("curl worker failed: {error}")))?
}

fn send_blocking(
    url: &ServoUrl,
    method: &Method,
    headers: &HeaderMap,
    body: Option<Vec<u8>>,
    user_agent: &str,
) -> Result<CurlImpersonateResponse, NetworkError> {
    let command = curl_command_for_user_agent(user_agent);
    match run_command(&command, url, method, headers, body.as_deref()) {
        Ok(response) => Ok(response),
        Err(error) if command != DEFAULT_CHROME_TARGET => {
            warn!(
                "curl-impersonate command `{command}` failed, retrying `{DEFAULT_CHROME_TARGET}`: {error:?}"
            );
            run_command(DEFAULT_CHROME_TARGET, url, method, headers, body.as_deref())
        },
        Err(error) => Err(error),
    }
}

fn run_command(
    command: &str,
    url: &ServoUrl,
    method: &Method,
    headers: &HeaderMap,
    body: Option<&[u8]>,
) -> Result<CurlImpersonateResponse, NetworkError> {
    let mut process = Command::new(command);
    process
        .arg("-sS")
        .arg("--http2")
        .arg("--path-as-is")
        .arg("--max-time")
        .arg(REQUEST_TIMEOUT_SECONDS)
        .arg("--connect-timeout")
        .arg(CONNECT_TIMEOUT_SECONDS)
        .arg("-D")
        .arg("-")
        .arg("-o")
        .arg("-");

    match (method, body) {
        (&Method::GET, None) => {},
        (&Method::HEAD, None) => {
            process.arg("-I");
        },
        (_, Some(_)) => {
            process.arg("-X").arg(method.as_str());
            process.arg("--data-binary").arg("@-");
            process.stdin(Stdio::piped());
        },
        _ => {
            process.arg("-X").arg(method.as_str());
        },
    }

    let top_level_navigation = is_top_level_navigation(headers);
    for (name, value) in headers {
        if !should_forward_header_to_curl(name, top_level_navigation) {
            continue;
        }
        let Ok(value) = value.to_str() else {
            continue;
        };
        process.arg("-H").arg(format!("{}: {value}", name.as_str()));
    }
    if !top_level_navigation {
        suppress_navigation_only_default_headers(&mut process);
    }

    process.arg(url.as_str()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = process
        .spawn()
        .map_err(|error| NetworkError::ResourceLoadError(format!("{command} failed to start: {error}")))?;

    if let Some(body) = body {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| NetworkError::ResourceLoadError(format!("{command} stdin unavailable")))?;
        stdin
            .write_all(body)
            .map_err(|error| NetworkError::ResourceLoadError(format!("{command} body write failed: {error}")))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| NetworkError::ResourceLoadError(format!("{command} failed: {error}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(NetworkError::ResourceLoadError(format!(
            "{command} exited with {}: {stderr}",
            output.status
        )));
    }

    let mut response = parse_curl_output(&output.stdout)?;
    normalize_decoded_response_headers(&mut response.headers);
    debug!(
        "curl-impersonate {} {} -> {} ({} bytes)",
        method,
        url,
        response.status,
        response.body.len()
    );
    Ok(response)
}

fn normalize_decoded_response_headers(headers: &mut HeaderMap) {
    headers.remove("content-encoding");
    headers.remove("content-length");
    headers.remove("transfer-encoding");
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

fn suppress_navigation_only_default_headers(process: &mut Command) {
    process
        .arg("-H")
        .arg("Upgrade-Insecure-Requests:")
        .arg("-H")
        .arg("Sec-Fetch-User:");
}

fn parse_curl_output(output: &[u8]) -> Result<CurlImpersonateResponse, NetworkError> {
    let mut remaining = output;
    let mut status = None;
    let mut headers = HeaderMap::new();

    loop {
        if !remaining.starts_with(b"HTTP/") {
            break;
        }

        let Some((header_block, body_start)) = split_header_block(remaining) else {
            return Err(NetworkError::ResourceLoadError(
                "curl response did not contain a complete header block".to_string(),
            ));
        };
        let (next_status, next_headers) = parse_header_block(header_block)?;
        status = Some(next_status);
        headers = next_headers;
        remaining = &remaining[body_start..];
    }

    let status = status.ok_or_else(|| {
        NetworkError::ResourceLoadError("curl response did not start with HTTP headers".to_string())
    })?;

    Ok(CurlImpersonateResponse {
        status,
        headers,
        body: remaining.to_vec(),
    })
}

fn split_header_block(bytes: &[u8]) -> Option<(&[u8], usize)> {
    find_bytes(bytes, b"\r\n\r\n")
        .map(|index| (&bytes[..index], index + 4))
        .or_else(|| find_bytes(bytes, b"\n\n").map(|index| (&bytes[..index], index + 2)))
}

fn parse_header_block(block: &[u8]) -> Result<(StatusCode, HeaderMap), NetworkError> {
    let text = String::from_utf8_lossy(block);
    let mut lines = text.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| NetworkError::ResourceLoadError("empty curl header block".to_string()))?;
    let status = parse_status_line(status_line)?;
    let mut headers = HeaderMap::new();

    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let Ok(name) = HeaderName::from_bytes(name.trim().as_bytes()) else {
            continue;
        };
        let Ok(value) = HeaderValue::from_str(value.trim()) else {
            continue;
        };
        headers.append(name, value);
    }

    Ok((status, headers))
}

fn parse_status_line(line: &str) -> Result<StatusCode, NetworkError> {
    let status = line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| NetworkError::ResourceLoadError(format!("invalid curl status line: {line}")))?
        .parse::<u16>()
        .map_err(|error| NetworkError::ResourceLoadError(format!("invalid curl status code: {error}")))?;

    StatusCode::from_u16(status)
        .map_err(|error| NetworkError::ResourceLoadError(format!("invalid curl status code: {error}")))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn curl_command_for_user_agent(user_agent: &str) -> String {
    major_version_after_token(user_agent, "Chrome/")
        .or_else(|| major_version_after_token(user_agent, "Chromium/"))
        .or_else(|| major_version_after_token(user_agent, "CriOS/"))
        .map(|major| format!("curl_chrome{major}"))
        .unwrap_or_else(|| DEFAULT_CHROME_TARGET.to_string())
}

fn major_version_after_token(value: &str, token: &str) -> Option<u16> {
    value
        .split_once(token)
        .and_then(|(_, rest)| rest.split(['.', ' ', ';', ')']).next())
        .and_then(|major| major.parse::<u16>().ok())
        .filter(|major| *major > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curl_command_tracks_chrome_user_agent() {
        assert_eq!(
            curl_command_for_user_agent("Mozilla/5.0 Chrome/142.0.0.0 Safari/537.36"),
            "curl_chrome142"
        );
    }

    #[test]
    fn parse_output_keeps_last_header_block_and_binary_body() {
        let response = parse_curl_output(
            b"HTTP/1.1 100 Continue\r\n\r\nHTTP/2 200\r\ncontent-type: text/plain\r\nset-cookie: a=b\r\n\r\nbody",
        )
        .unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            HeaderValue::from_static("text/plain")
        );
        assert_eq!(response.body, b"body");
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
    fn removes_headers_that_no_longer_match_curl_decoded_body() {
        let mut headers = HeaderMap::new();
        headers.insert("content-encoding", HeaderValue::from_static("br"));
        headers.insert("content-length", HeaderValue::from_static("12"));
        headers.insert("content-type", HeaderValue::from_static("text/html"));

        normalize_decoded_response_headers(&mut headers);

        assert!(!headers.contains_key("content-encoding"));
        assert!(!headers.contains_key("content-length"));
        assert_eq!(
            headers.get("content-type").unwrap(),
            HeaderValue::from_static("text/html")
        );
    }
}

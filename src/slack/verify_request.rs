use actix_http::error::ErrorBadRequest;
use actix_http::http::HeaderMap;
use actix_web::Error;

use ring::hmac;

static TIMESTAMP_FIVE_MINUTES: i64 = 300;

pub struct SlackValidationHeaders {
    pub request_timestamp: i64,
    pub request_signature: String,
}

pub fn validate_request_headers(headers: &HeaderMap) -> Result<SlackValidationHeaders, Error> {
    let request_timestamp = match headers.get("X-Slack-Request-Timestamp") {
        Some(t) => {
            let local_time = chrono::Utc::now().timestamp();
            let request_time: i64 = t
                .to_str()
                .map_err(|_| {
                    warn!("Could not extract timestamp from header");
                    ErrorBadRequest("Unable to extract timestamp from header.");
                })?
                .parse()
                .map_err(|_| {
                    warn!("Could not parse timestamp={:?}", t);
                    ErrorBadRequest("Invalid timestamp.");
                })?;

            let abz = (local_time - request_time).abs();
            if abz > TIMESTAMP_FIVE_MINUTES {
                warn!(
                    "Expired timestamp! request_time={}, local_time={}, abz={}",
                    request_time, local_time, abz
                );
                return Err(ErrorBadRequest(
                    "Expired X-Slack-Request-Timestamp in request",
                ));
            }

            request_time
        }

        None => {
            return Err(ErrorBadRequest(
                "Missing X-Slack-Request-Timestamp in request",
            ));
        }
    };

    let request_signature = match headers.get("X-Slack-Signature") {
        None => {
            warn!("Missing slack signature header");
            return Err(ErrorBadRequest(
                "Missing X-Slack-Request-Timestamp in request",
            ));
        }
        Some(signature) => {
            let sig = signature
                .to_str()
                .map_err(|_| ErrorBadRequest("Header X-Slack-Signature is invalid"))?;
            String::from(sig)
        }
    };

    Ok(SlackValidationHeaders {
        request_timestamp,
        request_signature,
    })
}

pub fn validate_slack_signature(
    key_value: &str,
    slack_signature: String,
    body: String,
    timestamp: i64,
) -> Result<(), Error> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_value.as_bytes());

    let base = format!(
        "v0={}",
        hex::encode(hmac::sign(
            &key,
            format!("v0:{}:{}", timestamp, body).as_bytes()
        ))
    );

    if base != slack_signature {
        warn!(
            "Unable to verify signature for incoming request body={:#?} base={:#?} slack_signature={:#?}",
            body, base, slack_signature
        );
        return Err(ErrorBadRequest("could not verify slack request signature"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::slack::verify_request::{validate_request_headers, validate_slack_signature};
    use crate::SlackSecret;
    use actix_http::http::{HeaderMap, HeaderName, HeaderValue};
    use std::str::FromStr;

    #[test]
    fn test_invalid_slack_signature() {
        let key_value: SlackSecret = "8f742231b10e8888abcd99yyyzzz85a5".to_string();

        let header_signature = "v0=asdf".to_string();
        let body = "1234".to_string();

        let timestamp: i64 = 1;
        let result = validate_slack_signature(&key_value, header_signature, body, timestamp);

        assert!(result.is_err(), "signature should not be verified");
    }

    #[test]
    fn test_validate_slack_signature() {
        let key_value: SlackSecret = "8f742231b10e8888abcd99yyyzzz85a5".to_string();

        let header_signature =
            "v0=a2114d57b48eac39b9ad189dd8316235a7b4a8d21a10bd27519666489c69b503".to_string();
        let body = "token=xyzz0WbapA4vBCDEFasx0q6G&team_id=T1DC2JH3J&team_domain=testteamnow&channel_id=G8PSS9T3V&channel_name=foobar&user_id=U2CERLKJA&user_name=roadrunner&command=%2Fwebhook-collect&text=&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2FT1DC2JH3J%2F397700885554%2F96rGlfmibIGlgcZRskXaIFfN&trigger_id=398738663015.47445629121.803a0bc887a14d10d2c447fce8b6703c"
            .to_string();

        let timestamp: i64 = 1531420618;
        let result = validate_slack_signature(&key_value, header_signature, body, timestamp);

        assert!(result.is_ok(), "signature should be verified");
    }

    #[test]
    fn test_check_headers_should_succeed() {
        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str("X-Slack-Request-Timestamp").unwrap(),
            HeaderValue::from(chrono::Utc::now().timestamp() + 200),
        );
        headers.append(
            HeaderName::from_str("X-Slack-Signature").unwrap(),
            HeaderValue::from_str("asd").unwrap(),
        );

        let result = validate_request_headers(&headers);

        assert!(result.is_ok(), "tokens should be valid")
    }

    #[test]
    fn test_check_timestamp_headers_should_fail() {
        let mut headers = HeaderMap::new();
        headers.append(
            HeaderName::from_str("X-Slack-Request-Timestamp").unwrap(),
            HeaderValue::from(chrono::Utc::now().timestamp() + 400),
        );
        headers.append(
            HeaderName::from_str("X-Slack-Signature").unwrap(),
            HeaderValue::from_str("asd").unwrap(),
        );

        let result = validate_request_headers(&headers);

        assert!(result.is_err(), "header should contain expired token")
    }

    #[test]
    fn test_check_headers_should_fail() {
        let mut headers_signature = HeaderMap::new();
        headers_signature.append(
            HeaderName::from_str("X-Slack-Signature").unwrap(),
            HeaderValue::from_str("na").unwrap(),
        );
        let mut headers_timestamp = HeaderMap::new();
        headers_timestamp.append(
            HeaderName::from_str("_").unwrap(),
            HeaderValue::from_str("").unwrap(),
        );

        let signature_result = validate_request_headers(&headers_signature);
        let signature_timestamp = validate_request_headers(&headers_timestamp);

        assert!(
            signature_result.is_err(),
            "should fail when not all required headers a present"
        );
        assert!(
            signature_timestamp.is_err(),
            "should fail when not all required headers a present"
        )
    }
}

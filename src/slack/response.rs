use crate::slack::model::SlackResponseCommand;
use reqwest::Client;

/// Simple default responses to Slack channels
/// One method for each response type, nothing fancy.

pub async fn in_channel_response(response_url: String, text: String, http_client: &Client) {
    response(response_url, "in_channel".to_string(), text, http_client).await
}

pub async fn response(
    response_url: String,
    response_type: String,
    text: String,
    http_client: &Client,
) {
    let result = http_client
        .post(&response_url)
        .json::<SlackResponseCommand>(&SlackResponseCommand {
            response_type,
            text,
        })
        .send()
        .await;

    if result.is_err() {
        warn!("Unable to send response to slack!")
    }

    //Don't really care at this point. We logged the error already.
}

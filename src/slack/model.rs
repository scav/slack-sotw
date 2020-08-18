use serde::{Deserialize, Serialize};

#[serde(rename_all = "lowercase")] //we need to support lower case here
#[derive(Serialize, Deserialize, Debug)]
pub enum BotSubCommand {
    START,
    VOTE,
    LIST,
    SONG,
    INFO,
}

// This is the incoming /command from Slack.
// Everything is an Option<T> here. No trust.
#[derive(Serialize, Deserialize, Debug)]
pub struct SlackRequestCommand {
    pub token: Option<String>,
    pub team_id: Option<String>,
    pub team_domain: Option<String>,
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub command: Option<String>,
    #[serde(rename(deserialize = "text"))]
    pub sub_command: Option<BotSubCommand>,
    pub api_app_id: Option<String>,
    pub response_url: Option<String>,
    pub trigger_id: Option<String>,
}

// Outgoing response to users.
// This follows the slack API specification
#[derive(Serialize, Deserialize, Debug)]
pub struct SlackResponseCommand {
    pub response_type: String,
    pub text: String,
}

use core::fmt;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum BotSubCommand {
    Start(String), // Starts a competition with String as theme
    Vote(Uuid),    // Vote for a song based on id
    List,          // List all songs in current active competition
    Song(String),  // Add a song to the competition
    Info,          // Get the build info and stuff
}

// This is the incoming /command from Slack.
// Command and sub_command are wrapped in Option<T> because users.
#[derive(Deserialize, Debug)]
pub struct SlackRequestCommand {
    pub token: String,
    pub team_id: String,
    pub team_domain: String,
    pub channel_id: String,
    pub channel_name: String,
    pub user_id: String,
    pub user_name: String,
    pub command: Option<String>,
    #[serde(deserialize_with = "str_as_cmd")]
    pub text: Option<BotSubCommand>,
    pub api_app_id: String,
    pub response_url: String,
    pub trigger_id: String,
}

// Outgoing response to users.
// This follows the slack API specification
#[derive(Serialize, Debug)]
pub struct SlackResponseCommand {
    pub response_type: String,
    pub text: String,
}

fn str_as_cmd<'de, D>(deserializer: D) -> Result<Option<BotSubCommand>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(CmdVisitor)
}

struct CmdVisitor;

impl<'de> Visitor<'de> for CmdVisitor {
    type Value = Option<BotSubCommand>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("option string representation of an enum")
    }
    fn visit_str<E>(self, value: &str) -> Result<Option<BotSubCommand>, E>
    where
        E: de::Error,
    {
        match cmd_payload(value) {
            Some(cmd) => {
                return match cmd {
                    ("start", x) => {
                        if let Some(cmd_val) = x {
                            Ok(Some(BotSubCommand::Start(cmd_val.to_string())))
                        } else {
                            Err(E::custom("cmd missing argument"))
                        }
                    }
                    ("vote", x) => {
                        if let Some(cmd_val) = x {
                            if let Ok(song_id) = Uuid::from_str(cmd_val) {
                                return Ok(Some(BotSubCommand::Vote(song_id)));
                            }
                            Err(E::custom("song_id is not a valid uuid"))
                        } else {
                            Err(E::custom("cmd missing argument"))
                        }
                    }
                    ("list", _) => Ok(Some(BotSubCommand::List)),

                    ("song", x) => {
                        if let Some(cmd_val) = x {
                            Ok(Some(BotSubCommand::Song(cmd_val.to_string())))
                        } else {
                            Err(E::custom("cmd missing argument"))
                        }
                    }
                    ("info", _) => Ok(Some(BotSubCommand::Info)),
                    (&_, _) => Err(E::custom("unable to match input with cmd")),
                }
            }
            None => Err(E::custom("it's all over")),
        }
    }
}

fn cmd_payload(input: &str) -> Option<(&str, Option<&str>)> {
    let s: Vec<&str> = input.splitn(2, ' ').collect();
    match s.len() {
        1 => Some((s.get(0).unwrap(), None)),
        2 => Some((s.get(0).unwrap(), Some(s.get(1).unwrap()))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::slack::model::{cmd_payload, SlackRequestCommand};

    #[test]
    fn test_dez() {
        let input = "token=gIkuvaNzQIHg97ATvDxqgjtO\
        &team_id=T0001&team_domain=example\
        &channel_id=C2147483705&channel_name=test\
        &user_id=U2147483697&user_name=Steve\
        &command=/sotw\
        &text=song fwe f  few few0\
        &response_url=https://hooks.slack.com/commands/1234/5678\
        &trigger_id=13345224609.738474920.8088930838d88f008e0\
        &api_app_id=A123456";

        let result = serde_urlencoded::from_str::<SlackRequestCommand>(input);

        println!("data={:?}", result);
    }

    #[test]
    fn test_payload() {
        let input_start = "start moar music please";
        let input_info = "start";
        let result_start: (&str, Option<&str>) = cmd_payload(input_start).unwrap();
        let result_info: (&str, Option<&str>) = cmd_payload(input_info).unwrap();

        assert_eq!(
            ("start", "moar music please"),
            (result_start.0, result_start.1.unwrap()),
            "should contain both cmd and argument"
        );
        assert!(
            result_info.1.is_none(),
            "should not contain a cmd argument "
        );
    }
}

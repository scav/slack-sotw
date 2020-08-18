use crate::slack::model::{SlackRequestCommand, SlackResponseCommand};
use crate::DbPool;
use actix_web::{web, Error, HttpResponse};
use diesel::PgConnection;

/// Contains all the handlers for the different Slack commands
pub async fn handler(
    _command: web::Form<SlackRequestCommand>,
    _db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let command = _command.into_inner();
    info!("Got command={:?}", &command);

    let command_prefix = std::env::var("SLACK_COMMAND_PREFIX").unwrap_or("/sotw".to_string());
    if command.command != Some(command_prefix) {
        return Err(Error::from(
            HttpResponse::BadRequest().body("Unknown command received."),
        ));
    }

    match command.sub_command {
        _ => {}
    }

    if let Some(response_url) = &command.response_url {
        // response_url will create a public message in the chan everyone can see.
        let client = reqwest::Client::new();
        let _response = client
            .post(response_url)
            .json::<SlackResponseCommand>(&SlackResponseCommand {
                response_type: "in_channel".to_string(),
                text: "build_info_here".to_string(),
            })
            .send()
            .await;
    }

    Ok(HttpResponse::Ok().finish()) //Return .body() instead of .finish() to provide a hidden response to the user.
}

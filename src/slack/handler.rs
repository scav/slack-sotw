use crate::slack::model::{BotSubCommand, SlackRequestCommand};
use crate::sotw_db::database::save;
use crate::sotw_db::errors::{BotError, DataError};
use crate::sotw_db::model::CompetitionInsert;
use crate::DbPool;
use actix_rt::blocking::BlockingError;
use actix_web::{web, Error, HttpResponse, ResponseError};

/// Delegate to sub-handlers for the different bot commands
pub async fn handler(
    new_command: web::Form<SlackRequestCommand>,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let command = new_command.into_inner();

    match &command.text {
        Some(sub_command) => match sub_command {
            BotSubCommand::Start(description) => handle_start(description, &command, db_pool).await,
            BotSubCommand::Vote(_) => handle_vote().await,
            BotSubCommand::List => handle_list().await,
            BotSubCommand::Song(_) => handle_song().await,
            BotSubCommand::Info => handle_info().await,
        },
        None => handle_unimplemented().await,
    }
}

pub async fn handle_start(
    description: &String,
    command: &SlackRequestCommand,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let competition = command_to_competition(description.clone(), command);
    let competition = web::block(move || save(competition, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    Ok(HttpResponse::Ok().json(competition))
}

pub async fn handle_vote() -> Result<HttpResponse, Error> {
    handle_unimplemented().await
}

pub async fn handle_list() -> Result<HttpResponse, Error> {
    handle_unimplemented().await
}
pub async fn handle_song() -> Result<HttpResponse, Error> {
    handle_unimplemented().await
}
pub async fn handle_info() -> Result<HttpResponse, Error> {
    handle_unimplemented().await
}

pub async fn handle_unimplemented() -> Result<HttpResponse, Error> {
    Err(Error::from(
        BotError {
            data_error: DataError::NotImplementedError,
            message: "this command is not yet implemented".to_string(),
        }
        .error_response(),
    ))
}

pub fn command_to_competition(
    description: String,
    command: &SlackRequestCommand,
) -> CompetitionInsert {
    CompetitionInsert {
        description,
        user_id: command.user_id.clone(),
        user_name: command.user_name.clone(),
        started: chrono::Utc::now(),
        ended: None,
        is_active: false,
    }
}

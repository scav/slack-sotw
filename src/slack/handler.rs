use crate::slack::model::{BotSubCommand, SlackRequestCommand};
use crate::sotw_db::database::{
    close_competition, list_songs_active_competition, save_competition, save_song, save_song_vote,
};
use crate::sotw_db::errors::{BotError, DataError};
use crate::sotw_db::model::CompetitionInsert;
use crate::DbPool;
use actix_rt::blocking::BlockingError;
use actix_web::{web, Error, HttpResponse, ResponseError};
use uuid::Uuid;

/// Delegate to sub-handlers for the different bot commands
pub async fn handler(
    new_command: web::Form<SlackRequestCommand>,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let command = new_command.into_inner();

    match &command.text {
        Some(sub_command) => match sub_command {
            BotSubCommand::Start(description) => handle_start(description, &command, db_pool).await,
            BotSubCommand::Stop => handle_stop(command.user_id.clone(), db_pool).await,
            BotSubCommand::Vote(song_id) => {
                handle_vote(
                    song_id.clone(),
                    command.user_id.clone(),
                    command.user_name.clone(),
                    db_pool,
                )
                .await
            }
            BotSubCommand::List => handle_list(db_pool).await,
            BotSubCommand::Song(song_uri) => {
                handle_song(
                    song_uri.clone(),
                    command.user_id.clone(),
                    command.user_name.clone(),
                    db_pool,
                )
                .await
            }
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
    let competition = web::block(move || save_competition(competition, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    Ok(HttpResponse::Ok().json(competition))
}

pub async fn handle_stop(
    user_id: String,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let close_result = web::block(move || close_competition(user_id, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    Ok(HttpResponse::Ok().json(close_result))
}

pub async fn handle_vote(
    song_id: Uuid,
    user_id: String,
    user_name: String,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let song =
        web::block(move || save_song_vote(song_id, user_id, user_name, &db_pool.get().unwrap()))
            .await
            .map_err(|e| match e {
                BlockingError::Error(e) => e.error_response(),
                _ => HttpResponse::InternalServerError().finish(),
            })?;

    Ok(HttpResponse::Ok().json(song))
}

pub async fn handle_list(db_pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let active_songs = web::block(move || list_songs_active_competition(&db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    Ok(HttpResponse::Ok().json(active_songs))
}

pub async fn handle_song(
    song_uri: String,
    user_id: String,
    user_name: String,
    db_pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let song = web::block(move || save_song(song_uri, user_id, user_name, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    Ok(HttpResponse::Ok().json(song))
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

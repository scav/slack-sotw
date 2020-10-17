use crate::slack::model::{BotSubCommand, SlackRequestCommand};
use crate::slack::response::in_channel_response;
use crate::slack::verify_request::{validate_request_headers, validate_slack_signature};
use crate::sotw_db::database::{
    close_competition, list_songs_active_competition, save_competition, save_song, save_song_vote,
};
use crate::sotw_db::model::CompetitionInsert;
use crate::{DbPool, SlackSecret};
use actix_rt::blocking::BlockingError;
use actix_web::{web, Error, HttpRequest, HttpResponse, ResponseError};
use reqwest::Client;
use uuid::Uuid;

/// Delegate to sub-handlers for the different bot commands
/// Instead of calling web::Form<SlackRequestCommand> get web::Bytes
/// directly in order to use the body in Slack signature verification.
pub async fn handler(
    request: HttpRequest,
    raw_body: web::Bytes,
    db_pool: web::Data<DbPool>,
    slack_secret: web::Data<SlackSecret>,
    http_client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    //Verify request
    let slack_validated_headers = validate_request_headers(request.headers())?;
    let body = String::from_utf8(raw_body.to_vec()).unwrap();
    validate_slack_signature(
        slack_secret.as_ref(),
        slack_validated_headers.request_signature,
        body,
        slack_validated_headers.request_timestamp,
    )?;

    let command: SlackRequestCommand = serde_urlencoded::from_bytes(&raw_body)?;

    match &command.text {
        Some(sub_command) => match sub_command {
            BotSubCommand::Start(description) => {
                handle_start(description.clone(), &command, db_pool, http_client).await
            }
            BotSubCommand::Stop => handle_stop(&command, db_pool, http_client).await,
            BotSubCommand::Vote(song_id) => {
                handle_vote(
                    *song_id,
                    command.user_id.clone(),
                    command.response_url.clone(),
                    db_pool,
                    http_client,
                )
                .await
            }
            BotSubCommand::List => handle_list(db_pool).await,
            BotSubCommand::Song(song_uri) => {
                handle_song(
                    song_uri.clone(),
                    command.user_id.clone(),
                    command.response_url.clone(),
                    db_pool,
                    http_client,
                )
                .await
            }
            BotSubCommand::Info => handle_info().await,
        },
        None => handle_unimplemented().await,
    }
}

pub async fn handle_start(
    description: String,
    command: &SlackRequestCommand,
    db_pool: web::Data<DbPool>,
    http_client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let competition = CompetitionInsert {
        description,
        user_id: command.user_id.clone(),
        started: chrono::Utc::now(),
        ended: None,
        is_active: false,
    };

    let competition = web::block(move || save_competition(competition, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    let response_text = format!(
        "<@{}> started competition with description: *{}*",
        competition.user_id, competition.description
    );

    in_channel_response(
        command.response_url.clone(),
        response_text,
        &http_client.get_ref(),
    )
    .await;

    Ok(HttpResponse::Ok().finish()) //.json(competition))
}

pub async fn handle_stop(
    command: &SlackRequestCommand,
    db_pool: web::Data<DbPool>,
    http_client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let user_id = command.user_id.clone();
    let close_result = web::block(move || close_competition(user_id, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    let response_text = format!(
        "<@{}> *ended* competition with description: *{}*",
        command.user_id, close_result.description
    );

    in_channel_response(
        command.response_url.clone(),
        response_text,
        &http_client.get_ref(),
    )
    .await;

    Ok(HttpResponse::Ok().json(close_result))
}

pub async fn handle_vote(
    song_id: Uuid,
    user_id: String,
    response_url: String,
    db_pool: web::Data<DbPool>,
    http_client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let song_vote = web::block(move || save_song_vote(song_id, user_id, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    let response_text = format!(
        "<@{}> *voted* for song_id: {}",
        song_vote.user_id, song_vote.song_id
    );

    in_channel_response(response_url, response_text, &http_client.get_ref()).await;

    Ok(HttpResponse::Ok().json(song_vote))
}

pub async fn handle_list(db_pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    let active_songs = web::block(move || list_songs_active_competition(&db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    let list_response = active_songs
        .iter()
        .map(|song| format!("<@{}> - {}", song.user_id, song.song_uri.clone()))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(HttpResponse::Ok().body(list_response))
}

pub async fn handle_song(
    song_uri: String,
    user_id: String,
    response_url: String,
    db_pool: web::Data<DbPool>,
    http_client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let song = web::block(move || save_song(song_uri, user_id, &db_pool.get().unwrap()))
        .await
        .map_err(|e| match e {
            BlockingError::Error(e) => e.error_response(),
            _ => HttpResponse::InternalServerError().finish(),
        })?;

    let response_text = format!("<@{}> *added* song: {}", song.user_id, song.song_uri);

    in_channel_response(response_url, response_text, &http_client.get_ref()).await;

    Ok(HttpResponse::Ok().json(song))
}
pub async fn handle_info() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().body("Bot information - TODO"))
}

pub async fn handle_unimplemented() -> Result<HttpResponse, Error> {
    warn!("Received unimplemented command");
    Ok(HttpResponse::Ok().body("Unknown or unimplemented command"))
}

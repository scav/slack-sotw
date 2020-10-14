use crate::schema::sotw::competition::dsl::*;
use crate::schema::sotw::song::columns::competition_id;
use crate::sotw_db::errors::*;
use crate::sotw_db::model::{
    Competition, CompetitionInsert, Song, SongInsert, SongVote, SongVoteInsert,
};
use diesel::prelude::*;
use diesel::{insert_into, update, PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

pub fn save_competition(
    mut competition_insert: CompetitionInsert,
    connection: &PgConnection,
) -> Result<Competition, BotError> {
    let result: Option<Competition> = competition
        .filter(is_active.eq(true))
        .first::<Competition>(connection)
        .optional()?;

    match result {
        Some(other_competition) => {
            if other_competition.is_active {
                warn!("Found active competition. Will not continue.");
                return Err(BotError {
                    data_error: DataError::ActiveCompetitionExists(other_competition.id),
                    message: "Active competition already exists".to_string(),
                });
            }
        }
        None => {
            competition_insert.is_active = true;
        }
    }

    let saved_competition = insert_into(competition)
        .values(&competition_insert)
        .get_result::<Competition>(connection)?;

    info!("Created new competition with id={:?}", saved_competition);

    Ok(saved_competition)
}

pub fn close_competition(
    cmd_user_id: String,
    connection: &PgConnection,
) -> Result<Competition, BotError> {
    use crate::schema::sotw::competition::dsl::*;

    let result = find_active_competition(connection)?;

    if let Some(active_competition) = result {
        if active_competition.user_id != cmd_user_id {
            return Err(BotError {
                data_error: DataError::UserDoesNotOwnEntity(active_competition.id),
                message: "User does not own currently active competition".to_string(),
            });
        }

        let closed_id = update(competition.filter(id.eq(active_competition.id)))
            .set((is_active.eq(false), ended.eq(Some(chrono::Utc::now()))))
            .get_result(connection)?;

        return Ok(closed_id);
    }

    Err(BotError {
        data_error: DataError::DieselError("Not found".to_string()),
        message: "Unable to find an existing active competition".to_string(),
    })
}

pub fn find_active_competition(connection: &PgConnection) -> Result<Option<Competition>, BotError> {
    let result = competition
        .filter(is_active.eq(true))
        .first::<Competition>(connection)
        .optional()?;

    Ok(result)
}

pub fn list_songs_active_competition(connection: &PgConnection) -> Result<Vec<Song>, BotError> {
    use crate::schema::sotw::song::dsl::song;
    let active_competition = find_active_competition(connection)?;

    return match active_competition {
        Some(active_competition) => {
            let songs = song
                .filter(competition_id.eq(active_competition.id))
                .load::<Song>(connection)?;
            Ok(songs)
        }
        None => Err(BotError {
            data_error: DataError::NoActiveCompetition,
            message: "Unable to find active competition when trying to list songs".to_string(),
        }),
    };
}

pub fn save_song(
    new_song_uri: String,
    new_song_user_id: String,
    connection: &PgConnection,
) -> Result<Song, BotError> {
    use crate::schema::sotw::song::dsl::song;

    if let Some(active_competition) = find_active_competition(connection)? {
        let new_song_insert = SongInsert {
            user_id: new_song_user_id,
            song_uri: new_song_uri,
            competition_id: active_competition.id,
        };

        let saved_song = insert_into(song)
            .values(&new_song_insert)
            .get_result::<Song>(connection)?;

        return Ok(saved_song);
    }

    Err(BotError {
        data_error: DataError::NotImplementedError,
        message: "".to_string(),
    })
}

pub fn save_song_vote(
    new_vote_song_id: Uuid,
    new_vote_song_user_id: String,
    connection: &PgConnection,
) -> Result<SongVote, BotError> {
    use crate::schema::sotw::song_vote::dsl::song_vote;

    let new_song_vote = SongVoteInsert {
        user_id: new_vote_song_user_id,
        song_id: new_vote_song_id,
    };

    let saved_song_vote = insert_into(song_vote)
        .values(&new_song_vote)
        .get_result::<SongVote>(connection)?;

    Ok(saved_song_vote)
}

#[cfg(test)]
mod tests {
    use crate::sotw_db::database::{
        close_competition, find_active_competition, list_songs_active_competition,
        save_competition, save_song, save_song_vote,
    };
    use crate::sotw_db::errors::{BotError, DataError};
    use crate::sotw_db::model::{Competition, CompetitionInsert};
    use diesel::{Connection, PgConnection};

    fn random_user_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    fn test_db_connection() -> PgConnection {
        dotenv::dotenv().ok();

        let connection_string =
            std::env::var("DATABASE_URL").expect("Database connection string missing!");
        PgConnection::establish(&connection_string).unwrap()
    }

    fn create_competition_insert(user_id: String, is_active: bool) -> CompetitionInsert {
        CompetitionInsert {
            description: "asdf".to_string(),
            user_id,
            started: chrono::Utc::now(),
            ended: if !is_active {
                Some(chrono::Utc::now())
            } else {
                None
            },
            is_active,
        }
    }

    #[test]
    fn test_save() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, DataError, _>(|| {
            let competition = create_competition_insert(random_user_id(), true);

            let result = save_competition(competition, connection);

            assert!(
                !result.is_err(),
                "should not be an error when doing a clean save"
            );

            Ok(())
        });
    }

    #[test]
    fn test_save_exists() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            let user_id_owner = random_user_id();

            let result_owner_ok = save_competition(
                create_competition_insert(user_id_owner.clone(), true),
                connection,
            );
            let result_owner_err =
                save_competition(create_competition_insert(user_id_owner, true), connection);

            assert!(
                &result_owner_ok.is_ok(),
                "owner should be able to insert the first competition fine"
            );

            assert_eq!(
                result_owner_err.err().unwrap().data_error,
                BotError {
                    data_error: DataError::ActiveCompetitionExists(result_owner_ok.unwrap().id),
                    message: "".to_string(),
                }
                .data_error,
                "should not be able to save while active competition exists"
            );

            Ok(())
        });
    }

    #[test]
    fn test_close() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            let user_id_owner = random_user_id();
            let user_id_other = random_user_id();

            let result_owner = save_competition(
                create_competition_insert(user_id_owner.clone(), true),
                connection,
            );

            let result_close_other = close_competition(user_id_other, connection);
            let result_close_owner = close_competition(user_id_owner, connection);

            assert!(
                result_close_owner.is_ok(),
                "owner should be able to close competition"
            );

            assert!(
                result_close_owner.unwrap().ended.is_some(),
                "should contain an ended date when closed"
            );

            assert_eq!(
                result_close_other.err().unwrap().data_error,
                BotError {
                    data_error: DataError::UserDoesNotOwnEntity(result_owner.unwrap().id),
                    message: "".to_string()
                }
                .data_error,
                "should not be able to close competition not owned"
            );

            Ok(())
        });
    }

    #[test]
    fn test_find_active() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            let insert_competition = create_competition_insert(random_user_id(), true);
            let inserted_competition = save_competition(insert_competition, connection)?;
            let active_competition = find_active_competition(connection)?;

            assert_eq!(
                inserted_competition,
                active_competition.unwrap(),
                "the correct competition must match the one active"
            );

            Ok(())
        })
    }

    #[test]
    fn test_not_found() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            let result: Option<Competition> = find_active_competition(connection)?;

            assert_eq!(result, None, "should return None since there is no data");

            Ok(())
        });
    }

    #[test]
    fn test_save_song() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            let active_competition = save_competition(
                create_competition_insert(random_user_id(), false),
                connection,
            )?;

            let inserted_song = save_song("".to_string(), "cmd".to_string(), connection)?;
            assert_eq!(
                inserted_song.competition_id, active_competition.id,
                "inserted song needs to match the id of the active competition"
            );

            Ok(())
        });
    }

    #[test]
    fn test_list_songs_active_competition() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            save_competition(
                create_competition_insert(random_user_id(), true),
                connection,
            )?;

            for _ in 1..=10 {
                save_song(
                    "http://example.org/song123".to_string(),
                    "example|123".to_string(),
                    connection,
                )?;
            }

            let active_songs = list_songs_active_competition(connection)?;

            assert_eq!(active_songs.len(), 10, "all songs should be present");

            Ok(())
        });
    }

    #[test]
    fn test_vote_song() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, BotError, _>(|| {
            let active_competition = save_competition(
                create_competition_insert(random_user_id(), false),
                connection,
            )?;
            let inserted_song = save_song(
                "http://example.org/song123".to_string(),
                "example|123".to_string(),
                connection,
            )?;
            let voted_song =
                save_song_vote(inserted_song.id, "example|123".to_string(), connection)?;

            assert_eq!(
                inserted_song.competition_id, active_competition.id,
                "song must match competition"
            );
            assert_eq!(
                voted_song.song_id, inserted_song.id,
                "voted song must match song"
            );

            Ok(())
        });
    }
}

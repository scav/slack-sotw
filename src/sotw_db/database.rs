use crate::schema::sotw::competition::dsl::*;
use crate::sotw_db::errors::*;
use crate::sotw_db::model::{Competition, CompetitionInsert};
use diesel::prelude::*;
use diesel::{insert_into, update, PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

pub fn save(
    competition_insert: CompetitionInsert,
    connection: &PgConnection,
) -> Result<Competition, DataError> {
    let result: Option<Competition> = competition
        .filter(is_active.eq(true))
        .first::<Competition>(connection)
        .optional()?;

    match result {
        Some(other_competition) => {
            if other_competition.is_active {
                warn!("Found active competition. Will not continue.");
                return Err(DataError::ActiveCompetitionExists(other_competition.id));
            }
        }
        _ => {}
    }

    let saved_competition = insert_into(competition)
        .values(&competition_insert)
        .get_result::<Competition>(connection)?;

    info!("Created new competition with id={:?}", saved_competition.id);

    Ok(saved_competition)
}

pub fn close_competition(
    _user_id: Uuid,
    connection: &PgConnection,
) -> Result<Competition, DataError> {
    use crate::schema::sotw::competition::dsl::*;

    let result = find_active(connection)?;

    if result.user_id != _user_id {
        return Err(DataError::UserDoesNotOwnEntity(result.id));
    }

    let closed_id = update(competition.filter(id.eq(result.id)))
        .set((is_active.eq(false), ended.eq(Some(chrono::Utc::now()))))
        .get_result(connection)?;

    Ok(closed_id)
}

pub fn find_active(connection: &PgConnection) -> Result<Competition, DataError> {
    let result = competition
        .filter(is_active.eq(true))
        .first::<Competition>(connection)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::sotw_db::database::{close_competition, find_active, save};
    use crate::sotw_db::errors::DataError;
    use crate::sotw_db::model::CompetitionInsert;
    use diesel::{Connection, PgConnection};
    use uuid::Uuid;

    fn test_db_connection() -> PgConnection {
        dotenv::dotenv().ok();

        let connection_string =
            std::env::var("DATABASE_URL").expect("Database connection string missing!");
        PgConnection::establish(&connection_string).unwrap()
    }

    fn create_competition_insert(user_id: Uuid, is_active: bool) -> CompetitionInsert {
        CompetitionInsert {
            description: "a test theme for music".to_string(),
            user_id,
            user_name: "Ola Nordmann".to_string(),
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
            let user_id = Uuid::new_v4();
            let competition = create_competition_insert(user_id, true);

            let result = save(competition, connection);

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

        connection.test_transaction::<_, DataError, _>(|| {
            let user_id_owner = Uuid::new_v4();

            let result_owner_ok = save(create_competition_insert(user_id_owner, true), connection);
            let result_owner_err = save(create_competition_insert(user_id_owner, true), connection);

            assert!(
                &result_owner_ok.is_ok(),
                "owner should be able to insert the first competition fine"
            );

            assert_eq!(
                result_owner_err.err().unwrap(),
                DataError::ActiveCompetitionExists(result_owner_ok.unwrap().id),
                "should not be able to save while active competition exists"
            );

            Ok(())
        });
    }

    #[test]
    fn test_close() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, DataError, _>(|| {
            let user_id_owner = Uuid::new_v4();
            let user_id_other = Uuid::new_v4();

            let result_owner = save(create_competition_insert(user_id_owner, true), connection);

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
                result_close_other.err().unwrap(),
                DataError::UserDoesNotOwnEntity(result_owner.unwrap().id),
                "should not be able to close competition not owned"
            );

            Ok(())
        });
    }

    #[test]
    fn test_find_active() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, DataError, _>(|| {
            save(create_competition_insert(Uuid::new_v4(), false), connection)?;

            let insert_competition = create_competition_insert(Uuid::new_v4(), true);
            let inserted_competition = save(insert_competition, connection)?;
            let active_competition = find_active(connection)?;

            assert_eq!(
                inserted_competition, active_competition,
                "the correct competition must match the one active"
            );

            Ok(())
        })
    }

    #[test]
    fn test_not_found() {
        let connection = &test_db_connection();

        connection.test_transaction::<_, DataError, _>(|| {
            let result = find_active(connection);

            assert!(result.is_err(), "should fail when finding nothing");
            assert_eq!(
                result.err().unwrap(),
                DataError::DieselError(diesel::NotFound),
                "should return diesels not found error wrapped in DataError"
            );

            Ok(())
        });
    }
}

use crate::schema::sotw::competition as competition_table;
use crate::schema::sotw::song as song_table;
use crate::schema::sotw::song_vote as song_vote_table;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes the SOTW specific tables in the database
/// Changes ot the underlying schema must be reflected here.

// Competition
#[derive(PartialEq, Debug, Serialize, Deserialize, Queryable)]
pub struct Competition {
    pub id: Uuid,
    pub description: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub started: DateTime<Utc>,
    pub ended: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(PartialEq, Debug, Deserialize, Insertable)]
#[table_name = "competition_table"]
pub struct CompetitionInsert {
    pub description: String,
    pub user_id: Uuid,
    pub user_name: String,
    pub started: DateTime<Utc>,
    pub ended: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// A song for the competition
#[derive(PartialEq, Debug, Serialize, Deserialize, Queryable)]
pub struct Song {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub song_uri: String,
    pub competition_id: Uuid,
}

#[derive(PartialEq, Debug, Deserialize, Insertable)]
#[table_name = "song_table"]
pub struct SongInsert {
    pub user_id: Uuid,
    pub user_name: String,
    pub song_uri: String,
}

// A vote for any given song
// For consistency, a vote is not cast incrementing a sequence
#[derive(PartialEq, Debug, Serialize, Deserialize, Queryable)]
pub struct SongVote {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub song_id: Uuid,
}

#[derive(PartialEq, Debug, Deserialize, Insertable)]
#[table_name = "song_vote_table"]
pub struct SongVoteInsert {
    pub user_id: Uuid,
    pub user_name: String,
    pub song_id: Uuid,
}

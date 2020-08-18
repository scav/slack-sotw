pub mod sotw {
    table! {
        sotw.competition (id) {
            id -> Uuid,
            description -> Varchar,
            user_id -> Uuid,
            user_name -> Varchar,
            started -> Timestamptz,
            ended -> Nullable<Timestamptz>,
            is_active -> Bool,
        }
    }

    table! {
        sotw.song (id) {
            id -> Uuid,
            user_id -> Uuid,
            user_name -> Varchar,
            song_uri -> Varchar,
            competition_id -> Uuid,
        }
    }

    table! {
        sotw.song_vote (id) {
            id -> Uuid,
            user_id -> Uuid,
            user_name -> Varchar,
            song_id -> Uuid,
        }
    }

    joinable!(song -> competition (competition_id));
    joinable!(song_vote -> song (song_id));

    allow_tables_to_appear_in_same_query!(
        competition,
        song,
        song_vote,
    );
}

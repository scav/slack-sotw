create table competition
(
    id        uuid        not null
        constraint competition_pkey primary key
                                   default uuid_generate_v4(),
    description varchar not null,
    user_id   varchar        not null,
    started   timestamp with time zone not null default (now() at time zone 'utc'),
    ended     timestamp with time zone,
    is_active boolean     not null default true
);

create table song
(
    id             uuid    not null
        constraint song_pkey primary key
        default uuid_generate_v4(),
    user_id        varchar    not null,
    song_uri       varchar not null,
    competition_id uuid    not null references competition (id)
);

create table song_vote
(
    id        uuid    not null
        constraint song_vote_pkey primary key
        default uuid_generate_v4(),
    user_id   varchar    not null,
    song_id   uuid    not null references song (id)
);

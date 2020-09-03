# slack-sotw

Slack bot written in Rust for creating and voting on song competitions based on a current theme.

![slack-sotw](https://github.com/scav/slack-sotw/workflows/Rust/badge.svg)

##### Planned supported deploy options:
- [ ] docker compose
- [ ] Kubernetes
- [ ] ...?

##### Requirements
* Rust
* Diesel
* PostgreSQL

## Description
Create and manage the process up until the point where a poll should be made.

### Commands
These are the planned commands

For the judge
* `/sotw start <description>` start a new competition with the given description
* `/sotw stop` stop the current active competition

For everyone
* `/sotw list` list all songs in the currently active competition
* `/sotw song <url>` this will add a song to this weeks contest
* `/sotw vote <song_id> ` vote for a song currently in the active competition, 
  * calling song again will overwrite prior contribution
* `/sotw info` get information

## Development

Planned features:
- [ ] Create polls 
- [ ] Create polls from existing polls (needs an API?)
- [ ] Web view of current and past competitions

### Setup locally
Install diesel-cli for running migrations
```
$ cargo install diesel_cli --no-default-features --features postgres
```

Use [ngrok](https://ngrok.com/) to give outside access to bot from localhost.
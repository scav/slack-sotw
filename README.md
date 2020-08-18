# sotw

Slack bot written in Rust for creating and voting on song competitions based on a current theme.

Planned supported deploy options:
- [ ] docker compose
- [ ] Kubernetes
- [ ] ...?

Requirements
* PostgreSQL

## Commands
These are the planned commands

For the judge
* `/sotw start` start a new competition
* `/sotw vote` create a vote for all added songs, 
  * calling vote again will create a new vote with the top songs from first vote
* `/sotw list` list all songs for this week

For everyone
* `/sotw song` this will add a song to this weeks contest
  * calling song again will overwrite prior contribution
* `/sotw info` get version and deployment info
* `/sotw theme` get the active theme of the competition

## Development

Install diesel-cli for running migrations
```
$ cargo install diesel_cli --no-default-features --features postgres
```

Use [ngrok](https://ngrok.com/) to give outside access to bot from localhost.
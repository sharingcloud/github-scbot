# GitHub SC Bot

Experimental GitHub Bot to manage our development workflows.

## Features

- [x] Manage PR lifecycle with local data stored in a PostgreSQL database
- [x] Reacts to GitHub Webhooks to update review status
- [x] Generate a summary comment (once per PR)
- [x] Validate PR titles depending on per-repository regexes
- [ ] Reacts to comments:
    - [x] Override checks status
    - [x] Set QA status (or skip)
    - [x] Require reviewers
    - [ ] Require mandatory reviewers
    - [ ] Enable auto-merge

## Building

This project use the **just** command runner (https://github.com/casey/just).  
To install, use `cargo install just`.

You can then type `just --list` to print available commands.

## Developing

You can quick mount a PostgreSQL instance via Docker with the following commands:

    docker volume create postgres-data
    docker run -d \
        --name postgres \
        -p 5432:5432 \
        -v postgres-data:/var/lib/postgresql/data \
        -e POSTGRES_USER=user \
        -e POSTGRES_PASSWORD=pass \
        -e POSTGRES_DB=bot \
        postgres:alpine \
        -c max_connections=200
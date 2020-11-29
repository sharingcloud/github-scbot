# GitHub SC Bot

Experimental GitHub Bot to manage our development workflows.

## Roadmap

- [ ] Manage PR lifecycle with local data stored in a PostgreSQL database
- [ ] Reacts to GitHub Webhooks to update review status
- [ ] Reacts to comments:
    - [ ] Require reviews
    - [ ] Require mandatory reviews
    - [ ] Enable auto-merge

## Building

This project use the **just** command runner (https://github.com/casey/just).  
To install, use `cargo install just`.
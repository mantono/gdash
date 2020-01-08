# gdash
**This project is being superseded by [mantono/giss](https://github.com/mantono/giss)**, which has equivalent functionality.

List assigned work for a GitHub user, such as
- Issues
- Pull requests
- Review requests for pull requests

## Run
Syntax: `gdash USERNAME [ORGANISATIONS]...`

Example: `gdash ghost apple microsoft` list all assigned items for user _ghost_ in repositories for organisations _apple_ and _microsoft_.

Enviroment variable GITHUB_TOKEN must also be set with a valid token for GitHub's API.

## Build
Build with cargo: `cargo build`

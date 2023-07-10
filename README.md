# Actix Diesel REST API

## Desciption

This Api is focused on a video streaming back-end with tables such as users, liked-videos and watched-videos.
All coming together to create a netflix type architecture for storing basic user data.
It implements with tokio to make diesel db querys with blocking functions easy. Diesel is my ORM of
choice, DX is awsome and it's simple to get started with. Ended up going to Actix for the server framework
, they have great documentation combined with great community libraries made it easy to pick.

## Features

- Sessions
- Cookies
- Authentication
- Authorization
- Guards

## How To Run

```bash
cargo run
```

#### Go to **localhost:8080/swagger-ui/**

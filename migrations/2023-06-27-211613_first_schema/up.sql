-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR NOT NULL
);

CREATE TABLE liked_videos (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    video_id INT NOT NULL,
    user_id INT NOT NULL
);

CREATE TABLE watched_videos (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    video_id INT NOT NULL,
    user_id INT NOT NULL
);

-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL,
    password_hash CHAR(60) NOT NULL,
    role VARCHAR(5) DEFAULT 'User'
);

CREATE TABLE liked_videos (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    video_id INT NOT NULL,
    user_id INT NOT NULL,
     FOREIGN KEY (user_id) REFERENCES users(Id)
);

CREATE TABLE watched_videos (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    video_id INT NOT NULL,
    user_id INT NOT NULL,
     FOREIGN KEY (user_id) REFERENCES users(Id)
);

CREATE TABLE IF NOT EXISTS submissions(
    submission_id BIGSERIAL PRIMARY KEY,
    user_id Integer NOT NULL,
    name VARCHAR(255) NOT NULL,
    source VARCHAR(255) NOT NULL,
    FOREIGN KEY (name) REFERENCES Problems(name),
    FOREIGN KEY (user_id) REFERENCES Users(user_id)
);
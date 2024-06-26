CREATE TABLE IF NOT EXISTS users(
	user_id BIGSERIAL PRIMARY KEY,
	username VARCHAR(255) NOT NULL UNIQUE,
	password_hash VARCHAR(255) NOT NULL,
	logged_in_time TIMESTAMPTZ,
    logged_in_token CHAR(20),
	is_admin BOOLEAN NOT NULL
);

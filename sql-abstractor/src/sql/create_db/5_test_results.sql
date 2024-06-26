CREATE TABLE IF NOT EXISTS test_results(
    test_results_id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    compiled TEXT NOT NULL,
    runned TEXT NOT NULL,
    points FLOAT NOT NULL,
    refers_to BIGINT,
    FOREIGN KEY (refers_to) REFERENCES submissions(submission_id)
);
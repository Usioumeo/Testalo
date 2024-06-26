CREATE TABLE IF NOT EXISTS enabled_executors(
    incoming VARCHAR(255) ,
    outgoing VARCHAR(255) ,
	additional_data TEXT,
    PRIMARY KEY (incoming, outgoing)
);
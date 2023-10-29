CREATE TABLE subscription (
    id TEXT NOT NULL,
    auth_key TEXT NOT NULL,
    p256 VARCHAR(255) NOT NULL,
    endpoint TEXT NOT NULL,
    expiration_time INT,
    PRIMARY KEY (id),
    UNIQUE (id, endpoint)
);

CREATE TABLE notification (
	id TEXT,
	action_condition TEXT,
	subscription TEXT,
	CONSTRAINT notification_FK FOREIGN KEY (subscription) REFERENCES subscription(id)
);


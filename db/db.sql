CREATE TABLE subscription (
	id TEXT NOT NULL,
	auth_key TEXT NOT NULL,
	p256 VARCHAR NOT NULL,
	endpoint TEXT NOT NULL,
	expiration_time INT, 
    subscribed INTEGER, 
    action_condition TEXT NOT NULL,
    UNIQUE (endpoint, action_condition)
);


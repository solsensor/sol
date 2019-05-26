CREATE TABLE onetime_logins (
    token TEXT PRIMARY KEY NOT NULL,
	user_id INTEGER NOT NULL,
	created TIMESTAMP NOT NULL DEFAULT (datetime('now')),
	expires TIMESTAMP NOT NULL DEFAULT (datetime('now', '+10 minutes')),
	FOREIGN KEY(user_id) REFERENCES users(id)
);

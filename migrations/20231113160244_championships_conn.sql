-- Add migration script here

-- intelli.user_championships definition

CREATE TABLE
    user_championships (
        user_id INTEGER NOT NULL,
        championship_id INTEGER NOT NULL,
        PRIMARY KEY (user_id, championship_id),
        FOREIGN KEY (user_id) REFERENCES "users" (id),
        FOREIGN KEY (championship_id) REFERENCES championship (id)
    );
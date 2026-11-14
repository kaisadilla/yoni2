-- Contains key-value string pairs.
CREATE TABLE state (
  key TEXT PRIMARY KEY,
  value TEXT
) STRICT;

-- Contains gifs that Yoni may send with the /gif command.
CREATE TABLE yoni_gifs
(
  url TEXT PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE CHECK(length(hash) = 32)
) STRICT;

-- Contains gifs that Yoni won't add to yoni_gifs.
CREATE TABLE disabled_gifs (
  url TEXT PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE CHECK(length(hash) = 32)
) STRICT;

-- Contains gifs that aren't allowed in the server, and that Yoni will delete
-- automatically on sight.
CREATE TABLE forbidden_gifs (
  url TEXT PRIMARY KEY,
  hash BLOB NOT NULL UNIQUE CHECK(length(hash) = 32)
) STRICT;

-- Stats related to users in the server.
CREATE TABLE user_stats (
  guild_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,

  -- The amount of times a message from this user was reacted with a :tomato:
  tomatoes_received INTEGER NOT NULL DEFAULT 0,
  -- The last time this user said something related to shit.
  last_mierda INTEGER,
  record_mierda INTEGER NOT NULL DEFAULT 0,

  PRIMARY KEY (guild_id, user_id)
) STRICT;

-- Each row represents a mod in a specific thread.
CREATE TABLE thread_mod (
  guild_id INTEGER NOT NULL,
  thread_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,

  PRIMARY KEY (guild_id, thread_id, user_id)
) STRICT;

-- Each row represents a user that is blocked in a specific thread.
CREATE TABLE users_blocked_in_threads (
  guild_id INTEGER NOT NULL,
  thread_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,

  PRIMARY KEY (guild_id, thread_id, user_id)
) STRICT;
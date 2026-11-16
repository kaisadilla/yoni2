-- Contains key-value string pairs.
CREATE TABLE state
(
  guild_id INTEGER NOT NULL,
  key      TEXT PRIMARY KEY,
  value    TEXT
) STRICT;

CREATE TABLE gifs
(
  guild_id        INTEGER NOT NULL,
  url             TEXT    NOT NULL,
  exact_hash      BLOB,
  visual_hash     BLOB,
  similarity_hash BLOB,
  -- allowed: used by Yoni.
  -- disabled: can be used in the server, but Yoni won't send it.
  -- banned: cannot be used in the server and Yoni won't send it.
  status          TEXT    NOT NULL CHECK (status IN ('allowed', 'disabled', 'banned')),

  PRIMARY KEY (guild_id, url)
) STRICT;

CREATE INDEX gifs_allowed_by_guild
  ON gifs (guild_id, url)
  WHERE status = 'allowed';

-- Stats related to users in the server.
CREATE TABLE user_stats
(
  guild_id          INTEGER NOT NULL,
  user_id           INTEGER NOT NULL,

  -- The amount of times a message from this user was reacted with a :tomato:
  tomatoes_received INTEGER NOT NULL DEFAULT 0,
  -- The last time this user said something related to shit.
  last_mierda       INTEGER,
  record_mierda     INTEGER NOT NULL DEFAULT 0,

  PRIMARY KEY (guild_id, user_id)
) STRICT;

-- Each row represents a mod in a specific thread.
CREATE TABLE thread_mod
(
  guild_id  INTEGER NOT NULL,
  thread_id INTEGER NOT NULL,
  user_id   INTEGER NOT NULL,

  PRIMARY KEY (guild_id, thread_id, user_id)
) STRICT;

-- Each row represents a user that is blocked in a specific thread.
CREATE TABLE users_blocked_in_threads
(
  guild_id  INTEGER NOT NULL,
  thread_id INTEGER NOT NULL,
  user_id   INTEGER NOT NULL,

  PRIMARY KEY (guild_id, thread_id, user_id)
) STRICT;
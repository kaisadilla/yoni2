use crate::gif::{GifPHash, GifStatus};
use crate::util::Hash;
use anyhow::Result;
use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{SqlitePool, query_scalar};
use std::str::FromStr;

const SECONDS_PER_DAY: i64 = 86_400;

pub struct Store {
  db: SqlitePool,
}

impl Store {
  pub async fn load(db_url: &str) -> Result<Store> {
    let options =
      SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);

    let db = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&db).await?;

    Ok(Store { db })
  }

  pub async fn add_yoni_gif(
    &self,
    guild: GuildId,
    gif_url: impl Into<String>,
    status: GifStatus,
  ) -> Result<()> {
    let guild = i64::try_from(guild)?;
    let gif_url = gif_url.into();

    sqlx::query(
      r#"
        INSERT INTO gifs
          (guild_id, url, exact_hash, visual_hash, similarity_hash, status)
        VALUES (?, ?, NULL, NULL, NULL, ?)
        ON CONFLICT DO NOTHING
      "#,
    )
    .bind(&guild)
    .bind(&gif_url)
    .bind(status)
    .execute(&self.db)
    .await?;

    Ok(())
  }

  pub async fn update_gif_status(
    &self,
    guild: GuildId,
    gif_url: impl Into<String>,
    status: GifStatus,
  ) -> Result<()> {
    let guild = i64::try_from(guild.get())?;
    let gif_url = gif_url.into();

    sqlx::query("UPDATE gifs SET status = ? WHERE guild_id = ? AND url = ?")
      .bind(status)
      .bind(guild)
      .bind(gif_url)
      .execute(&self.db)
      .await?;

    Ok(())
  }

  /// IF the GIF url given is in the store, returns its status, otherwise it
  /// returns [`None`].
  pub async fn get_gif_status_by_url(
    &self,
    guild: GuildId,
    gif_url: impl Into<String>,
  ) -> Result<Option<GifStatus>> {
    let guild = i64::try_from(guild)?;
    let gif_url = gif_url.into();

    let status = sqlx::query_scalar::<_, GifStatus>(
      "SELECT status FROM gifs WHERE guild_id = ? AND url = ?",
    )
    .bind(guild)
    .bind(gif_url)
    .fetch_optional(&self.db)
    .await?;

    Ok(status)
  }

  /// IF the GIF exact_hash given is in the store, returns its status, otherwise
  /// it returns [`None`].
  pub async fn get_gif_status_by_exact_hash(
    &self,
    guild: GuildId,
    exact_hash: Hash,
  ) -> Result<Option<GifStatus>> {
    let guild = i64::try_from(guild)?;

    let status = sqlx::query_scalar::<_, GifStatus>(
      "SELECT status FROM gifs WHERE guild_id = ? AND exact_hash = ?",
    )
    .bind(guild)
    .bind(exact_hash.as_slice())
    .fetch_optional(&self.db)
    .await?;

    Ok(status)
  }

  /// IF the GIF visual_hash given is in the store, returns its status,
  /// otherwise it returns [`None`].
  pub async fn get_gif_status_by_visual_hash(
    &self,
    guild: GuildId,
    visual_hash: Hash,
  ) -> Result<Option<GifStatus>> {
    let guild = i64::try_from(guild)?;

    let status = sqlx::query_scalar::<_, GifStatus>(
      "SELECT status FROM gifs WHERE guild_id = ? AND visual_hash = ?",
    )
    .bind(guild)
    .bind(visual_hash.as_slice())
    .fetch_optional(&self.db)
    .await?;

    Ok(status)
  }

  pub async fn set_gif_status(
    &self,
    guild: GuildId,
    gif_url: impl Into<String>,
    status: GifStatus,
  ) -> Result<()> {
    let guild = i64::try_from(guild.get())?;

    sqlx::query(
      r#"
        UPDATE gifs
        SET status = ?
        WHERE guild_id = ? AND url = ?
      "#,
    )
    .bind(status)
    .bind(guild)
    .bind(gif_url.into())
    .execute(&self.db)
    .await?;

    Ok(())
  }

  /// Returns the url of a random GIF from among those Yoni is allowed to use.
  pub async fn get_random_yoni_gif(
    &self,
    guild: GuildId,
  ) -> Result<Option<String>> {
    let guild = i64::try_from(guild)?;

    let url = sqlx::query_scalar(
      r#"
        SELECT url
        FROM gifs
        WHERE guild_id = ?1 AND status = 'allowed'
        LIMIT 1
        OFFSET (
          SELECT abs(random() % MAX(COUNT(*), 1))
          FROM gifs
          WHERE guild_id = ?1 AND status = 'allowed'
        )
      "#,
    )
    .bind(guild)
    .fetch_optional(&self.db)
    .await?;

    Ok(url)
  }

  pub async fn thread_block(
    &self,
    guild: GuildId,
    thread: ChannelId,
    usr: UserId,
  ) -> Result<()> {
    let guild = i64::try_from(guild)?;
    let thread = i64::try_from(thread)?;
    let usr = i64::try_from(usr)?;

    sqlx::query(
      r#"
        INSERT OR IGNORE INTO users_blocked_in_threads
          (guild_id, thread_id, user_id)
        VALUES (?, ?, ?)
      "#,
    )
    .bind(guild)
    .bind(thread)
    .bind(usr)
    .execute(&self.db)
    .await?;

    Ok(())
  }

  pub async fn thread_unblock(
    &self,
    guild: GuildId,
    thread: ChannelId,
    usr: UserId,
  ) -> Result<()> {
    let guild = i64::try_from(guild)?;
    let thread = i64::try_from(thread)?;
    let usr = i64::try_from(usr)?;

    sqlx::query(
      r#"
        DELETE FROM users_blocked_in_threads
        WHERE guild_id = ? AND thread_id = ? AND user_id = ?
      "#,
    )
    .bind(guild)
    .bind(thread)
    .bind(usr)
    .execute(&self.db)
    .await?;

    Ok(())
  }

  pub async fn is_blocked_in_thread(
    &self,
    guild: GuildId,
    thread: ChannelId,
    usr: UserId,
  ) -> Result<bool> {
    let guild = i64::try_from(guild)?;
    let thread = i64::try_from(thread)?;
    let usr = i64::try_from(usr)?;

    let exists: i64 = sqlx::query_scalar(
      r#"
        SELECT EXISTS (
          SELECT 1
          FROM users_blocked_in_threads
          WHERE guild_id = ? AND thread_id = ? AND user_id = ?
        )
      "#,
    )
    .bind(guild)
    .bind(thread)
    .bind(usr)
    .fetch_one(&self.db)
    .await?;

    Ok(exists != 0)
  }

  pub async fn add_thread_mod(
    &self,
    guild: GuildId,
    thread: ChannelId,
    usr: UserId,
  ) -> Result<()> {
    let guild = i64::try_from(guild)?;
    let thread = i64::try_from(thread)?;
    let usr = i64::try_from(usr)?;

    sqlx::query(
      r#"
        INSERT OR IGNORE INTO thread_mod
          (guild_id, thread_id, user_id)
        VALUES (?, ?, ?)
      "#,
    )
    .bind(guild)
    .bind(thread)
    .bind(usr)
    .execute(&self.db)
    .await?;

    Ok(())
  }

  pub async fn remove_thread_mod(
    &self,
    guild: GuildId,
    thread: ChannelId,
    usr: UserId,
  ) -> Result<()> {
    let guild = i64::try_from(guild)?;
    let thread = i64::try_from(thread)?;
    let usr = i64::try_from(usr)?;

    sqlx::query(
      r#"
        DELETE FROM thread_mod
        WHERE guild_id = ? AND thread_id = ? AND user_id = ?
      "#,
    )
    .bind(guild)
    .bind(thread)
    .bind(usr)
    .execute(&self.db)
    .await?;

    Ok(())
  }

  pub async fn is_thread_mod(
    &self,
    guild: GuildId,
    thread: ChannelId,
    usr: UserId,
  ) -> Result<bool> {
    let guild = i64::try_from(guild)?;
    let thread = i64::try_from(thread)?;
    let usr = i64::try_from(usr)?;

    let exists: i64 = sqlx::query_scalar(
      r#"
        SELECT EXISTS (
          SELECT 1
          FROM thread_mod
          WHERE guild_id = ? AND thread_id = ? AND user_id = ?
        )
      "#,
    )
    .bind(guild)
    .bind(thread)
    .bind(usr)
    .fetch_one(&self.db)
    .await?;

    Ok(exists != 0)
  }

  /// Sets the time of the user's last mierda to now, and returns a tuple
  /// containing the amount of days since the last mierda, and whether this is
  /// a new record.
  pub async fn update_mierda(
    &self,
    guild: GuildId,
    usr: UserId,
  ) -> Result<Option<(i64, bool)>> {
    let guild = i64::try_from(guild.get())?;
    let usr = i64::try_from(usr.get())?;

    let mut tx = self.db.begin_with("BEGIN IMMEDIATE").await?;

    let now: i64 =
      sqlx::query_scalar("SELECT unixepoch()").fetch_one(&mut *tx).await?;

    let prev: Option<(i64, i64)> = sqlx::query_as(
      r#"
        SELECT last_mierda, record_mierda
        FROM user_stats
        WHERE guild_id = ? AND user_id = ?
      "#,
    )
    .bind(guild)
    .bind(usr)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((last, record)) = prev else {
      sqlx::query(
        r#"
        INSERT INTO
          user_stats (guild_id, user_id, last_mierda)
        VALUES
          (?, ?, ?)
        "#,
      )
      .bind(guild)
      .bind(usr)
      .bind(now)
      .execute(&mut *tx)
      .await?;

      tx.commit().await?;

      return Ok(None);
    };

    let days = now.saturating_sub(last) / SECONDS_PER_DAY;
    let is_new_record = days > record;

    sqlx::query(
      r#"
        UPDATE
          user_stats
        SET
          last_mierda = ?,
          record_mierda = MAX(record_mierda, ?)
        WHERE
          guild_id = ? AND user_id = ?
      "#,
    )
    .bind(now)
    .bind(days)
    .bind(guild)
    .bind(usr)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Some((days, is_new_record)))
  }

  /// Returns a tuple where the first item represents the user's record of
  /// mierdaless days and the second item represents their current streak (also
  /// in days)
  pub async fn get_mierda_stats(
    &self,
    guild: GuildId,
    usr: UserId,
  ) -> Result<Option<(i64, i64)>> {
    let guild = i64::try_from(guild.get())?;
    let usr = i64::try_from(usr.get())?;

    let res: Option<(i64, i64)> = sqlx::query_as(
      r#"
        SELECT last_mierda, record_mierda
        FROM user_stats
        WHERE guild_id = ? AND user_id = ?
      "#,
    )
    .bind(guild)
    .bind(usr)
    .fetch_optional(&self.db)
    .await?;

    let Some((last, record)) = res else {
      return Ok(None);
    };

    let now: i64 =
      sqlx::query_scalar("SELECT unixepoch()").fetch_one(&self.db).await?;

    let streak = now.saturating_sub(last) / SECONDS_PER_DAY;

    Ok(Some((record, streak)))
  }

  pub async fn add_tomato(&self, guild: GuildId, usr: UserId) -> Result<i64> {
    let guild = i64::try_from(guild.get())?;
    let usr = i64::try_from(usr.get())?;

    let count: i64 = sqlx::query_scalar(
      r#"
        INSERT INTO user_stats (guild_id, user_id, tomatoes_received)
        VALUES (?, ?, 1)
        ON CONFLICT (guild_id, user_id)
          DO UPDATE SET tomatoes_received = tomatoes_received + 1
        RETURNING tomatoes_received
      "#,
    )
    .bind(guild)
    .bind(usr)
    .fetch_one(&self.db)
    .await?;

    Ok(count)
  }

  pub async fn remove_tomato(
    &self,
    guild: GuildId,
    usr: UserId,
  ) -> Result<i64> {
    let guild = i64::try_from(guild.get())?;
    let usr = i64::try_from(usr.get())?;

    let count: Option<i64> = sqlx::query_scalar(
      r#"
        UPDATE user_stats
        SET tomatoes_received = MAX(0, tomatoes_received - 1)
        WHERE guild_id = ? AND user_id = ?
        RETURNING tomatoes_received
      "#,
    )
    .bind(guild)
    .bind(usr)
    .fetch_one(&self.db)
    .await?;

    Ok(count.unwrap_or(0))
  }

  pub async fn top_tomatoes(
    &self,
    guild: GuildId,
    count: i64,
  ) -> Result<Vec<(UserId, i64)>> {
    let guild = i64::try_from(guild.get())?;

    let rows: Vec<(i64, i64)> = sqlx::query_as(
      r#"
        SELECT user_id, tomatoes_received
        FROM user_stats
        WHERE guild_id = ?
          AND tomatoes_received > 0
        ORDER BY tomatoes_received DESC, user_id ASC
        LIMIT ?
      "#,
    )
    .bind(guild)
    .bind(count)
    .fetch_all(&self.db)
    .await?;

    rows
      .into_iter()
      .map(|(id, count)| Ok((UserId::new(u64::try_from(id)?), count)))
      .collect()
  }
}

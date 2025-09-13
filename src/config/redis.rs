use redis::{Client, RedisError};

pub async fn create_client(redis_url: &str) -> Result<Client, RedisError> {
    Client::open(redis_url)
}

pub async fn test_connection(client: &Client) -> Result<(), RedisError> {
    let mut conn = client.get_multiplexed_async_connection().await?;
    redis::cmd("PING").query_async::<_, ()>(&mut conn).await?;
    Ok(())
}

pub struct RedisSession {
    client: Client,
}

impl RedisSession {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn set_session(
        &self,
        session_id: &str,
        data: &str,
        ttl_seconds: usize,
    ) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("SETEX")
            .arg(format!("session:{}", session_id))
            .arg(ttl_seconds)
            .arg(data)
            .query_async(&mut conn)
            .await
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<String>, RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("GET")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        redis::cmd("DEL")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await
    }
}

use r2d2_sqlite::rusqlite::params;
use uuid::Uuid;

use crate::{error::CustomError, Subscription, SubscriptionBody, SubscriptionOptions};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

/// Checks if the subscription already exists in the database. with the action_condition and endpoint.
pub fn insert_subscription(pool: &Pool, subs: SubscriptionBody) -> Result<usize, CustomError> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare("INSERT INTO subscription (id, endpoint, auth_key, p256, expiration_time, subscribed, action_condition)
                                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                                    ON CONFLICT (endpoint, action_condition)
                                    DO UPDATE SET
                                      subscribed = ?6;")?;
    for action in subs.action_condition.iter() {
        let subscription_id = Uuid::new_v4().to_string();

        let mut subscribed = 0;
        if action.1 {
            subscribed = 1;
        }
         
        let result = stmt.execute((
            &subscription_id,
            &subs.subscription_push.endpoint,
            &subs.subscription_push.keys.auth,
            &subs.subscription_push.keys.p256dh,
            &subs.subscription_push.expirationTime,
            subscribed,
            &action.0,
        ))?;
    }

    return Ok(0);
}

pub fn get_subscriptions_by_endpoint(
    endpoint: &String,
    pool: &Pool,
) -> Result<Vec<String>, CustomError> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare(
        "SELECT action_condition  FROM subscription WHERE  endpoint = ?1 AND subscribed = 1",
    )?;

    let mut rows = stmt.query_map([endpoint], |row| Ok(row.get(0)?))?;

    let mut subscriptions: Vec<String> = Vec::new();

    for row in rows {
        subscriptions.push(row?);
    }

    return Ok(subscriptions);
}

pub fn get_subscription_by_action_condition(
    pool: &Pool,
    action_condition: &String,
) -> Result<Vec<Subscription>, CustomError> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare("SELECT endpoint, auth_key, p256, expiration_time  FROM subscription WHERE  action_condition = ?1 AND subscribed = 1")?;

    let mut rows = stmt.query_map([action_condition], |row| {
        Ok(Subscription {
            endpoint: row.get(0)?,
            expirationTime: row.get(3)?,
            keys: SubscriptionOptions {
                auth: row.get(1)?,
                p256dh: row.get(2)?,
            },
        })
    })?;

    let mut subscriptions: Vec<Subscription> = Vec::new();

    for row in rows {
        subscriptions.push(row?);
    }

    return Ok(subscriptions);
}

/// Set the subscription status to unsubscribed.
pub fn switch_subscription_status(
    pool: &Pool,
    endpoint_url: &String,
) -> Result<usize, CustomError> {
    let conn = pool.get()?;

    let mut stmt = conn.prepare("UPDATE subscription SET subscribed = 0 WHERE endpoint = ?1")?;

    let result = stmt.execute(params![endpoint_url])?;

    return Ok(result);
}

use r2d2_sqlite::rusqlite::params;
use uuid::Uuid;

use crate::{Subscription, SubscriptionBody, SubscriptionOptions};

pub type Pool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type Connection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

pub fn insert_subscription(pool: &Pool, subs: SubscriptionBody) -> usize {
    let conn = pool.get().unwrap();

    let mut stmt = conn.prepare("
                                INSERT INTO subscription (id, endpoint, auth_key, p256, expiration_time, subscribed) VALUES (?1, ?2, ?3, ?4, ?5, ?6 )

                                ").unwrap();

    let subscription_id = Uuid::new_v4().to_string();
    let notification_id = Uuid::new_v4().to_string();

    let result = stmt
        .execute((
            &subscription_id,
            &subs.subscription_push.endpoint,
            &subs.subscription_push.keys.auth,
            &subs.subscription_push.keys.p256dh,
            &subs.subscription_push.expirationTime,
            1,
        ))
        .unwrap();
    insert_notification_array(
        pool,
        subs.action_condition,
        &notification_id,
        &subscription_id,
    );

    result
}

fn insert_notification_array(
    pool: &Pool,
    actions: Vec<String>,
    id: &String,
    subscription_id: &String,
) {
    for action in actions.iter() {
        let conn = pool.get().unwrap();

        let mut stmt = conn
            .prepare(
                "INSERT INTO notification (id, action_condition, subscription) VALUES (?1, ?2, ?3)",
            )
            .unwrap();

        let result = stmt
            .execute((&id.to_string(), &action, &subscription_id))
            .unwrap();
    }
}

pub fn get_subscription_by_action_condition(
    pool: &Pool,
    action_condition: &String,
) -> Vec<Subscription> {
    let conn = pool.get().unwrap();

    let mut stmt = conn.prepare("SELECT endpoint, auth_key, p256, expiration_time  FROM subscription WHERE id IN (SELECT subscription FROM notification WHERE action_condition = ?1) AND subscribed = 1").unwrap();

    let mut rows = stmt
        .query_map([action_condition], |row| {
            Ok(Subscription {
                endpoint: row.get(0).unwrap(),
                expirationTime: row.get(3).unwrap(),
                keys: SubscriptionOptions {
                    auth: row.get(1).unwrap(),
                    p256dh: row.get(2).unwrap(),
                },
            })
        })
        .unwrap();

    let mut subscriptions: Vec<Subscription> = Vec::new();

    for row in rows {
        subscriptions.push(row.unwrap());
    }

    subscriptions
}

/// Set the subscription status to unsubscribed.
pub fn switch_subscription_status(pool: &Pool, endpoint_url: &String) -> usize {
    let conn = pool.get().unwrap();

    let mut stmt = conn
        .prepare("UPDATE subscription SET subscribed = 0 WHERE endpoint = ?1")
        .unwrap();

    let result = stmt.execute(params![endpoint_url]).unwrap();

    result
}

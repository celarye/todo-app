use std::time::SystemTime;

use actix_web::cookie::Cookie;
use rand::{Rng, distr::Alphanumeric};

use crate::Database;
use crate::app::handlers::{TodoItem, User};
use crate::logic::auth::github;

pub struct Logic {
    database: Database,
}

impl Logic {
    pub fn new(database: Database) -> Self {
        Logic { database }
    }

    pub async fn user_count(&self) -> Result<u32, ()> {
        self.database.user_count().await
    }

    pub async fn validate(&self, session: Cookie<'_>) -> Result<u32, ()> {
        self.database.get_session(session.value().to_string()).await
    }

    pub async fn get_user(&self, user_id: u32) -> Result<User, ()> {
        self.database.get_user(user_id).await
    }

    pub async fn github_init(&self) -> Result<String, ()> {
        let (redirect_url, csrf_token) = github::init();
        self.database.add_csrf_token(csrf_token).await?;

        Ok(redirect_url.to_string())
    }

    pub async fn github_success(&self, code: &String, csrf_token: &String) -> Result<String, ()> {
        self.database.get_csrf_token(csrf_token).await?;

        self.database.delete_csrf_token(csrf_token).await?;

        let user_data = github::success(code.clone()).await?;

        let user_id = match self.database.get_user_by_github_id(user_data.0).await {
            Ok(user) => user.id,
            Err(_) => {
                self.database
                    .add_user(user_data.0, user_data.1, user_data.2, user_data.3)
                    .await?
            }
        };

        let session_value: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();

        let session_expires = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60 * 60 * 6;

        self.database
            .add_session(user_id, session_value.clone(), session_expires.clone())
            .await?;

        Ok(session_value)
    }

    pub async fn logout(&self, session: Cookie<'_>) -> Result<(), ()> {
        self.database
            .delete_session(session.value().to_string())
            .await?;

        Ok(())
    }

    pub async fn get_items(&self, user_id: u32) -> Result<Vec<TodoItem>, ()> {
        self.database.get_todo_items(user_id).await.map_err(|_| ())
    }

    pub async fn add_item(&self, user_id: u32, content: String) -> Result<(), ()> {
        self.database
            .add_todo_item(user_id, content)
            .await
            .map_err(|_| ())
    }

    pub async fn update_item(&self, user_id: u32, item_id: u32, done: bool) -> Result<(), ()> {
        self.database
            .update_todo_item(user_id, item_id, done)
            .await
            .map_err(|_| ())
    }

    pub async fn delete_item(&self, user_id: u32, item_id: u32) -> Result<(), ()> {
        self.database
            .delete_todo_item(user_id, item_id)
            .await
            .map_err(|_| ())
    }
}

use std::time::SystemTime;

use oauth2::CsrfToken;
use sqlx::{
    Row, Sqlite,
    migrate::MigrateDatabase,
    sqlite::{SqlitePool, SqlitePoolOptions},
};
use tracing::{error, info};

use crate::app::handlers::{TodoItem, User};

pub struct Database {
    connection_pool: SqlitePool,
}

impl Database {
    pub async fn connect(database_path: &str, max_connections: u32) -> Result<Self, ()> {
        if !Sqlite::database_exists(database_path)
            .await
            .unwrap_or(false)
        {
            info!("No database file found, creating a new one");
            Database::create(database_path).await?
        }

        match SqlitePoolOptions::new()
            .max_connections(max_connections)
            .connect_lazy(database_path)
        {
            Ok(connection_pool) => Ok(Database { connection_pool }),
            Err(err) => {
                error!(
                    "Something went wrong while creating a connection pool to the database: {}",
                    &err
                );
                return Err(());
            }
        }
    }

    async fn create(database_path: &str) -> Result<(), ()> {
        if let Err(err) = Sqlite::create_database(database_path).await {
            error!("Something wrong while creating the database: {}", err);
            return Err(());
        }

        let connection_pool = match SqlitePool::connect(database_path).await {
            Ok(connection_pool) => connection_pool,
            Err(err) => {
                error!(
                    "Something went wrong while creating a connection pool to the database (during creation): {}",
                    &err
                );
                return Err(());
            }
        };

        sqlx::query(
                "CREATE TABLE users (id INTEGER PRIMARY KEY, github_id INTEGER NOT NULL UNIQUE, username TEXT NOT NULL UNIQUE , email TEXT NOT NULL UNIQUE, profile_picture_url TEXT);",
            )
            .execute(&connection_pool)
            .await
            .expect("self written SQL query, should not fail");

        sqlx::query(
                "CREATE TABLE user_sessions (user_id INTEGER NOT NULL, session TEXT NOT NULL, expires TEXT NOT NULL, PRIMARY KEY (user_id, session)) WITHOUT ROWID;",
            )
            .execute(&connection_pool)
            .await
            .expect("self written SQL query, should not fail");

        sqlx::query("CREATE TABLE csrf_tokens (value TEXT PRIMARY KEY NOT NULL, expires TEXT NOT NULL) WITHOUT ROWID;")
            .execute(&connection_pool)
            .await
            .expect("self written SQL query, should not fail");

        sqlx::query(
                "CREATE TABLE todo_items (id INTEGER PRIMARY KEY, content TEXT NOT NULL, done INTEGER NOT NULL, user_id INTEGER NOT NULL);",
            )
            .execute(&connection_pool)
            .await
            .expect("self written SQL query, should not fail");

        Ok(())
    }

    pub async fn user_count(&self) -> Result<u32, ()> {
        match sqlx::query("SELECT COUNT() FROM users;")
            .fetch_one(&self.connection_pool)
            .await
        {
            Ok(row) => {
                return Ok(row.get::<u32, _>(0));
            }
            Err(err) => {
                error!(
                    "Something went wrong while retrieving the user count from the database: {}",
                    &err
                );
                return Err(());
            }
        };
    }

    pub async fn add_csrf_token(&self, csrf_token: CsrfToken) -> Result<(), ()> {
        if let Err(err) = sqlx::query("INSERT INTO csrf_tokens (value, expires) VALUES (?1, ?2);")
            .bind(csrf_token.secret())
            .bind(
                (SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 600)
                    .to_string(),
            )
            .execute(&self.connection_pool)
            .await
        {
            error!(
                "Something went wrong while inserting the csrf token into the database: {}",
                &err
            );
            return Err(());
        }

        Ok(())
    }

    pub async fn get_csrf_token(&self, csrf_token: &String) -> Result<(), ()> {
        match sqlx::query("SELECT expires FROM csrf_tokens WHERE value = ?1;")
            .bind(csrf_token)
            .fetch_one(&self.connection_pool)
            .await
        {
            Ok(row) => {
                let expires_str: String = row.get(0);
                let expires_timestamp: u64 = expires_str.parse().unwrap_or(0);

                if expires_timestamp
                    < SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                {
                    error!("The csrf token has expired");

                    self.delete_csrf_token(csrf_token).await.unwrap();

                    return Err(());
                }

                self.delete_csrf_token(csrf_token).await.unwrap();
                return Ok(());
            }
            Err(err) => {
                error!(
                    "Something went wrong while retrieving the csrf token from the database: {}",
                    &err
                );
                return Err(());
            }
        };
    }

    pub async fn delete_csrf_token(&self, csrf_token: &String) -> Result<(), ()> {
        if let Err(err) = sqlx::query("DELETE FROM csrf_tokens WHERE value = ?1;")
            .bind(csrf_token)
            .execute(&self.connection_pool)
            .await
        {
            error!(
                "Something went wrong while deleting the csrf token from the database: {}",
                &err
            );
            return Err(());
        };

        Ok(())
    }

    pub async fn add_session(&self, user_id: u32, session: String, expires: u64) -> Result<(), ()> {
        if let Err(err) = sqlx::query(
            "INSERT INTO user_sessions (user_id, session, expires) VALUES (?1, ?2, ?3);",
        )
        .bind(user_id)
        .bind(session)
        .bind(expires.to_string())
        .execute(&self.connection_pool)
        .await
        {
            error!(
                "Something went wrong while inserting the session into the database: {}",
                &err
            );
            return Err(());
        }

        Ok(())
    }

    pub async fn get_session(&self, session: String) -> Result<u32, ()> {
        match sqlx::query("SELECT user_id, expires FROM user_sessions WHERE session = ?1;")
            .bind(&session)
            .fetch_one(&self.connection_pool)
            .await
        {
            Ok(row) => {
                let expires_str: String = row.get(1);
                let expires_timestamp: u64 = expires_str.parse().unwrap_or(0);

                if expires_timestamp
                    < SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                {
                    error!("The session has expired");

                    self.delete_session(session).await.unwrap();

                    Err(())
                } else {
                    let user_id: u32 = row.get(0);
                    Ok(user_id)
                }
            }
            Err(err) => {
                error!(
                    "Something went wrong while retrieving the session from the database: {}",
                    &err
                );
                Err(())
            }
        }
    }

    pub async fn delete_session(&self, session: String) -> Result<(), ()> {
        if let Err(err) = sqlx::query("DELETE FROM user_sessions WHERE value = ?1;")
            .bind(session)
            .execute(&self.connection_pool)
            .await
        {
            error!(
                "Something went wrong while deleting the session from the database: {}",
                &err
            );
            return Err(());
        };

        Ok(())
    }

    pub async fn add_user(
        &self,
        github_id: u32,
        username: String,
        email: String,
        profile_picture_url: String,
    ) -> Result<u32, ()> {
        match sqlx::query(
            "INSERT INTO users (github_id, username, email, profile_picture_url) VALUES (?1, ?2, ?3, ?4);",
        )
        .bind(github_id)
        .bind(username)
        .bind(email)
        .bind(profile_picture_url)
        .execute(&self.connection_pool)
        .await
        {
            Ok(row) => Ok(u32::try_from(row.last_insert_rowid()).unwrap()),
            Err(err) => {
            error!(
                "Something went wrong while inserting the user into the database: {}",
                &err
            );
            Err(())
            }
        }
    }

    pub async fn get_user(&self, user_id: u32) -> Result<User, ()> {
        match sqlx::query("SELECT * FROM users WHERE id = ?1;")
            .bind(user_id)
            .fetch_one(&self.connection_pool)
            .await
        {
            Ok(row) => {
                let (id, github_id, username, email, profile_picture_url): (
                    u32,
                    u32,
                    String,
                    String,
                    String,
                ) = (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4));

                return Ok(User {
                    id,
                    github_id,
                    username,
                    email,
                    profile_picture_url,
                });
            }
            Err(err) => {
                error!(
                    "Something went wrong while retrieving the user from the database: {}",
                    &err
                );
                return Err(());
            }
        };
    }

    pub async fn get_user_by_github_id(&self, github_id: u32) -> Result<User, ()> {
        match sqlx::query("SELECT * FROM users WHERE github_id = ?1;")
            .bind(github_id)
            .fetch_one(&self.connection_pool)
            .await
        {
            Ok(row) => {
                let (id, github_id, username, email, profile_picture_url): (
                    u32,
                    u32,
                    String,
                    String,
                    String,
                ) = (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4));

                return Ok(User {
                    id,
                    github_id,
                    username,
                    email,
                    profile_picture_url,
                });
            }
            Err(err) => {
                error!(
                    "Something went wrong while retrieving the user from the database: {}",
                    &err
                );
                return Err(());
            }
        };
    }

    pub async fn get_todo_items(&self, user_id: u32) -> Result<Vec<TodoItem>, sqlx::Error> {
        sqlx::query_as::<_, TodoItem>("SELECT * FROM todo_items WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(&self.connection_pool)
            .await
    }

    pub async fn add_todo_item(&self, user_id: u32, content: String) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO todo_items (content, done, user_id) VALUES (?, 0, ?)")
            .bind(content)
            .bind(user_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn update_todo_item(
        &self,
        user_id: u32,
        id: u32,
        done: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE todo_items SET done = ? WHERE id = ? AND user_id = ?")
            .bind(done)
            .bind(id)
            .bind(user_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn delete_todo_item(&self, user_id: u32, id: u32) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM todo_items WHERE id = ? AND user_id = ?")
            .bind(id)
            .bind(user_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}

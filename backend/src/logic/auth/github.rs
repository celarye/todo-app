//! ```sh
//! GITHUB_CLIENT_ID=xxx GITHUB_CLIENT_SECRET=yyy make run_release
//! ```

use std::env;

use oauth2::basic::BasicClient;
use oauth2::reqwest;
use oauth2::url::Url;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize)]
struct GitHubUser {
    id: u32,
    login: String,
    avatar_url: String,
}

#[derive(Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
}

// TODO: Make struct instance for the client

pub fn init() -> (Url, CsrfToken) {
    let github_client_id = ClientId::new(
        env::var("GITHUB_CLIENT_ID").expect("Missing the GITHUB_CLIENT_ID environment variable."),
    );
    let github_client_secret = ClientSecret::new(
        env::var("GITHUB_CLIENT_SECRET")
            .expect("Missing the GITHUB_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Github OAuth2 process.
    let client = BasicClient::new(github_client_id)
        .set_client_secret(github_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(
            RedirectUrl::new(String::from("https://todo.celarye.dev"))
                .expect("Invalid redirect URL"),
        );

    // Generate the authorization URL to which we'll redirect the user.
    client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url()
}

pub async fn success(code: String) -> Result<(u32, String, String, String), ()> {
    let github_client_id = ClientId::new(
        env::var("GITHUB_CLIENT_ID").expect("Missing the GITHUB_CLIENT_ID environment variable."),
    );
    let github_client_secret = ClientSecret::new(
        env::var("GITHUB_CLIENT_SECRET")
            .expect("Missing the GITHUB_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    let client = BasicClient::new(github_client_id)
        .set_client_secret(github_client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(
            RedirectUrl::new(String::from("https://todo.celarye.dev"))
                .expect("Invalid redirect URL"),
        );

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Exchange the code with a token.
    let token_res = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(&http_client)
        .await;

    let Ok(token) = token_res else {
        error!("No access token returned by GitHub");
        return Err(());
    };

    let access_token = token.access_token().secret();

    let p_user_res = http_client
        .get("https://api.github.com/user")
        .header("User-Agent", "celarye-todo-app")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;

    let user = match p_user_res {
        Ok(user_res) => match user_res.json::<GitHubUser>().await {
            Ok(user) => user,
            Err(err) => {
                error!("Failed to fetch the GitHub user profile: {}", err);
                return Err(());
            }
        },
        Err(err) => {
            error!("Failed to fetch the GitHub user profile: {}", err);
            return Err(());
        }
    };

    let p_emails_res = http_client
        .get("https://api.github.com/user/emails")
        .header("User-Agent", "celarye-todo-app")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;

    let emails = match p_emails_res {
        Ok(email_res) => match email_res.json::<Vec<GitHubEmail>>().await {
            Ok(emails) => emails,
            Err(err) => {
                error!("Failed to fetch the GitHub email: {}", err);
                return Err(());
            }
        },
        Err(err) => {
            error!("Failed to fetch the GitHub email: {}", err);
            return Err(());
        }
    };

    let primary_email = emails
        .into_iter()
        .find(|e| e.primary)
        .map(|e| e.email)
        .unwrap_or_else(|| String::from("https://avatars.githubusercontent.com/u/96624179"));

    Ok((user.id, user.login, primary_email, user.avatar_url))
}

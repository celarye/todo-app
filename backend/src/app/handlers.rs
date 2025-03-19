use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::app::AppData;

#[derive(Serialize)]
struct Root {
    user_count: u32,
}

#[derive(Serialize)]
pub struct User {
    pub id: u32,
    pub github_id: u32,
    pub username: String,
    pub email: String,
    pub profile_picture_url: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct GitHubInit {
    pub redirect_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitHubSucces {
    pub code: String,
    pub csrf_token: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct TodoItem {
    pub id: i32,
    pub content: String,
    pub done: bool,
    pub user_id: i32,
}

#[derive(Deserialize)]
pub struct NewTodoItem {
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateTodoItem {
    pub done: bool,
}

pub async fn root(data: web::Data<AppData>) -> impl Responder {
    let Ok(user_count) = data.logic.user_count().await else {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok().json(Root { user_count })
}

pub async fn info(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    if let Some(session) = req.cookie("sessionid") {
        if let Ok(user_id) = data.logic.validate(session).await {
            if let Ok(user) = data.logic.get_user(user_id).await {
                return HttpResponse::Ok().json(user);
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

// Auth

pub async fn github_init(data: web::Data<AppData>) -> impl Responder {
    let Ok(redirect_url) = data.logic.github_init().await else {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Ok().json(GitHubInit { redirect_url })
}

pub async fn github_success(
    data: web::Data<AppData>,
    github_success: web::Json<GitHubSucces>,
) -> impl Responder {
    if let Ok(session_value) = data
        .logic
        .github_success(&github_success.code, &github_success.csrf_token)
        .await
    {
        return HttpResponse::Ok()
            .insert_header((
                "Set-Cookie",
                format!(
                    "sessionid={}; Max-Age={}; HttpOnly; Path=/; Secure; Partitioned;",
                    session_value,
                    60 * 60 * 6
                ),
            ))
            .finish();
    };

    return HttpResponse::Unauthorized().finish();
}

pub async fn logout(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    if let Some(session) = req.cookie("sessionid") {
        if let Err(()) = data.logic.logout(session).await {
            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::Unauthorized().finish()
}

// Todo

pub async fn get_items(req: HttpRequest, data: web::Data<AppData>) -> impl Responder {
    let Some(session) = req.cookie("sessionid") else {
        return HttpResponse::Unauthorized().finish();
    };

    let Ok(user_id) = data.logic.validate(session).await else {
        return HttpResponse::Unauthorized().finish();
    };

    match data.logic.get_items(user_id).await {
        Ok(items) => return HttpResponse::Ok().json(items),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }
}

pub async fn set_item(
    req: HttpRequest,
    data: web::Data<AppData>,
    json: web::Json<NewTodoItem>,
) -> impl Responder {
    let Some(session) = req.cookie("sessionid") else {
        return HttpResponse::Unauthorized().finish();
    };

    let Ok(user_id) = data.logic.validate(session).await else {
        return HttpResponse::Unauthorized().finish();
    };

    match data.logic.add_item(user_id, json.content.clone()).await {
        Ok(_) => return HttpResponse::Created().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_item(
    req: HttpRequest,
    data: web::Data<AppData>,
    path: web::Path<u32>,
    json: web::Json<UpdateTodoItem>,
) -> impl Responder {
    let Some(session) = req.cookie("sessionid") else {
        return HttpResponse::Unauthorized().finish();
    };

    let Ok(user_id) = data.logic.validate(session).await else {
        return HttpResponse::Unauthorized().finish();
    };

    let item_id = path.into_inner();

    match data.logic.update_item(user_id, item_id, json.done).await {
        Ok(_) => return HttpResponse::Ok().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }
}

pub async fn delete_item(
    req: HttpRequest,
    data: web::Data<AppData>,
    path: web::Path<u32>,
) -> impl Responder {
    let Some(session) = req.cookie("sessionid") else {
        return HttpResponse::Unauthorized().finish();
    };

    let Ok(user_id) = data.logic.validate(session).await else {
        return HttpResponse::Unauthorized().finish();
    };

    let item_id = path.into_inner();

    match data.logic.delete_item(user_id, item_id).await {
        Ok(_) => return HttpResponse::Ok().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    }
}

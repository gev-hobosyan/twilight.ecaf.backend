use std::env;
use std::io::empty;
use std::path::PathBuf;
use std::sync::Arc;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use krilla::color::rgb::Color;
use krilla::Document;
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, Paint};
use krilla::text::{Font, TextDirection};
use sea_orm::{ColumnTrait, EntityTrait, Iden, QueryFilter};
use sea_orm::prelude::Uuid;
use crate::api::APIResponse;
use crate::app_state::AppState;
use crate::entity::auth::prelude::User;
use crate::entity::auth::user;
use crate::entity::public::prelude::{Project, Team};
use crate::entity::public::{project, team};
use crate::pdf_utils::{draw_title, draw_table, TableDrawable, text_size};

struct UserTableElement {
    name: String,
    project: String,
    team: String
}

impl TableDrawable for UserTableElement {
    fn into_vec(self) -> Vec<String> {
        vec![self.name, self.team, self.project]
    }
}

fn check_api_key(headers: &HeaderMap, ) -> bool {
    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());

    match api_key {
        Some(api_key) => {
            let original = env::var("ADMIN_API_KEY").unwrap();

            if api_key != original {
                return false;
            }

            true
        }
        None => {
            false
        }
    }
}

async fn get_participants(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<Vec<user::Model>> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let users = User::find()
        .filter(user::Column::Role.eq("user"))
        .all(&state.db)
        .await;

    if let Err(_) = users {
        return APIResponse::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Connection Failed".into()
        )
    }

    APIResponse::Json(users.unwrap())
}

async fn get_projects(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<Vec<Project>>{
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let projects = project::Entity::find().all(&state.db).await;

    if let Err(_) = projects {
        return APIResponse::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Connection failed".into()
        )
    }

    APIResponse::Json(projects.unwrap())
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>
) -> APIResponse<user::Model> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        );
    }

    let user = User::find_by_id(id)
        .one(&state.db).await;

    match user {
        Ok(user) => {
            match user {
                Some(user) => {
                    APIResponse::Json(user)
                }
                None => {
                    APIResponse::Error(
                        StatusCode::NOT_FOUND,
                        "Not found".into()
                    )
                }
            }
        }
        Err(_) => {
            APIResponse::Error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Connection failed".into()
            )
        }
    }
}

async fn get_teams(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> APIResponse<Vec<Team>> {
    if check_api_key(&headers) == false {
        return APIResponse::Error(
            StatusCode::BAD_REQUEST,
            "API key not provided".into()
        )
    }

    let teams = team::Entity::find()
        .all(&state.db).await;

    if let Err(_) = teams {
        return APIResponse::Error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Connection failed".into()
        )
    }

    APIResponse::Json(teams.unwrap())
}

#[axum::debug_handler]
async fn get_pdf(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap
) -> Result<impl IntoResponse, StatusCode> {
    if check_api_key(&headers) == false {
        return Err(StatusCode::BAD_REQUEST);
    }

    let users = User::find()
        // TODO: uncomment
        // .filter(user::Column::Role.eq("user"))
        .all(&state.db).await.unwrap_or(vec![]);

    let projects = project::Entity::find()
        .all(&state.db)
        .await;

    let teams = team::Entity::find()
        .all(&state.db)
        .await.unwrap_or(vec![]);

    let mut data: Vec<UserTableElement> = vec![];

    for user in users.clone() {
        let project = user.project.unwrap_or(0);
        let team = user.team.unwrap_or(0);

        let mut user = UserTableElement {
            name: user.name.clone(),
            project: String::from("None"),
            team: String::from("None")
        };

        if project > 0 {
            let project = project::Entity::find_by_id(project)
                .one(&state.db)
                .await;

            if let Ok(Some(project)) = project {
                user.project = project.name;
            }
        }

        if team > 0 {
            let team = team::Entity::find_by_id(team)
                .one(&state.db)
                .await;

            if let Ok(Some(team)) = team {
                user.team = team.name.unwrap();
            }
        }

        data.push(user);
    }

    let font = {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts/Roboto-Regular.ttf");
        let data = std::fs::read(&path).unwrap();
        Font::new(data.into(), 0).unwrap()
    };

    let mut document = Document::new();

    let page_width = 210.0 * (72.0 / 25.4);
    let page_height = 297.0 * (72.0 / 25.4);

    let mut page = document.start_page_with(PageSettings::from_wh(page_width, page_height).unwrap());
    let mut surface = page.surface();

    let (_, title_y) = draw_title(&mut surface, page_width, page_height, "Participants", 20.0, font.clone());

    draw_table(&mut surface, page_width, page_height, vec!["Name", "Team", "Project"], data, 16.0, font.clone(), title_y + 20.0);

    surface.finish();
    page.finish();

    if let Ok(projects) = projects {
        for project in projects {
            let mut page = document.start_page_with(PageSettings::from_wh(page_width, page_height).unwrap());
            let mut surface = page.surface();

            let (_, title_y) = draw_title(&mut surface, page_width, page_height, project.name.as_str(), 20.0, font.clone());

            let (owner, team_id): (String, i32) = match users.iter().find(|user| user.id == project.owner.unwrap_or(Uuid::new_v4())) {
                Some(owner) => {
                    (owner.name.clone(), owner.team.unwrap_or(0))
                }
                None => {
                    ("None".to_string(), 0)
                }
            };

            let team = match teams.iter().find(|team| team.id == team_id) {
                Some(team) => {
                    team.name.clone().unwrap_or("None".to_string()).clone()
                }
                None => {"None".to_string()}
            };

            let mut y = title_y + 50.0;

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                format!("Owner: {}", owner).as_str(),
                false,
                TextDirection::Auto
            );

            y += 25.0;

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                format!("Team: {}", team).as_str(),
                false,
                TextDirection::Auto
            );

            y += 25.0;

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                "Use AI:",
                false,
                TextDirection::Auto
            );

            let x = 30.0 + text_size(18.0, "Use AI:").0;

            let paint = Paint::from(match project.use_ai {
                true => {
                    Color::new(19, 143, 13)
                }
                false => {
                    Color::new(255, 0, 0)
                }
            });

            let rect = {
                let mut pb = PathBuilder::new();
                pb.move_to(x, y - 13.0);
                pb.line_to(x + 15.0, y - 13.0);
                pb.line_to(x + 15.0, y + 2.0);
                pb.line_to(x, y + 2.0);

                pb.close();
                pb.finish().unwrap()
            };

            surface.set_fill(
                Some(
                    Fill {
                        paint,
                        rule: FillRule::EvenOdd,
                        opacity: NormalizedF32::ONE
                    }
                )
            );

            surface.draw_path(
                &rect
            );

            surface.set_fill(None);

            y += 25.0;

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                format!("Tech Stack: {}", project.tech_stack).as_str(),
                false,
                TextDirection::Auto
            );

            y += 25.0;

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                "Description:",
                false,
                TextDirection::Auto
            );

            y += 25.0;
            let description: Vec<&str> = project.description.split("\n").filter(|text| !text.eq(&"")).collect();

            if description.len() <= 20 {
                for line in description {
                    surface.draw_text(
                        Point::from_xy(45.0, y),
                        font.clone(),
                        18.0,
                        line,
                        false,
                        TextDirection::Auto
                    );

                    y += 25.0;
                }
            } else {
                surface.draw_text(
                    Point::from_xy(45.0, y),
                    font.clone(),
                    18.0,
                    "Description too long",
                    false,
                    TextDirection::Auto
                );

                y += 25.0;
            }


            let story: Vec<&str> = project.story.split("\n").filter(|text| !text.eq(&"")).collect();

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                "Story:",
                false,
                TextDirection::Auto
            );

            y += 25.0;

            if story.len() <= 20 {
                for line in story {
                    surface.draw_text(
                        Point::from_xy(45.0, y),
                        font.clone(),
                        18.0,
                        line,
                        false,
                        TextDirection::Auto
                    );

                    y += 25.0;
                }
            } else {
                surface.draw_text(
                    Point::from_xy(45.0, y),
                    font.clone(),
                    18.0,
                    "Story too long",
                    false,
                    TextDirection::Auto
                );

                y += 25.0;
            }

            surface.draw_text(
                Point::from_xy(20.0, y),
                font.clone(),
                18.0,
                "Notes:",
                false,
                TextDirection::Auto
            );

            y += 25.0;

            match project.notes {
                Some(notes) => {
                    let notes: Vec<&str> = notes.split("\n").filter(|text| !text.eq(&"")).collect();

                    if notes.len() <= 20 {
                        for line in notes {
                            surface.draw_text(
                                Point::from_xy(45.0, y),
                                font.clone(),
                                18.0,
                                line,
                                false,
                                TextDirection::Auto
                            );

                            y += 25.0;
                        }
                    } else {
                        surface.draw_text(
                            Point::from_xy(45.0, y),
                            font.clone(),
                            18.0,
                            "Notes too long",
                            false,
                            TextDirection::Auto
                        );

                        y += 25.0;
                    }
                }
                None => {
                    surface.draw_text(
                        Point::from_xy(45.0, y),
                        font.clone(),
                        18.0,
                        "Notes not provided",
                        false,
                        TextDirection::Auto
                    );

                    y += 25.0;
                }
            }

            surface.finish();
            page.finish();
        }
    }

    let pdf = document.finish().unwrap();

    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE, "application/pdf".parse().unwrap()
    );

    headers.insert(
        header::CONTENT_DISPOSITION,
        "inline; filename=\"admin.pdf\"".parse().unwrap()
    );

    Ok((headers, Bytes::from(pdf)))
}

pub fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admin/users", get(get_participants))
        .route("/admin/projects", get(get_projects))
        .route("/admin/user/{id}", get(get_user))
        .route("/admin/teams", get(get_teams))
        .route("/admin/pdf", get(get_pdf))
}
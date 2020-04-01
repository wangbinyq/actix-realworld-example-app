use crate::db::{new_pool, DbExecutor};
use actix::prelude::{Addr, SyncArbiter};
use actix_web::{middleware::Logger, web, web::Data, App, HttpRequest, HttpServer};
use std::env;

pub mod articles;
pub mod profiles;
pub mod tags;
pub mod users;

pub struct AppState {
    pub db: Addr<DbExecutor>,
}

async fn index(_state: Data<AppState>, _req: HttpRequest) -> &'static str {
    "Hello world!"
}

pub async fn start() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_pool = new_pool(database_url).expect("Failed to create pool.");
    let database_address =
        SyncArbiter::start(num_cpus::get(), move || DbExecutor(database_pool.clone()));

    let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS is not set");

    let server = HttpServer::new(move || {
        let state = AppState {
            db: database_address.clone(),
        };
        App::new()
            .data(Data::new(state))
            .wrap(Logger::default())
            .configure(routes)
    })
    .bind(&bind_address)
    .unwrap_or_else(|_| panic!("Could not bind server to address {}", &bind_address))
    .run();

    println!("You can access the server at {}", bind_address);

    match server.await {
        Ok(_) => "",
        Err(_) => "",
    };

    return;
}

fn routes(app: &mut web::ServiceConfig) {
    app.service(web::resource("/").to(index)).service(
        web::scope("/api")
            // User routes ↓
            .service(web::resource("users").route(web::post().to(users::register)))
            .service(web::resource("users/login").route(web::post().to(users::login)))
            .service(
                web::resource("user")
                    .route(web::get().to(users::get_current))
                    .route(web::put().to(users::update)),
            )
            // Profile routes ↓
            .service(web::resource("profiles/{username}").route(web::get().to(profiles::get)))
            .service(
                web::resource("profiles/{username}/follow")
                    .route(web::post().to(profiles::follow))
                    .route(web::delete().to(profiles::unfollow)),
            )
            // Article routes ↓
            .service(
                web::resource("articles")
                    .route(web::get().to(articles::list))
                    .route(web::post().to(articles::create)),
            )
            .service(web::resource("articles/feed").route(web::get().to(articles::feed)))
            .service(
                web::resource("articles/{slug}")
                    .route(web::get().to(articles::get))
                    .route(web::put().to(articles::update))
                    .route(web::delete().to(articles::delete)),
            )
            .service(
                web::resource("articles/{slug}/favorite")
                    .route(web::post().to(articles::favorite))
                    .route(web::delete().to(articles::unfavorite)),
            )
            .service(
                web::resource("articles/{slug}/comments")
                    .route(web::get().to(articles::comments::list))
                    .route(web::post().to(articles::comments::add)),
            )
            .service(
                web::resource("articles/{slug}/comments/{comment_id}")
                    .route(web::delete().to(articles::comments::delete)),
            )
            // Tags routes ↓
            .service(web::resource("tags").route(web::get().to(tags::get))),
    );
}

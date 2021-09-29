use axum::{
    extract::Extension,
    handler::get,
    http::StatusCode,
    response::{Html, IntoResponse},
    service, AddExtensionLayer, Router,
};
use handlebars::Handlebars;
use rs_consul::{Config, Consul};
use std::convert::Infallible;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

async fn root(
    Extension(engine): Extension<Handlebars<'_>>,
    Extension(config): Extension<Config>,
) -> impl IntoResponse {
    let data = Consul::new(config.clone())
        .get_all_registered_service_names(None)
        .await
        .unwrap_or_else(|_| panic!("Could not connect to consul server on {}", config.address))
        .response;

    let content = engine
        .render("index", &data)
        .expect("Invalid handlebar syntax on index.hbs");
    Html(content)
}

#[tokio::main]
async fn main() {
    let file = "index.hbs"; // TODO get from cli
    let mut engine = Handlebars::new();
    engine.set_dev_mode(true); // TODO set from cli
    engine
        .register_template_file("index", file)
        .expect("File 'index.hbs' not found on current dir");

    let config = Config::from_env(); // TODO set from cli

    let app = Router::new()
        .nest(
            "/",
            service::get(ServeDir::new(".")).handle_error(|error: std::io::Error| {
                Ok::<_, Infallible>((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                ))
            }),
        )
        .route("/", get(root))
        .layer(AddExtensionLayer::new(engine))
        .layer(AddExtensionLayer::new(config));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // TODO read from cli
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|_| panic!("Could not start server on the current address: {:?}", addr));
}

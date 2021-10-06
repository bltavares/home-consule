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
use structopt::StructOpt;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "home-consule",
    about = "A (micro) dashboard for your homelab integrated with Consul",
    rename_all = "kebab-case"
)]
struct Args {
    #[structopt(long, env = "CONSUL_HTTP_TOKEN", hide_env_values = true)]
    consul_http_token: Option<String>,
    #[structopt(
        long,
        env = "CONSUL_HTTP_ADDR",
        default_value = "http://localhost:8500"
    )]
    consul_http_addr: String,

    /// Monitors the index.hbs file for changes on every request. Disabled by default.
    #[structopt(short = "d", long)]
    autoreload: bool,

    /// The name of the Handlebars file template
    #[structopt(short = "t", long, default_value = "index.hbs")]
    template: String,

    /// The address for the server to listen to
    #[structopt(short, long, default_value = "0.0.0.0:3000")]
    listen: SocketAddr,
}

async fn root(
    Extension(engine): Extension<Handlebars<'_>>,
    Extension(config): Extension<Config>,
) -> impl IntoResponse {
    let mut data = Consul::new(config.clone())
        .get_all_registered_service_names(None)
        .await
        .unwrap_or_else(|_| panic!("Could not connect to consul server on {}", config.address))
        .response;
    data.sort();

    let content = engine
        .render("index", &data)
        .expect("Invalid handlebar syntax on index.hbs");
    Html(content)
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "home_consule=info,tower_http=info")
    }

    tracing_subscriber::fmt::init();

    let file = args.template;
    let mut engine = Handlebars::new();
    engine.set_dev_mode(args.autoreload);
    engine
        .register_template_file("index", &file)
        .unwrap_or_else(|_| panic!("File '{}' not found on current dir", file));

    let config = Config {
        address: args.consul_http_addr,
        token: args.consul_http_token,
    };

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
        .layer(AddExtensionLayer::new(config))
        .layer(TraceLayer::new_for_http());

    let address = args.listen;
    hyper::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Could not start server on the current address: {:?}",
                address
            )
        });
}

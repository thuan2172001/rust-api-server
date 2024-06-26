#[cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use openssl;
#[rustfmt::skip]
#[cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use diesel;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use deadpool_diesel::postgres::Pool;
use deadpool_diesel::{Manager, Runtime};
use opentelemetry::global;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tracing::info;

use adapter::repositories::grpc::gpt_answer_client::GptAnswerClient;
use adapter::repositories::in_memory::question::QuestionInMemoryRepository;
use adapter::repositories::postgres::question_db::QuestionDBRepository;
use cli::options::Options;
use cli::router::Router;
use common::kill_signals;
use common::loggers::telemetry::init_telemetry;
use common::options::parse_options;
use rust_core::ports::question::QuestionPort;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if args.version {
        println!(env!("APP_VERSION"));
        return;
    }

    let options: Options = match parse_options(args.config_path) {
        Ok(options) => options,
        Err(err) => {
            println!("Failed to load config: {}", err);
            return;
        }
    };

    if let Some(Commands::Config) = args.command {
        println!("{:#?}", options);
        return;
    }

    init_telemetry(
        options.service_name.as_str(),
        options.exporter_endpoint.as_str(),
        options.log.level.as_str(),
    );

    let (tx, rx) = oneshot::channel();
    let server = tokio::spawn(serve(options, rx));

    kill_signals::wait_for_kill_signals().await;

    // Send the shutdown signal
    let _ = tx.send(());

    // Wait for the server to finish shutting down
    tokio::try_join!(server).expect("Failed to run server");

    global::shutdown_tracer_provider();
    info!("Shutdown successfully!");
}

/// Simple REST server.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Config file
    #[arg(short, long, default_value = "config/00-default.toml")]
    config_path: Vec<String>,
    /// Print version
    #[clap(short, long)]
    version: bool,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Print config
    Config,
}

pub async fn serve(options: Options, rx: Receiver<()>) {
    let question_port: Arc<dyn QuestionPort + Send + Sync> = if options.db.in_memory.is_some() {
        info!("Using in-memory database");
        Arc::new(QuestionInMemoryRepository::new())
    } else if options.db.pg.is_some() {
        let database_config = options.db.pg.clone().unwrap();
        info!("Using postgres database: {}", database_config.url);
        let manager = Manager::new(database_config.url, Runtime::Tokio1);
        let pool = Pool::builder(manager)
            .max_size(database_config.max_size)
            .build()
            .unwrap();
        Arc::new(QuestionDBRepository::new(pool))
    } else {
        info!("No database specified, falling back to in-memory");
        Arc::new(QuestionInMemoryRepository::new())
    };

    let gpt_answer_client =
        Arc::new(GptAnswerClient::new(options.gpt_answer_service_url.to_string()).unwrap());

    let router = Router::new(question_port, gpt_answer_client);
    let routes = router.routes();
    let address = SocketAddrV4::new(
        Ipv4Addr::from_str(options.server.url.as_str()).unwrap(),
        options.server.port,
    );
    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(address, async {
        rx.await.ok();
        info!("Warp server shut down");
    });

    server.await;
}

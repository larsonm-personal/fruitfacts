use harvest_chart_server::import_db;
use harvest_chart_server::queries;

use actix_cors::Cors;

use actix_web::{middleware::Logger, App, HttpServer};

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

extern crate clap;
use clap::{crate_version, Arg, Command as ClapApp};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = ClapApp::new("")
        .version(crate_version!())
        .arg(
            Arg::new("reload_db")
                .short('r')
                .long("reload_db")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .help("reload db"),
        )
        .get_matches();

    let mut db_conn = import_db::establish_connection();
    if matches.get_flag("reload_db") {
        import_db::reset_database(&mut db_conn);
        let items_loaded = import_db::load_all(&mut db_conn);

        if items_loaded.base_plants_found == 0 {
            panic!("directory \"plant_database\" not found");
        }
        std::process::exit(0);
    } else if import_db::count_base_plants(&mut db_conn) == 0 {
        panic!(
            r#"no plants found in database, import the database first with "cargo run -- --reload_db""#
        )
    }

    // run this so we load the gazetteer database with lazy_static before we get a query
    let coords = harvest_chart_server::gazetteer_load::from_to_location("zip:97231");
    println!("loaded a sample zip coordinate in order to trigger lazy_static: {coords:?}");

    let connspec = "database.sqlite3";
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    println!("starting http server");
    HttpServer::new(move || {
        let cors = Cors::default()
            .supports_credentials()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_origin_fn(|origin, _req_head| {
                let matched = origin
                    .as_bytes()
                    .ends_with(env!("FRONTEND_BASE").to_string().as_bytes());

                if !matched {
                    println!(
                        "cors failed, got {:?} expected {:?}",
                        origin,
                        env!("FRONTEND_BASE")
                    );
                }
                matched
                // todo - better handling of port for dev/release
            });

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            // set up DB pool to be used with web::Data<Pool> extractor
            .app_data(actix_web::web::Data::new(pool.clone()))
            .service(queries::get_collections)
            .service(queries::get_reference_category)
            .service(queries::get_recent_changes)
            .service(queries::get_fact)
            .service(queries::auth::get_auth_urls)
            .service(queries::auth::receive_oauth_redirect)
            .service(queries::auth::create_account)
            .service(queries::auth::get_user)
            .service(queries::auth::delete_user)
            .service(queries::auth::check_login)
            .service(queries::auth::logout)
            // todo - some of these can all be combined under the big search query
            .service(queries::get_plant)
            .service(queries::search::variety_search)
            .service(queries::list::create_list)
            .service(queries::list::add_plant_to_list)
            // combine later?
            .service(queries::map::locations_search)
    })
    .bind(("0.0.0.0", env!("BACKEND_PORT").parse::<u16>().unwrap()))? // 0.0.0.0 is actix-speak for "all local IPs"
    .run()
    .await
}

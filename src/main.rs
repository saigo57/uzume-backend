use std::env;
use axum::{
    extract::{Extension, DefaultBodyLimit},
    Router,
};
use utoipa_swagger_ui::SwaggerUi;
use utoipa::OpenApi;
use rusqlite::Connection;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::model::file::workspace::Workspace;
use crate::model::file::workspace_info::WorkspaceInfo;
use crate::model::file::writer::Writer;
use crate::model::file::writer::FileWriter;
use crate::model::file::config::Config;

mod schema;
mod initialize;
mod controller;
mod model;
mod util;
mod test_util;
mod logger;

const DEFAULT_PORT: u16 = 22113;

#[derive(Debug, PartialEq)]
struct PortArgs {
    port: u16,
}

#[derive(Debug, PartialEq)]
struct CommandArgs {
    args: String,
}

#[derive(Debug, PartialEq)]
struct CreateArgs {
    name: String,
    path: String,
}

#[derive(Debug, PartialEq)]
struct AddArgs {
    path: String,
}

#[derive(Debug, PartialEq)]
enum StartMode {
    Port(PortArgs),
    Help(CommandArgs),
    Create(CreateArgs),
    Add(AddArgs),
}

fn parse_args(args: &[String]) -> Result<StartMode, String> {
    if args.len() == 1 {
        return Ok(StartMode::Port(
            PortArgs { port: DEFAULT_PORT }
        ));
    }

    if args.len() == 2 {
        let args_str: &str = &args[1];
        
        if args_str == "help" {
            return Ok(StartMode::Help(
                CommandArgs { args: "".to_string() }
            ));
        }
        
        // 上に一致しない場合はポート番号として解釈
        let port_str = args_str;
        return match port_str.parse::<u16>() {
            Ok(port) => Ok(StartMode::Port(PortArgs { port })),
            Err(_) => Err(format!("[server mode] invalid port: {}", port_str)),
        };
    }
    
    if args.len() >= 3 {
        let command = &args[1];

        if command == "create" {
            if args.len() != 4 {
                return Err(format!("[create mode] invalid arg num: {}", args.len()));
            }

            return Ok(StartMode::Create(
                CreateArgs { name: args[2].clone(), path: args[3].clone() }
            ));
        }

        if command == "add" {
            if args.len() != 3 {
                return Err(format!("[add mode] invalid arg num: {}", args.len()));
            }

            return Ok(StartMode::Add(
                AddArgs { path: args[2].clone() }
            ));
        }
    }

    Err(format!("invalid arg num: {}", args.len()))
}

fn create_workspace<T: Writer>(writer: &mut T, config: &mut Config, name: &str, path: &str) -> Result<(), std::io::Error> {
    let workspace = Workspace::generate(name);
    workspace.save(writer, path)?;
    
    config.workspace_list.push(
        WorkspaceInfo {
            workspace_id: workspace.workspace_id.clone(),
            name: workspace.name.clone(),
            path: path.to_string(),
        }
    );
    config.save(writer)?;
    
    Ok(())
}

fn add_workspace<T: Writer>(writer: &mut T, config: &mut Config, path: &str) -> Result<(), std::io::Error> {
    let workspace = Workspace::load(path)?;

    config.workspace_list.push(
        WorkspaceInfo {
            workspace_id: workspace.workspace_id.clone(),
            name: workspace.name.clone(),
            path: path.to_string(),
        }
    );
    config.save(writer)?;
    
    Ok(())
}

#[tokio::main]
async fn main() {
    match logger::init_logger() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("logger init error!");
            eprintln!("{}", e);
            return;
        }
    }

    log::info!("start uzume server.");

    let args: Vec<String> = env::args().collect();

    let start_mode = match parse_args(&args) {
        Ok(mode) => mode,
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };
    
    let port = match start_mode {
        StartMode::Port(PortArgs { port }) => port,
        StartMode::Help(_) => {
            println!("help!");
            return;
        },
        StartMode::Create(CreateArgs { name, path }) => {
            match Config::new() {
                Ok(mut config) => {
                    match create_workspace(&mut FileWriter, &mut config, &name, &path) {
                        Ok(_) => {
                            log::info!("workspace created.");
                        },
                        Err(e) => {
                            log::error!("workspace create error!");
                            log::error!("{}", e);
                        }
                    }
                },
                Err(e) => {
                    log::error!("config load error!");
                    log::error!("{}", e);
                    return;
                }
            };
            return;
        },
        StartMode::Add(AddArgs { path }) => {
            match Config::new() {
                Ok(mut config) => {
                    match add_workspace(&mut FileWriter, &mut config, &path) {
                        Ok(_) => {
                            log::info!("workspace added.");
                        },
                        Err(e) => {
                            log::error!("workspace add error!");
                            log::error!("{}", e);
                        }
                    }
                },
                Err(e) => {
                    log::error!("config load error!");
                    log::error!("{}", e);
                    return;
                }
            };
            return;
        },
    };

    log::info!("port: {}", port);

    let conn = match Connection::open_in_memory() {
        Ok(conn) => conn,
        Err(e) => {
            log::error!("connection open error!");
            log::error!("{}", e);
            return;
        }
    };
    let conn = Arc::new(Mutex::new(conn));
    match schema::create_schema(conn.clone()).await {
        Ok(_) => {
            log::info!("schema created.");
        },
        Err(e) => {
            log::error!("schema create error!");
            log::error!("{}", e);
            return;
        }
    }

    match initialize::initialize(conn.clone()).await {
        Ok(_) => {
            log::info!("initialized.");
        },
        Err(e) => {
            log::error!("initialize error!");
            log::error!("{}", e);
            return;
        }
    }

    let writer = FileWriter;
    let v1_api_router = Router::new()
        .nest("/workspaces", controller::workspaces::router::<FileWriter>(conn.clone()))
        .nest("/images", controller::images::router(conn.clone()))
        .nest("/tags", controller::tags::router::<FileWriter>(conn.clone()));
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", controller::workspaces::ApiDoc::openapi()))
        .nest("/api/v1", v1_api_router)
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(Extension(conn))
        .layer(Extension(writer));
    let listener = match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(listener) => listener,
        Err(e) => {
            log::error!("listener bind error!");
            log::error!("{}", e);
            return;
        }
    };
    match axum::serve(listener, app).await {
        Ok(_) => {},
        Err(e) => {
            log::error!("serve error!");
            log::error!("{}", e);
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TestUtil;

    #[test]
    fn test_parse_args_param_is_none() {
        let args = vec!["this_app".to_string()];
        assert_eq!(parse_args(&args), Ok(StartMode::Port(PortArgs { port: DEFAULT_PORT })));
    }

    #[test]
    fn test_parse_args_param_with_port() {
        let port = 12345;
        let args = vec!["this_app".to_string(), port.to_string()];
        assert_ne!(DEFAULT_PORT, port);
        assert_eq!(parse_args(&args), Ok(StartMode::Port(PortArgs { port })));
    }

    #[test]
    fn test_parse_args_param_with_help_command() {
        let args = vec!["this_app".to_string(), "help".to_string()];
        assert_eq!(
            parse_args(&args),
            Ok(StartMode::Help(
                CommandArgs { args: "".to_string() }
            ))
        );
    }

    #[test]
    fn test_parse_args_param_with_create_command() {
        let args = vec![
            "this_app".to_string(),
            "create".to_string(),
            "workspace_name".to_string(),
            "/path/to/hoge.uzume".to_string(),
        ];
        assert_eq!(
            parse_args(&args),
            Ok(StartMode::Create(
                CreateArgs { name: "workspace_name".to_string(), path: "/path/to/hoge.uzume".to_string() }
            ))
        );
    }

    #[test]
    fn test_parse_args_param_with_add_command() {
        let args = vec![
            "this_app".to_string(),
            "add".to_string(),
            "/path/to/hoge.uzume".to_string(),
        ];
        assert_eq!(
            parse_args(&args),
            Ok(StartMode::Add(
                AddArgs { path: "/path/to/hoge.uzume".to_string() }
            ))
        );
    }

    #[test]
    fn test_parse_args_param_with_port_that_not_number() {
        let args = vec!["this_app".to_string(), "not_number".to_string()];
        assert_eq!(parse_args(&args), Err("[server mode] invalid port: not_number".to_string()));
    }

    #[test]
    fn test_parse_args_when_invalid_args_num() {
        let args = vec!["this_app".to_string(), "12345".to_string(), "invalid_args".to_string()];
        assert_eq!(parse_args(&args), Err("invalid arg num: 3".to_string()));
    }

    #[tokio::test]
    async fn test_create_workspace() {
        let mut tu = TestUtil::new().await;

        let mut config = Config {
            workspace_list: [
                WorkspaceInfo {
                    path: tu.workspace_path.clone(),
                    workspace_id: tu.workspace_id.clone(),
                    name: tu.workspace_name.clone(),
                },
            ].to_vec(),
        };
        create_workspace(&mut tu.writer, &mut config, "new_workspace_name", "/path/to/new_workspace.uzume").unwrap();
        
        let config: Config = serde_json::from_str(&tu.writer.get_json()).unwrap();
        assert_eq!(config.workspace_list.len(), 2);
        assert_eq!(config.workspace_list[0].path, tu.workspace_path);
        assert_eq!(config.workspace_list[0].name, tu.workspace_name);
        assert_eq!(config.workspace_list[0].workspace_id, tu.workspace_id);
        assert_eq!(config.workspace_list[1].path, "/path/to/new_workspace.uzume");
        assert_eq!(config.workspace_list[1].name, "new_workspace_name");
        assert_eq!(config.workspace_list[1].workspace_id.len(), 36);
    }

    #[tokio::test]
    async fn test_add_workspace() {
        // workspaceのloadがあるのでテストは一旦省略
    }
}

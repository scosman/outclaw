mod commands;
mod docker;
mod error;
mod github;
mod instance;
mod poller;

pub use error::EasyClawError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::docker::check_docker,
            commands::instances::list_instances,
            commands::instances::get_instance,
            commands::instances::create_instance,
            commands::instances::update_instance,
            commands::instances::delete_instance,
            commands::instances::rename_instance,
            commands::instances::start_instance,
            commands::instances::stop_instance,
            commands::instances::restart_instance,
            commands::instances::build_instance,
            commands::releases::get_releases,
            commands::system::get_system_timezone,
            commands::system::generate_instance_name,
            commands::system::open_in_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

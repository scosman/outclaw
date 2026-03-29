mod commands;
mod docker;
mod error;
mod github;
mod instance;
mod poller;

use std::sync::Arc;

use commands::instances::AppState;
use poller::Poller;
use tauri::Manager;

pub use error::OutClawError;

/// Wrapper to manage poller state across the app
#[derive(Clone)]
pub struct PollerState {
    pub poller: Arc<Poller>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_state = AppState::new();
    let poller = Arc::new(Poller::new());
    let poller_state = PollerState {
        poller: poller.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .manage(poller_state)
        .setup(move |app| {
            // Start the status poller within Tauri's async runtime
            let app_handle = app.handle().clone();
            let poller_clone = poller.clone();
            let state = app.state::<AppState>().inner().clone();

            tauri::async_runtime::spawn(async move {
                poller_clone.start(app_handle, Arc::new(state));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::docker::check_docker,
            commands::docker::set_poller_interval,
            commands::instances::list_instances,
            commands::instances::get_instance,
            commands::instances::create_instance,
            commands::instances::update_instance,
            commands::instances::delete_instance,
            commands::instances::rename_instance,
            commands::instances::start_instance,
            commands::instances::stop_instance,
            commands::instances::restart_instance,
            commands::instances::restart_gateway,
            commands::instances::build_instance,
            commands::instances::cancel_build,
            commands::instances::connect_provider,
            commands::instances::connect_whatsapp,
            commands::instances::add_telegram_channel,
            commands::instances::approve_telegram_pairing,
            commands::releases::get_releases,
            commands::system::get_system_timezone,
            commands::system::generate_instance_name,
            commands::system::open_in_browser,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

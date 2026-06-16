pub mod backups;
pub mod binder;
pub mod commands;
pub mod database;
pub mod documents;
pub mod domain;
pub mod errors;
pub mod filesystem;
pub mod inspector;
pub mod locks;
pub mod migrations;
pub mod project;
pub mod search_export;
pub mod time;

pub fn run() {
    tauri::Builder::default()
        .manage(commands::CommandState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::create_project,
            commands::create_default_project,
            commands::open_project,
            commands::close_project,
            commands::create_backup,
            commands::list_backups,
            commands::list_binder_items,
            commands::create_binder_item,
            commands::rename_binder_item,
            commands::set_binder_item_expanded,
            commands::duplicate_binder_item,
            commands::move_binder_item,
            commands::reorder_binder_items,
            commands::trash_binder_item,
            commands::restore_binder_item,
            commands::get_document,
            commands::save_document,
            commands::record_document_recovery,
            commands::get_document_recovery,
            commands::clear_document_recovery,
            commands::get_inspector_data,
            commands::save_binder_synopsis,
            commands::save_document_metadata,
            commands::save_project_note,
            commands::create_snapshot,
            commands::restore_snapshot,
            commands::search_project,
            commands::export_project
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Local Writer");
}

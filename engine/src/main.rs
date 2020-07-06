// There's a decent amount of code that's just for example and isn't called
#![allow(dead_code)]

fn main() {
    #[allow(unused_assignments)]
    let mut log_level = log::LevelFilter::Info;
    //#[cfg(debug_assertions)]
    {
        log_level = log::LevelFilter::Debug;
    }

    // Setup logging
    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .filter_module(
            "renderer_resources::resource_managers::descriptor_sets",
            log::LevelFilter::Info,
        )
        .filter_module("minimum_editor::resources::editor_selection", log::LevelFilter::Trace)
        .filter_module("renderer_base", log::LevelFilter::Info)
        .filter_level(log_level)
        // .format(|buf, record| { //TODO: Get a frame count in here
        //     writeln!(buf,
        //              "{} [{}] - {}",
        //              chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
        //              record.level(),
        //              record.args()
        //     )
        // })
        .init();

    // Spawn the daemon in a background thread. This could be a different process, but
    // for simplicity we'll launch it here.
    std::thread::spawn(move || {
        minimum::daemon::run();
    });

    engine::run();
}

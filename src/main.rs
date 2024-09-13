mod mmo_server;
use std::time::Duration;
use bevy_replicon::RepliconPlugins;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use mmo_server::MmoGameNodePlugin;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 30.0,
            ))),
            RepliconPlugins,
            RepliconRenetPlugins,
            MmoGameNodePlugin,
        ))
        .run();
}


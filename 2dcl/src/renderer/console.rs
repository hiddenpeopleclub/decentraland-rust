use bevy::prelude::*;
use bevy_console::{reply, AddConsoleCommand, ConsoleCommand, ConsolePlugin};
use dcl_common::Parcel;

use crate::renderer::scene_loader::get_parcel_spawn_point;

use super::player::PlayerComponent;
use super::scene_maker::RoadsData;
use super::scenes_io::SceneFilesMap;

pub struct MyConsolePlugin;

impl Plugin for MyConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ConsolePlugin)
            .add_console_command::<TeleportCommand, _, _>(teleport_command);
        // .add_console_command::<WhereCommand, _, _>(where_command);
    }
}

/// Teleports the player to a specific parcel
#[derive(ConsoleCommand)]
#[console_command(name = "jump")]
struct TeleportCommand {
    parcel_x: String,
    parcel_y: String,
}

/*
/// Teleports the player to a specific parcel
#[derive(ConsoleCommand)]
#[console_command(name = "where")]
struct WhereCommand;

fn where_command(
  mut where_cmd: ConsoleCommand<WhereCommand>,
  mut player_query: Query<&PlayerComponent>,
)
{
  let player = player_query.single_mut();
  if let _ = where_cmd.take() {

    reply!(where_cmd, "You're in the parcel {},{}", player.current_parcel.0, player.current_parcel.1);
  }
} */

fn teleport_command(
    mut tp: ConsoleCommand<TeleportCommand>,
    mut player_query: Query<(&mut PlayerComponent, &mut Transform)>,
    mut roads_data: ResMut<RoadsData>,
    scene_files_map: Res<SceneFilesMap>,
) {
    let (mut player, mut transform) = player_query.single_mut();
    if let Some(TeleportCommand { parcel_x, parcel_y }) = tp.take() {
        if let Ok(parcel_x) = parcel_x.parse::<i16>() {
            if let Ok(parcel_y) = parcel_y.parse::<i16>() {
                player.current_level = 0;
                transform.translation = get_parcel_spawn_point(
                    &Parcel(parcel_x, parcel_y),
                    0,
                    &mut roads_data,
                    &scene_files_map,
                );
                reply!(tp, "teleporting to parcel {},{}", parcel_x, parcel_y);
            } else {
                reply!(tp, "{} is not a valid value", parcel_y);
            }
        } else {
            reply!(tp, "{} is not a valid value", parcel_x);
        }
    }
}

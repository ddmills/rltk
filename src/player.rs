use super::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

    let map = ecs.fetch::<Map>();

    for (entity, _player, pos, viewshed) in
        (&entities, &mut players, &mut positions, &mut viewsheds).join()
    {
        let x = pos.x + delta_x;
        let y = pos.y + delta_y;
        let destination_idx = map.xy_idx(x, y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);

            if let Some(_target) = target {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add melee target failed");
            }

            match target {
                None => {}
                Some(_t) => {
                    console::log(&format!("From Hell's Heart, I stab thee!"));
                    return; // So we don't move after attacking
                }
            }
        }

        if map.tiles[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, x));
            pos.y = min(49, max(0, y));
            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
        }
    }
}

fn try_pickup_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::A => {
                try_move_player(-1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::D => {
                try_move_player(1, 0, &mut gs.ecs)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::W => {
                try_move_player(0, -1, &mut gs.ecs)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::S => {
                try_move_player(0, 1, &mut gs.ecs)
            }
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::G => try_pickup_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::V => return RunState::ShowDropItem,
            VirtualKeyCode::Escape => return RunState::SaveGame,
            _ => return RunState::AwaitingInput,
        },
    }

    RunState::PlayerTurn
}

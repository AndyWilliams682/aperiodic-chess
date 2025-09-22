mod graph_boards;
mod limited_int;
mod position;
mod chess_move;
mod move_generator;
mod piece_set;
mod movement_tables;
mod evaluator;
mod game;
mod engine;
mod bit_board;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_picking::prelude::*;

use graph_boards::traditional_board::TraditionalBoardGraph;
use graph_boards::hexagonal_board::HexagonalBoardGraph;
use position::Position;
use graph_boards::graph_board::TileIndex;

use crate::{engine::Engine, game::Game, graph_boards::graph_board::Tile, limited_int::LimitedInt};

#[derive(Component, Debug, Clone, Copy)]
pub struct GraphEdge {
    pub start_tile_id: u32,
    pub end_tile_id: u32,
}

#[derive(Component)]
struct MoveIndicator;

#[derive(Resource)]
struct CurrentTurnLabel(Entity);

#[derive(Resource, Default)]
struct GraphState {
    tile_count: u32,
    edge_count: u32,
}

#[derive(Resource, Default)]
struct SelectedTile {
    entity: Option<Entity>,
    tile_index: Option<TileIndex>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            DefaultPickingPlugins,
        ))
        .insert_resource(GraphState::default())
        .insert_resource(Game {
            engine: Engine::new(TraditionalBoardGraph::new().0.move_tables()),
            are_players_cpu: [false, true],
            current_position: Position::new_traditional(),
            board: TraditionalBoardGraph::new(),
            game_over_state: None
        })
        .insert_resource(SelectedTile::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_egui_ui,
            handle_tile_click,
            spawn_move_indicators,
            update_piece_labels,
            update_turn_indicator,
            make_cpu_moves,
        ))
        .run();
}

fn setup(mut commands: Commands, mut graph_state: ResMut<GraphState>, tile_query: Query<Entity, With<Tile<1>>>, edge_query: Query<Entity, With<GraphEdge>>, game: Res<Game>) {
    commands.spawn(Camera2dBundle::default());

    despawn_all_graph_entities(&mut commands, tile_query, edge_query);

    let player_type = match game.are_players_cpu[0] {
        true => "CPU",
        false => "Human"
    };

    let turn_text = commands.spawn(Text2dBundle {
        text: Text::from_section(
            format!("White ({}) to move", player_type),
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ),
        transform: Transform::from_translation(Vec3::new(0.0, 350.0, 0.5)),
        ..default()
    }).id();
    commands.insert_resource(CurrentTurnLabel(turn_text));

    spawn_traditional_graph(&mut commands, &mut graph_state, game);
}

fn despawn_all_graph_entities(
    commands: &mut Commands,
    tile_query: Query<Entity, With<Tile<1>>>,
    edge_query: Query<Entity, With<GraphEdge>>,
) {
    for entity in tile_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in edge_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn make_cpu_moves(
    mut game: ResMut<Game>,
) {
    if game.game_over_state == None && game.are_players_cpu[game.current_position.active_player.as_idx()] {
        game.make_cpu_move()
    }
}

fn handle_tile_click(
    mut event_reader: EventReader<Pointer<Click>>,
    tile_query: Query<&Tile<1>>,
    mut selected_tile: ResMut<SelectedTile>,
    mut game: ResMut<Game>,
) {
    for event in event_reader.read() {
        // TODO: Make this run less? It keeps looping
        if game.are_players_cpu[game.current_position.active_player.as_idx()] { 
            return // No clicks will register while the AI is thinking
        }

        if let Ok(clicked_tile) = tile_query.get(event.target) {
            // Assume the clicked tile should be selected if it has an occupant
            let original_selected_tile = selected_tile.tile_index;
            if clicked_tile.occupant != None {
                selected_tile.entity = Some(event.target);
                selected_tile.tile_index = Some(clicked_tile.id);
            }
            // Attempt to make a move if a different tile is already selected
            if let Some(source_tile) = original_selected_tile {
                let moves = game.query_tile(&source_tile);
                if moves.get_bit_at_tile(&clicked_tile.id) {
                    match game.attempt_move_input(&source_tile, &clicked_tile.id) {
                        Err(_) => {}, // TODO: Add code to display the error here
                        _ => { // Successful moves reset selected_tile
                            selected_tile.entity = None;
                            selected_tile.tile_index = None;
                        }
                    }
                }
            }
        }
    }
}

fn spawn_move_indicators(
    mut commands: Commands,
    selected_tile: Res<SelectedTile>,
    mut game: ResMut<Game>,
    tile_query: Query<(&Tile<1>, Entity)>,
    indicator_query: Query<Entity, With<MoveIndicator>>,
) {
    for indicator in indicator_query.iter() {
        commands.entity(indicator).despawn_recursive();
    }

    if let Some(tile_index) = selected_tile.tile_index {
        let moves = game.query_tile(&tile_index);

        for (tile, entity) in tile_query.iter() {
            // TODO: More efficient way to write this that only queries tiles in the moves (removing this check)
            if moves.get_bit_at_tile(&tile.id) {
                let mut bundle = PickableBundle::default(); // Needed to add this to get the right behavior
                bundle.pickable.should_block_lower = false;
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        MoveIndicator,
                        bundle,
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgba(0.0, 0.0, 0.0, 0.5),
                                custom_size: Some(Vec2::new(30.0, 30.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                            ..default()
                        },
                    ));
                });
            } else if tile.id == tile_index {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        MoveIndicator,
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgba(0.996, 0.996, 0.196, 0.5),
                                custom_size: Some(Vec2::new(85.0, 85.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                            ..default()
                        },
                    ));
                });
            }
        }
    }
}

fn update_piece_labels(
    game: Res<Game>,
    mut tile_query: Query<(&mut Tile<1>, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    if game.is_changed() {
        for (mut tile, children) in tile_query.iter_mut() {
            tile.occupant = game.current_position.get_occupant(&tile.id);

            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    let mut new_char = ' ';
                    let mut new_color = Color::BLACK;
                    if let Some(occupant) = tile.occupant {
                        new_char = occupant.display();
                        new_color = match occupant.color {
                            piece_set::Color::White => Color::WHITE,
                            piece_set::Color::Black => Color::BLACK
                        }
                    }
                    text.sections[0].value = new_char.to_string();
                    text.sections[0].style.color = new_color;
                }
            }
        }
    }
}

fn update_turn_indicator(
    mut game: ResMut<Game>,
    current_turn_res: Res<CurrentTurnLabel>,
    mut text_query: Query<&mut Text>,
) {
    if game.is_changed() {
        if let Ok(mut text) = text_query.get_mut(current_turn_res.0) {
            let active_player = game.current_position.active_player;
            let player_name = if active_player == piece_set::Color::White { "White" } else { "Black" };
            let player_type = match game.are_players_cpu[active_player.as_idx()] {
                true => "CPU",
                false => "Human"
            };
            game.check_if_over();
            if let Some(game_over_condition) = &game.game_over_state {
                text.sections[0].value = game_over_condition.display(game.current_position.active_player.opponent());
            } else {
                text.sections[0].value = format!("{} ({}) to move", player_name, player_type);
            }
        }
    }
}

fn spawn_traditional_graph(commands: &mut Commands, graph_state: &mut ResMut<GraphState>, game: Res<Game>) {
    let num_tiles = game.board.0.node_count() as u32;
    let num_edges = game.board.0.edge_count() as u32;
    let mut tiles: Vec<(Entity, Tile<1>)> = Vec::with_capacity(num_tiles as usize);

    for i in 0..num_tiles {
        let x = ((i % 8) as f32) * ((600 / 7) as f32);
        let y = ((i / 8) as f32) * ((600 / 7) as f32);
        let pos = Vec2::new(x - 300.0, y - 300.0);
        let tile_index = TileIndex::new(i as usize);
        let occupant = game.current_position.get_occupant(&tile_index);
        let mut occupant_char = ' ';
        if let Some(occ) = occupant {
            occupant_char = occ.display();
        }

        let graph_tile_component = Tile { id: TileIndex::new(i as usize), occupant, orientation: LimitedInt::<1>::new(1), pawn_start: None };

        let color = match (i + (i / 8)) % 2 {
            0 => Color::rgb(0.46, 0.58, 0.33),
            _ => Color::rgb(0.92, 0.92, 0.81)
        };

        let tile_entity = commands.spawn((
            graph_tile_component,
            SpriteBundle {
                sprite: Sprite {
                    color: color,
                    custom_size: Some(Vec2::new(85.0, 85.0)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                ..default()
            },
            PickableBundle::default(),
        )).with_children(|parent| {
            parent.spawn(Text2dBundle {
                text: Text::from_section(
                    occupant_char.to_string(),
                    TextStyle {
                        font_size: 50.0,
                        color: Color::BLACK,
                        ..default()
                    }
                ),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.5)),
                ..default()
            });
        })
        .id();
        tiles.push((tile_entity, graph_tile_component));
    }

    graph_state.tile_count = num_tiles;
    graph_state.edge_count = num_edges;
}

fn handle_egui_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    graph_state: ResMut<GraphState>,
    tile_query: Query<Entity, With<Tile<1>>>,
    edge_query: Query<Entity, With<GraphEdge>>,
) {
    egui::Window::new("Graph Controls")
        .default_pos(egui::pos2(10.0, 10.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Graph Information");
            ui.label(format!("Tiles: {}", graph_state.tile_count));
            ui.label(format!("Edges: {}", graph_state.edge_count));
            ui.separator();
            if ui.button("Delete Graph").clicked() {
                despawn_all_graph_entities(&mut commands, tile_query, edge_query);
            }
        });
}

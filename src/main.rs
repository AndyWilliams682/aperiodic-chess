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

use graph_boards::traditional_board::TraditionalBoardGraph;
use graph_boards::hexagonal_board::HexagonalBoardGraph;
use position::Position;
use graph_boards::graph_board::TileIndex;

use crate::{engine::Engine, game::Game, graph_boards::graph_board::Tile, limited_int::LimitedInt};

// fn main() {
//     let board = TraditionalBoardGraph::new();
//     let move_tables = board.0.move_tables();
//     let position = Position::new_traditional();

//     let mut game = Game {
//         engine: Engine::new(move_tables),
//         are_players_cpu: vec![false, true],
//         current_position: position,
//         board
//     };

//     game.play_game();
// }

// Trying bevy
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_mod_picking::prelude::*;

// --- Components and Resources ---

/// A component that identifies an edge in our graph.
#[derive(Component, Debug, Clone, Copy)]
pub struct GraphEdge {
    pub start_node_id: u32,
    pub end_node_id: u32,
}

// A component to mark the visual indicators for possible moves
#[derive(Component)]
struct MoveIndicator;

/// A resource to hold the global state of our graph.
#[derive(Resource, Default)]
struct GraphState {
    node_count: u32,
    edge_count: u32,
}

// A new resource to hold the chess game state
#[derive(Resource)]
struct ChessGame(Game);

// A resource to hold the entity of the turn indicator text
#[derive(Resource)]
struct CurrentTurn(Entity);

#[derive(Resource, Default)]
struct GameHasFinished(bool);


// A resource to track the currently selected node for move visualization
#[derive(Resource, Default)]
struct SelectedNode {
    entity: Option<Entity>,
    tile_index: Option<TileIndex>,
}

// --- Plugins and Setup ---

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            DefaultPickingPlugins,
        ))
        .insert_resource(GraphState::default())
        // Insert the ChessGame resource here so it's available to all systems
        .insert_resource(ChessGame(Game {
            engine: Engine::new(TraditionalBoardGraph::new().0.move_tables()),
            are_players_cpu: vec![false, true],
            current_position: Position::new_traditional(),
            board: TraditionalBoardGraph::new()
        }))
        .insert_resource(SelectedNode::default())
        .insert_resource(GameHasFinished::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_egui_ui,
            handle_node_click,
            spawn_move_indicators,
            update_piece_labels,
            update_turn_indicator,
            make_cpu_moves,
        ))
        .run();
}

fn setup(mut commands: Commands, mut graph_state: ResMut<GraphState>, node_query: Query<Entity, With<Tile<1>>>, edge_query: Query<Entity, With<GraphEdge>>, chess_game: Res<ChessGame>) {
    // Spawn a camera to view the scene.
    commands.spawn(Camera2dBundle::default());

    // Despawn all entities before generating the first graph.
    despawn_all_graph_entities(&mut commands, node_query, edge_query);

    let player_type = match chess_game.0.are_players_cpu[0] {
        true => "CPU",
        false => "Human"
    };

    // Spawn the turn indicator text
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
    commands.insert_resource(CurrentTurn(turn_text));

    spawn_traditional_graph(&mut commands, &mut graph_state, chess_game);
}

// --- Systems ---

/// Helper function to despawn all nodes and edges.
fn despawn_all_graph_entities(
    commands: &mut Commands,
    node_query: Query<Entity, With<Tile<1>>>,
    edge_query: Query<Entity, With<GraphEdge>>,
) {
    for entity in node_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in edge_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn make_cpu_moves(
    mut chess_game: ResMut<ChessGame>,
    finished: Res<GameHasFinished>,
) {
    if !finished.0 && chess_game.0.are_players_cpu[chess_game.0.current_position.active_player.as_idx()] {
        chess_game.0.make_cpu_move()
    }
}

// A new system to handle node clicks. It takes `EventReader`, `Query`, and `ResMut` as parameters.
fn handle_node_click(
    mut event_reader: EventReader<Pointer<Click>>,
    node_query: Query<&Tile<1>>,
    mut selected_node: ResMut<SelectedNode>,
    mut chess_game: ResMut<ChessGame>,
) {
    for event in event_reader.read() {
        // Handle selecting a new piece
        // TODO: Make this run less? It keeps looping
        if chess_game.0.are_players_cpu[chess_game.0.current_position.active_player.as_idx()] { 
            return
        }

        if let Ok(node) = node_query.get(event.target) {
            if let Some(source_tile) = selected_node.tile_index {
                let moves = chess_game.0.query_tile(&source_tile);
                if moves.get_bit_at_tile(&node.id) {
                    selected_node.entity = None;
                    selected_node.tile_index = None;
                    // TODO: Rewrite with a better pattern; clean this trash up
                    // Should be 1) assume clicked node is selected
                    // 2) if move is possible, then reset selected_node instead
                    // TODO: Actually handle errors and notify users
                    match chess_game.0.attempt_move_input(&source_tile, &node.id) {
                        Err(_) => {
                            // TODO: This is where unplayable moves due to legality should be handled
                            if node.occupant != None { // TODO: Make function to reduce repeat code
                                selected_node.entity = Some(event.target);
                                selected_node.tile_index = Some(node.id); 
                            } else {
                                selected_node.entity = None;
                                selected_node.tile_index = None;
                            }
                        },
                        _ => {}
                    }
                } else if node.occupant != None {
                    selected_node.entity = Some(event.target);
                    selected_node.tile_index = Some(node.id);
                }
            } else if node.occupant != None {
                // Select this node if a piece is present
                selected_node.entity = Some(event.target);
                selected_node.tile_index = Some(node.id);
            }
        }
    }
}

// A new system to spawn and despawn visual indicators for possible moves.
fn spawn_move_indicators(
    mut commands: Commands,
    selected_node: Res<SelectedNode>,
    mut chess_game: ResMut<ChessGame>,
    node_query: Query<(&Tile<1>, Entity)>,
    indicator_query: Query<Entity, With<MoveIndicator>>,
) {
    // If a node is selected, spawn indicators for its valid moves
    // TODO: When move indicators spawn, it breaks if piece at promotion tile
    if let Some(tile_index) = selected_node.tile_index {

        // TODO: Make this not loop?
        // Despawn all existing indicators first
        for indicator in indicator_query.iter() {
            commands.entity(indicator).despawn_recursive();
        }

        let moves = chess_game.0.query_tile(&tile_index);

        for (node, entity) in node_query.iter() {
            // TODO: More efficient way to write this that only queries nodes in the moves (removing this check)
            if moves.get_bit_at_tile(&node.id) {
                // Spawn a small circle as a child of the destination node
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
            } else if node.id == tile_index {
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
    } else {
        // TODO: Make this not loop?
        // Despawn all existing indicators first
        // TODO: Two of this code
        for indicator in indicator_query.iter() {
            commands.entity(indicator).despawn_recursive();
        }
    }
}

// A new system to update the character labels on the nodes based on the game state.
fn update_piece_labels(
    chess_game: Res<ChessGame>,
    mut node_query: Query<(&mut Tile<1>, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    // This system only runs when the ChessGame resource has been changed.
    if chess_game.is_changed() {
        for (mut node, children) in node_query.iter_mut() {
            node.occupant = chess_game.0.current_position.get_occupant(&node.id);

            for &child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    let mut new_char = ' ';
                    let mut new_color = Color::BLACK;
                    if let Some(occupant) = node.occupant {
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
    mut chess_game: ResMut<ChessGame>,
    current_turn_res: Res<CurrentTurn>,
    mut text_query: Query<&mut Text>,
    mut finished: ResMut<GameHasFinished>,
) {
    if chess_game.is_changed() {
        if let Ok(mut text) = text_query.get_mut(current_turn_res.0) {
            let active_player = chess_game.0.current_position.active_player;
            let player_name = if active_player == piece_set::Color::White { "White" } else { "Black" };
            let player_type = match chess_game.0.are_players_cpu[active_player.as_idx()] {
                true => "CPU",
                false => "Human"
            };
            if let Some(game_over_condition) = chess_game.0.is_over() {
                text.sections[0].value = game_over_condition.display(chess_game.0.current_position.active_player.opponent());
                finished.0 = true;
            } else {
                text.sections[0].value = format!("{} ({}) to move", player_name, player_type);
            }
        }
    }
}

fn spawn_traditional_graph(commands: &mut Commands, graph_state: &mut ResMut<GraphState>, chess_game_res: Res<ChessGame>) {
    // Get the ChessGame resource to access the position.
    let num_nodes = chess_game_res.0.board.0.node_count() as u32;
    let num_edges = chess_game_res.0.board.0.edge_count() as u32;
    let mut nodes: Vec<(Entity, Tile<1>)> = Vec::with_capacity(num_nodes as usize);

    for i in 0..num_nodes {
        let x = ((i % 8) as f32) * ((600 / 7) as f32);
        let y = ((i / 8) as f32) * ((600 / 7) as f32);
        let pos = Vec2::new(x - 300.0, y - 300.0);
        let tile_index = TileIndex::new(i as usize);
        let occupant = chess_game_res.0.current_position.get_occupant(&tile_index);
        let mut occupant_char = ' ';
        if let Some(occ) = occupant {
            occupant_char = occ.display();
        }

        let graph_node_component = Tile { id: TileIndex::new(i as usize), occupant, orientation: LimitedInt::<1>::new(1), pawn_start: None };

        let color = match (i + (i / 8)) % 2 {
            0 => Color::rgb(0.46, 0.58, 0.33),
            _ => Color::rgb(0.92, 0.92, 0.81)
        };

        // A node is an entity with a sprite and our custom `GraphNode` component.
        let node_entity = commands.spawn((
            graph_node_component,
            SpriteBundle {
                sprite: Sprite {
                    color: color,
                    custom_size: Some(Vec2::new(85.0, 85.0)),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                ..default()
            },
            // The `bevy_mod_picking` components are essential for interaction.
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
        nodes.push((node_entity, graph_node_component));
    }

    graph_state.node_count = num_nodes;
    graph_state.edge_count = num_edges;
}

/// A system to render the Egui UI.
fn handle_egui_ui(
    mut contexts: EguiContexts,
    mut commands: Commands,
    graph_state: ResMut<GraphState>,
    node_query: Query<Entity, With<Tile<1>>>,
    edge_query: Query<Entity, With<GraphEdge>>,
) {
    egui::Window::new("Graph Controls")
        .default_pos(egui::pos2(10.0, 10.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Graph Information");
            ui.label(format!("Nodes: {}", graph_state.node_count));
            ui.label(format!("Edges: {}", graph_state.edge_count));
            ui.separator();
            if ui.button("Delete Graph").clicked() {
                despawn_all_graph_entities(&mut commands, node_query, edge_query);
            }
        });
}

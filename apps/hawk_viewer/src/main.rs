use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use hawk_core::{Edge, Graph, Node, NodeKind};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct HawkGraph {
    graph: Graph,
}

#[derive(Resource, Default)]
struct ViewerState {
    selected_node: Option<String>,
    search_query: String,
    layers: LayerToggles,
    highlight_edges: HashSet<usize>,
}

#[derive(Default)]
struct LayerToggles {
    event: bool,
    compute: bool,
    storage: bool,
    orchestration: bool,
}

impl LayerToggles {
    fn all_on() -> Self {
        Self {
            event: true,
            compute: true,
            storage: true,
            orchestration: true,
        }
    }

    fn is_visible(&self, kind: &NodeKind) -> bool {
        match kind {
            NodeKind::Lambda | NodeKind::EcsService | NodeKind::Ec2Instance => self.compute,
            NodeKind::EventRule
            | NodeKind::ApiGateway
            | NodeKind::ApiRoute
            | NodeKind::SnsTopic
            | NodeKind::SqsQueue
            | NodeKind::LogGroup => self.event,
            NodeKind::S3Bucket | NodeKind::DynamoStream => self.storage,
            NodeKind::StepFunction => self.orchestration,
            _ => true,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct GraphNode {
    id: String,
    kind: NodeKind,
    name: String,
}

#[derive(Component)]
struct NodeLabel;

#[derive(Component)]
struct EdgeLine {
    index: usize,
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("hawk.json");

    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to read {path}: {e}");
            std::process::exit(1);
        }
    };
    let graph: Graph = match serde_json::from_str(&data) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to parse {path}: {e}");
            std::process::exit(1);
        }
    };

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hawk Viewer".to_string(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(HawkGraph { graph })
        .insert_resource(ViewerState {
            layers: LayerToggles::all_on(),
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (ui_system, click_system, visibility_system))
        .run();
}

// ---------------------------------------------------------------------------
// Setup: spawn camera + node sprites + edge lines
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    hawk: Res<HawkGraph>,
) {
    commands.spawn(Camera2d);

    let positions = compute_layout(&hawk.graph);

    // Spawn node sprites
    for node in &hawk.graph.nodes {
        let pos = positions.get(&node.id).copied().unwrap_or(Vec2::ZERO);
        let color = color_for_kind(&node.kind);

        commands
            .spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
                Transform::from_translation(pos.extend(1.0)),
                GraphNode {
                    id: node.id.clone(),
                    kind: node.kind.clone(),
                    name: node.name.clone(),
                },
            ));

        // Label
        commands.spawn((
            Text2d::new(truncate(&node.name, 20)),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation((pos + Vec2::new(0.0, -18.0)).extend(2.0)),
            NodeLabel,
        ));
    }

    // Spawn edge lines as thin sprites
    for (i, edge) in hawk.graph.edges.iter().enumerate() {
        let from = positions.get(&edge.from).copied().unwrap_or(Vec2::ZERO);
        let to = positions.get(&edge.to).copied().unwrap_or(Vec2::ZERO);
        let mid = (from + to) / 2.0;
        let diff = to - from;
        let len = diff.length().max(1.0);
        let angle = diff.y.atan2(diff.x);

        commands.spawn((
            Sprite {
                color: Color::srgba(0.6, 0.6, 0.6, 0.5),
                custom_size: Some(Vec2::new(len, 1.5)),
                ..default()
            },
            Transform::from_translation(mid.extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            EdgeLine { index: i },
        ));
    }
}

// ---------------------------------------------------------------------------
// Layout: group nodes by kind into vertical bands
// ---------------------------------------------------------------------------

fn compute_layout(graph: &Graph) -> HashMap<String, Vec2> {
    let mut positions = HashMap::new();

    // Group nodes by band
    let mut bands: HashMap<&str, Vec<&Node>> = HashMap::new();
    for node in &graph.nodes {
        let band = match node.kind {
            NodeKind::EventRule
            | NodeKind::ApiGateway
            | NodeKind::ApiRoute
            | NodeKind::SnsTopic
            | NodeKind::SqsQueue
            | NodeKind::LogGroup => "triggers",
            NodeKind::Lambda | NodeKind::EcsService | NodeKind::Ec2Instance => "compute",
            NodeKind::S3Bucket | NodeKind::DynamoStream => "storage",
            NodeKind::StepFunction => "orchestration",
            _ => "other",
        };
        bands.entry(band).or_default().push(node);
    }

    let band_order = ["triggers", "orchestration", "compute", "storage", "other"];
    let band_spacing = 300.0;
    let node_spacing = 60.0;

    for (bi, band_name) in band_order.iter().enumerate() {
        if let Some(nodes) = bands.get(band_name) {
            let x = (bi as f32 - 2.0) * band_spacing;
            let total_height = (nodes.len() as f32 - 1.0) * node_spacing;
            let start_y = total_height / 2.0;
            for (ni, node) in nodes.iter().enumerate() {
                let y = start_y - ni as f32 * node_spacing;
                positions.insert(node.id.clone(), Vec2::new(x, y));
            }
        }
    }

    positions
}

fn color_for_kind(kind: &NodeKind) -> Color {
    match kind {
        NodeKind::Lambda => Color::srgb(1.0, 0.6, 0.0),
        NodeKind::ApiGateway | NodeKind::ApiRoute => Color::srgb(0.3, 0.7, 1.0),
        NodeKind::EventRule => Color::srgb(0.9, 0.3, 0.3),
        NodeKind::SqsQueue => Color::srgb(0.4, 0.8, 0.4),
        NodeKind::SnsTopic => Color::srgb(0.8, 0.4, 0.8),
        NodeKind::S3Bucket => Color::srgb(0.3, 0.8, 0.3),
        NodeKind::StepFunction => Color::srgb(0.5, 0.5, 1.0),
        NodeKind::LogGroup => Color::srgb(0.6, 0.6, 0.6),
        NodeKind::DynamoStream => Color::srgb(0.2, 0.5, 0.9),
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

// ---------------------------------------------------------------------------
// UI system (egui panels)
// ---------------------------------------------------------------------------

fn ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<ViewerState>,
    hawk: Res<HawkGraph>,
) {
    // Left panel: search + filters + layers
    egui::SidePanel::left("left_panel")
        .default_width(220.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Hawk Viewer");
            ui.separator();

            ui.label("Search:");
            ui.text_edit_singleline(&mut state.search_query);
            ui.separator();

            ui.label("Layers:");
            ui.checkbox(&mut state.layers.compute, "Compute (Lambda, ECS, EC2)");
            ui.checkbox(&mut state.layers.event, "Events (EB, API, SNS, SQS, Logs)");
            ui.checkbox(&mut state.layers.storage, "Storage (S3, DynamoDB)");
            ui.checkbox(&mut state.layers.orchestration, "Orchestration (Step Functions)");
            ui.separator();

            ui.label(format!("Nodes: {}", hawk.graph.stats.node_count));
            ui.label(format!("Edges: {}", hawk.graph.stats.edge_count));
        });

    // Right panel: selected node details
    egui::SidePanel::right("right_panel")
        .default_width(280.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Node Details");
            ui.separator();

            if let Some(ref selected_id) = state.selected_node {
                if let Some(node) = hawk.graph.nodes.iter().find(|n| &n.id == selected_id) {
                    ui.label(format!("Name: {}", node.name));
                    ui.label(format!("Kind: {:?}", node.kind));
                    if let Some(ref arn) = node.arn {
                        ui.label(format!("ARN: {arn}"));
                    }
                    if let Some(ref region) = node.region {
                        ui.label(format!("Region: {region}"));
                    }
                    ui.separator();
                    ui.label("Properties:");
                    let props_str =
                        serde_json::to_string_pretty(&node.props).unwrap_or_default();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.monospace(&props_str);
                    });
                }
            } else {
                ui.label("Click a node to see details.");
            }
        });
}

// ---------------------------------------------------------------------------
// Click system: select nodes
// ---------------------------------------------------------------------------

fn click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    nodes: Query<(&GraphNode, &Transform)>,
    mut state: ResMut<ViewerState>,
    hawk: Res<HawkGraph>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };
    let (camera, cam_transform) = match cameras.get_single() {
        Ok(c) => c,
        Err(_) => return,
    };

    let cursor_pos = match window.cursor_position() {
        Some(p) => p,
        None => return,
    };

    let world_pos = match camera.viewport_to_world_2d(cam_transform, cursor_pos) {
        Ok(p) => p,
        Err(_) => return,
    };

    // Find closest node within 20 pixels
    let mut best: Option<(f32, String)> = None;
    for (gn, transform) in &nodes {
        let node_pos = transform.translation.truncate();
        let dist = world_pos.distance(node_pos);
        if dist < 20.0 {
            if best.as_ref().map_or(true, |(d, _)| dist < *d) {
                best = Some((dist, gn.id.clone()));
            }
        }
    }

    state.selected_node = best.map(|(_, id)| id.clone());

    // Highlight connected edges
    state.highlight_edges.clear();
    if let Some(ref sel) = state.selected_node {
        for (i, edge) in hawk.graph.edges.iter().enumerate() {
            if edge.from == *sel || edge.to == *sel {
                state.highlight_edges.insert(i);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Visibility system: hide/show based on layers + search, highlight edges
// ---------------------------------------------------------------------------

fn visibility_system(
    state: Res<ViewerState>,
    mut node_query: Query<(&GraphNode, &mut Visibility, &mut Sprite), Without<EdgeLine>>,
    mut edge_query: Query<(&EdgeLine, &mut Visibility, &mut Sprite), Without<GraphNode>>,
    hawk: Res<HawkGraph>,
) {
    let search_lower = state.search_query.to_lowercase();

    // Collect visible node IDs
    let mut visible_ids: HashSet<String> = HashSet::new();
    for (gn, mut vis, _sprite) in &mut node_query {
        let layer_visible = state.layers.is_visible(&gn.kind);
        let search_visible =
            search_lower.is_empty() || gn.name.to_lowercase().contains(&search_lower);

        if layer_visible && search_visible {
            *vis = Visibility::Visible;
            visible_ids.insert(gn.id.clone());
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // Update edge visibility and highlight
    for (edge_line, mut vis, mut sprite) in &mut edge_query {
        if let Some(edge) = hawk.graph.edges.get(edge_line.index) {
            if visible_ids.contains(&edge.from) && visible_ids.contains(&edge.to) {
                *vis = Visibility::Visible;
                if state.highlight_edges.contains(&edge_line.index) {
                    sprite.color = Color::srgb(1.0, 1.0, 0.0);
                    sprite.custom_size = Some(Vec2::new(
                        sprite.custom_size.map(|s| s.x).unwrap_or(100.0),
                        3.0,
                    ));
                } else {
                    sprite.color = Color::srgba(0.6, 0.6, 0.6, 0.5);
                    sprite.custom_size = Some(Vec2::new(
                        sprite.custom_size.map(|s| s.x).unwrap_or(100.0),
                        1.5,
                    ));
                }
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

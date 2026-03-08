use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use hawk_core::{Graph, Node, NodeKind};
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
    is_dragging: bool,
    last_cursor_pos: Option<Vec2>,
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

#[derive(Component)]
struct ArrowHead {
    index: usize,
}

#[derive(Component)]
struct NodeIcon;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NODE_SIZE: f32 = 32.0;
const LABEL_OFFSET_Y: f32 = -24.0;
const HIT_RADIUS: f32 = 24.0;
const ZOOM_SPEED: f32 = 0.1;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 5.0;
const BG_COLOR: Color = Color::srgb(0.06, 0.065, 0.09);

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
                title: format!(
                    "Hawk Viewer — {} nodes, {} edges",
                    graph.nodes.len(),
                    graph.edges.len()
                ),
                resolution: (1440., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(HawkGraph { graph })
        .insert_resource(ViewerState {
            layers: LayerToggles::all_on(),
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                ui_system,
                click_system,
                pan_system,
                zoom_system,
                visibility_system,
            ),
        )
        .run();
}

// ---------------------------------------------------------------------------
// Setup: spawn camera + node sprites + edge lines
// ---------------------------------------------------------------------------

fn setup(mut commands: Commands, hawk: Res<HawkGraph>) {
    commands.spawn(Camera2d);

    let positions = compute_layout(&hawk.graph);

    // Spawn edge lines first (lower z)
    for (i, edge) in hawk.graph.edges.iter().enumerate() {
        let from = positions.get(&edge.from).copied().unwrap_or(Vec2::ZERO);
        let to = positions.get(&edge.to).copied().unwrap_or(Vec2::ZERO);
        let mid = (from + to) / 2.0;
        let diff = to - from;
        let len = diff.length().max(1.0);
        let angle = diff.y.atan2(diff.x);

        // Edge line
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.45, 0.55, 0.4),
                custom_size: Some(Vec2::new(len, 1.5)),
                ..default()
            },
            Transform::from_translation(mid.extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            EdgeLine { index: i },
        ));

        // Arrowhead (small diamond near the target)
        let arrow_pos = from + diff * 0.85;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.45, 0.55, 0.5),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_translation(arrow_pos.extend(0.5))
                .with_rotation(Quat::from_rotation_z(angle)),
            ArrowHead { index: i },
        ));
    }

    // Spawn node sprites
    for node in &hawk.graph.nodes {
        let pos = positions.get(&node.id).copied().unwrap_or(Vec2::ZERO);
        let color = color_for_kind(&node.kind);

        // Node background (shadow/border effect)
        commands.spawn((
            Sprite {
                color: Color::srgba(0.1, 0.12, 0.16, 0.8),
                custom_size: Some(Vec2::new(NODE_SIZE + 4.0, NODE_SIZE + 4.0)),
                ..default()
            },
            Transform::from_translation(pos.extend(0.9)),
        ));

        // Node sprite (colored, shape varies by kind)
        let (w, h) = shape_for_kind(&node.kind);
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_translation(pos.extend(1.0)),
            GraphNode {
                id: node.id.clone(),
                kind: node.kind.clone(),
                name: node.name.clone(),
            },
        ));

        // Kind icon letter
        let icon = icon_for_kind(&node.kind);
        commands.spawn((
            Text2d::new(icon),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(pos.extend(1.5)),
            NodeIcon,
        ));

        // Label below node
        commands.spawn((
            Text2d::new(truncate(&node.name, 22)),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgba(0.75, 0.78, 0.85, 0.9)),
            Transform::from_translation((pos + Vec2::new(0.0, LABEL_OFFSET_Y)).extend(2.0)),
            NodeLabel,
        ));
    }
}

// ---------------------------------------------------------------------------
// Layout: band placement + force-directed refinement
// ---------------------------------------------------------------------------

fn compute_layout(graph: &Graph) -> HashMap<String, Vec2> {
    let mut positions = HashMap::new();

    let band_order = ["triggers", "orchestration", "compute", "storage", "other"];
    let band_spacing = 350.0;
    let node_spacing = 70.0;

    // Group nodes by band
    let mut bands: HashMap<&str, Vec<&Node>> = HashMap::new();
    for node in &graph.nodes {
        let band = band_for_kind(&node.kind);
        bands.entry(band).or_default().push(node);
    }

    // Initial band-based placement
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

    // Skip force simulation for very large graphs (> 500 nodes) to keep startup fast
    if graph.nodes.len() > 500 {
        return positions;
    }

    // Force-directed refinement
    let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let repulsion = 8000.0;
    let attraction = 0.005;
    let damping = 0.85;
    let iterations = 50;

    for _ in 0..iterations {
        let mut forces: HashMap<&str, Vec2> = HashMap::new();
        for id in &node_ids {
            forces.insert(id.as_str(), Vec2::ZERO);
        }

        // Repulsion between all pairs
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let pa = positions[&node_ids[i]];
                let pb = positions[&node_ids[j]];
                let diff = pa - pb;
                let dist = diff.length().max(1.0);
                let force = diff.normalize_or_zero() * repulsion / (dist * dist);
                *forces.get_mut(node_ids[i].as_str()).unwrap() += force;
                *forces.get_mut(node_ids[j].as_str()).unwrap() -= force;
            }
        }

        // Attraction along edges
        for edge in &graph.edges {
            if let (Some(&pa), Some(&pb)) = (positions.get(&edge.from), positions.get(&edge.to)) {
                let diff = pb - pa;
                let force = diff * attraction;
                if let Some(f) = forces.get_mut(edge.from.as_str()) {
                    *f += force;
                }
                if let Some(f) = forces.get_mut(edge.to.as_str()) {
                    *f -= force;
                }
            }
        }

        // Band constraint: gently pull nodes toward their band x
        for node in &graph.nodes {
            let band = band_for_kind(&node.kind);
            let band_idx = band_order.iter().position(|b| *b == band).unwrap_or(2);
            let target_x = (band_idx as f32 - 2.0) * band_spacing;
            if let Some(pos) = positions.get(&node.id) {
                let pull = Vec2::new((target_x - pos.x) * 0.05, 0.0);
                if let Some(f) = forces.get_mut(node.id.as_str()) {
                    *f += pull;
                }
            }
        }

        // Apply forces
        for id in &node_ids {
            if let Some(pos) = positions.get_mut(id) {
                let force = forces.get(id.as_str()).copied().unwrap_or(Vec2::ZERO);
                let clamped = Vec2::new(force.x.clamp(-50.0, 50.0), force.y.clamp(-50.0, 50.0));
                *pos += clamped * damping;
            }
        }
    }

    positions
}

fn band_for_kind(kind: &NodeKind) -> &'static str {
    match kind {
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
    }
}

fn color_for_kind(kind: &NodeKind) -> Color {
    match kind {
        NodeKind::Lambda => Color::srgb(1.0, 0.6, 0.0),
        NodeKind::ApiGateway | NodeKind::ApiRoute => Color::srgb(0.29, 0.56, 0.89),
        NodeKind::EventRule => Color::srgb(0.87, 0.30, 0.30),
        NodeKind::SqsQueue => Color::srgb(0.27, 0.73, 0.27),
        NodeKind::SnsTopic => Color::srgb(0.61, 0.35, 0.71),
        NodeKind::S3Bucket => Color::srgb(0.22, 0.68, 0.37),
        NodeKind::StepFunction => Color::srgb(0.35, 0.55, 0.90),
        NodeKind::LogGroup => Color::srgb(0.58, 0.65, 0.65),
        NodeKind::DynamoStream => Color::srgb(0.25, 0.47, 0.85),
        NodeKind::LoadBalancer => Color::srgb(0.85, 0.55, 0.25),
        NodeKind::EcsService => Color::srgb(0.95, 0.50, 0.15),
        NodeKind::Ec2Instance => Color::srgb(0.92, 0.75, 0.20),
        NodeKind::Unknown => Color::srgb(0.5, 0.5, 0.5),
    }
}

fn shape_for_kind(kind: &NodeKind) -> (f32, f32) {
    match kind {
        NodeKind::Lambda => (NODE_SIZE, NODE_SIZE),
        NodeKind::S3Bucket | NodeKind::DynamoStream => (NODE_SIZE + 4.0, NODE_SIZE - 4.0),
        NodeKind::StepFunction => (NODE_SIZE + 6.0, NODE_SIZE + 6.0),
        NodeKind::SqsQueue => (NODE_SIZE + 8.0, NODE_SIZE - 6.0),
        NodeKind::ApiGateway | NodeKind::ApiRoute => (NODE_SIZE - 2.0, NODE_SIZE + 2.0),
        _ => (NODE_SIZE, NODE_SIZE),
    }
}

fn icon_for_kind(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Lambda => "fn",
        NodeKind::ApiGateway | NodeKind::ApiRoute => "API",
        NodeKind::EventRule => "EB",
        NodeKind::SqsQueue => "Q",
        NodeKind::SnsTopic => "SNS",
        NodeKind::S3Bucket => "S3",
        NodeKind::StepFunction => "SF",
        NodeKind::LogGroup => "CW",
        NodeKind::DynamoStream => "DB",
        NodeKind::LoadBalancer => "LB",
        NodeKind::EcsService => "ECS",
        NodeKind::Ec2Instance => "EC2",
        NodeKind::Unknown => "?",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

// ---------------------------------------------------------------------------
// Pan system: right-click drag or middle-click drag
// ---------------------------------------------------------------------------

fn pan_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    mut state: ResMut<ViewerState>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.wants_pointer_input() {
        state.is_dragging = false;
        state.last_cursor_pos = None;
        return;
    }

    let window = match windows.get_single() {
        Ok(w) => w,
        Err(_) => return,
    };

    let dragging = buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle);

    if dragging {
        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(last_pos) = state.last_cursor_pos {
                let delta = cursor_pos - last_pos;
                if let Ok(mut cam_transform) = cameras.get_single_mut() {
                    let scale = cam_transform.scale.x;
                    cam_transform.translation.x -= delta.x * scale;
                    cam_transform.translation.y += delta.y * scale;
                }
            }
            state.last_cursor_pos = Some(cursor_pos);
            state.is_dragging = true;
        }
    } else {
        state.is_dragging = false;
        state.last_cursor_pos = None;
    }
}

// ---------------------------------------------------------------------------
// Zoom system: scroll wheel
// ---------------------------------------------------------------------------

fn zoom_system(
    mut scroll_events: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.wants_pointer_input() {
        return;
    }

    for event in scroll_events.read() {
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 100.0,
        };

        if let Ok(mut cam_transform) = cameras.get_single_mut() {
            let zoom_delta = -scroll_amount * ZOOM_SPEED;
            let new_scale = (cam_transform.scale.x + zoom_delta).clamp(MIN_ZOOM, MAX_ZOOM);
            cam_transform.scale = Vec3::splat(new_scale);
        }
    }
}

// ---------------------------------------------------------------------------
// UI system (egui panels)
// ---------------------------------------------------------------------------

fn ui_system(mut contexts: EguiContexts, mut state: ResMut<ViewerState>, hawk: Res<HawkGraph>) {
    // Left panel
    egui::SidePanel::left("left_panel")
        .default_width(240.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Hawk Viewer");
            ui.separator();

            ui.label("Search:");
            ui.text_edit_singleline(&mut state.search_query);
            ui.add_space(8.0);

            ui.separator();
            ui.label("Layers:");
            ui.checkbox(&mut state.layers.compute, "Compute (Lambda, ECS, EC2)");
            ui.checkbox(&mut state.layers.event, "Events (EB, API, SNS, SQS, Logs)");
            ui.checkbox(&mut state.layers.storage, "Storage (S3, DynamoDB)");
            ui.checkbox(
                &mut state.layers.orchestration,
                "Orchestration (Step Functions)",
            );
            ui.add_space(8.0);

            ui.separator();
            ui.label("Stats:");
            ui.label(format!("  Nodes: {}", hawk.graph.stats.node_count));
            ui.label(format!("  Edges: {}", hawk.graph.stats.edge_count));
            ui.add_space(8.0);

            ui.separator();
            ui.label("Legend:");
            legend_entry(ui, "fn  Lambda", [255, 153, 0]);
            legend_entry(ui, "EB  EventBridge", [222, 76, 76]);
            legend_entry(ui, "Q   SQS Queue", [68, 186, 68]);
            legend_entry(ui, "SNS Topic", [156, 89, 182]);
            legend_entry(ui, "S3  Bucket", [56, 174, 94]);
            legend_entry(ui, "SF  Step Function", [89, 140, 230]);
            legend_entry(ui, "API Gateway", [74, 143, 226]);
            legend_entry(ui, "CW  CloudWatch", [148, 166, 166]);
            legend_entry(ui, "DB  DynamoDB", [64, 120, 216]);
            ui.add_space(8.0);

            ui.separator();
            ui.label("Controls:");
            ui.label("  Left click: Select node");
            ui.label("  Right drag: Pan");
            ui.label("  Scroll: Zoom");
        });

    // Right panel
    egui::SidePanel::right("right_panel")
        .default_width(300.0)
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
                    if let Some(ref account_id) = node.account_id {
                        ui.label(format!("Account: {account_id}"));
                    }
                    ui.separator();

                    // Connected edges
                    let incoming: Vec<_> = hawk
                        .graph
                        .edges
                        .iter()
                        .filter(|e| e.to == *selected_id)
                        .collect();
                    let outgoing: Vec<_> = hawk
                        .graph
                        .edges
                        .iter()
                        .filter(|e| e.from == *selected_id)
                        .collect();

                    if !incoming.is_empty() {
                        ui.label(format!("Incoming ({}):", incoming.len()));
                        for edge in &incoming {
                            let name = hawk
                                .graph
                                .nodes
                                .iter()
                                .find(|n| n.id == edge.from)
                                .map(|n| n.name.as_str())
                                .unwrap_or("?");
                            ui.label(format!("  {:?} <- {}", edge.kind, name));
                        }
                        ui.add_space(4.0);
                    }

                    if !outgoing.is_empty() {
                        ui.label(format!("Outgoing ({}):", outgoing.len()));
                        for edge in &outgoing {
                            let name = hawk
                                .graph
                                .nodes
                                .iter()
                                .find(|n| n.id == edge.to)
                                .map(|n| n.name.as_str())
                                .unwrap_or("?");
                            ui.label(format!("  {:?} -> {}", edge.kind, name));
                        }
                        ui.add_space(4.0);
                    }

                    ui.separator();
                    ui.label("Properties:");
                    let props_str = serde_json::to_string_pretty(&node.props).unwrap_or_default();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.monospace(&props_str);
                    });
                }
            } else {
                ui.label("Click a node to see details.");
            }
        });
}

fn legend_entry(ui: &mut egui::Ui, label: &str, rgb: [u8; 3]) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 2.0, egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
        ui.label(label);
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
    mut contexts: EguiContexts,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() || ctx.wants_pointer_input() {
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

    // Find closest node within hit radius
    let mut best: Option<(f32, String)> = None;
    for (gn, transform) in &nodes {
        let node_pos = transform.translation.truncate();
        let dist = world_pos.distance(node_pos);
        if dist < HIT_RADIUS && best.as_ref().is_none_or(|(d, _)| dist < *d) {
            best = Some((dist, gn.id.clone()));
        }
    }

    let selected_id = best.map(|(_, id)| id);

    // Compute highlighted edges before assigning to state
    let mut highlight = HashSet::new();
    if let Some(ref sel) = selected_id {
        for (i, edge) in hawk.graph.edges.iter().enumerate() {
            if edge.from == *sel || edge.to == *sel {
                highlight.insert(i);
            }
        }
    }

    state.selected_node = selected_id;
    state.highlight_edges = highlight;
}

// ---------------------------------------------------------------------------
// Visibility system
// ---------------------------------------------------------------------------

fn visibility_system(
    state: Res<ViewerState>,
    mut node_query: Query<
        (&GraphNode, &mut Visibility, &mut Sprite),
        (Without<EdgeLine>, Without<ArrowHead>),
    >,
    mut edge_query: Query<
        (&EdgeLine, &mut Visibility, &mut Sprite),
        (Without<GraphNode>, Without<ArrowHead>),
    >,
    mut arrow_query: Query<
        (&ArrowHead, &mut Visibility, &mut Sprite),
        (Without<GraphNode>, Without<EdgeLine>),
    >,
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

    // Update edges
    for (edge_line, mut vis, mut sprite) in &mut edge_query {
        if let Some(edge) = hawk.graph.edges.get(edge_line.index) {
            if visible_ids.contains(&edge.from) && visible_ids.contains(&edge.to) {
                *vis = Visibility::Visible;
                if state.highlight_edges.contains(&edge_line.index) {
                    sprite.color = Color::srgba(0.48, 0.67, 0.97, 0.9);
                    sprite.custom_size = Some(Vec2::new(
                        sprite.custom_size.map(|s| s.x).unwrap_or(100.0),
                        3.0,
                    ));
                } else {
                    sprite.color = Color::srgba(0.4, 0.45, 0.55, 0.4);
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

    // Update arrowheads
    for (arrow, mut vis, mut sprite) in &mut arrow_query {
        if let Some(edge) = hawk.graph.edges.get(arrow.index) {
            if visible_ids.contains(&edge.from) && visible_ids.contains(&edge.to) {
                *vis = Visibility::Visible;
                if state.highlight_edges.contains(&arrow.index) {
                    sprite.color = Color::srgba(0.48, 0.67, 0.97, 0.9);
                } else {
                    sprite.color = Color::srgba(0.4, 0.45, 0.55, 0.5);
                }
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

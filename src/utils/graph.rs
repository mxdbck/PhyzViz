use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct GraphParams {
    /// Position on screen (top-left corner)
    pub position: Vec2,
    /// Size of the graph widget
    pub size: Vec2,
    /// Maximum number of data points to display
    pub max_points: usize,
    /// Color of the graph line
    pub line_color: Color,
    /// Color of gridlines
    pub grid_color: Color,
    /// Number of horizontal gridlines
    pub num_horizontal_lines: usize,
    /// Number of vertical gridlines
    pub num_vertical_lines: usize,
    /// Distance threshold from edge to trigger expansion (as fraction of range, e.g., 0.1 = 10%)
    pub expansion_threshold: f32,
    /// Minimum y-range to prevent division by zero
    pub min_y_range: f32,
    /// Label for the graph
    pub label: String,
}

impl Default for GraphParams {
    fn default() -> Self {
        Self {
            position: Vec2::new(-400.0, 300.0),
            size: Vec2::new(300.0, 200.0),
            max_points: 200,
            line_color: Color::linear_rgba(3.0, 0.6, 0.2, 1.0),
            grid_color: Color::srgba(0.5, 0.5, 0.5, 0.5),
            num_horizontal_lines: 3,
            num_vertical_lines: 3,
            expansion_threshold: 0.1,
            min_y_range: 0.1,
            label: "Graph".to_string(),
        }
    }
}

#[derive(Component)]
pub struct GraphWidget {
    pub params: GraphParams,
    /// Data points stored as (time, value)
    pub data: VecDeque<(f32, f32)>,
    /// Current axis ranges
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl GraphWidget {
    pub fn new(params: GraphParams) -> Self {
        Self {
            params,
            data: VecDeque::new(),
            x_min: 0.0,
            x_max: 10.0,
            y_min: -1.0,
            y_max: 1.0,
        }
    }

    /// Add a new data point (time, value)
    pub fn add_point(&mut self, time: f32, value: f32) {
        self.data.push_back((time, value));
        
        // Remove old points
        if self.data.len() > self.params.max_points {
            self.data.pop_front();
        }

        // Update axis ranges
        self.update_ranges();
    }

    fn update_ranges(&mut self) {
        if self.data.is_empty() {
            return;
        }

        // Get current data bounds
        let (mut data_x_min, mut data_x_max) = (f32::MAX, f32::MIN);
        let (mut data_y_min, mut data_y_max) = (f32::MAX, f32::MIN);

        for &(x, y) in &self.data {
            data_x_min = data_x_min.min(x);
            data_x_max = data_x_max.max(x);
            data_y_min = data_y_min.min(y);
            data_y_max = data_y_max.max(y);
        }

        // X-axis: sliding window (always show most recent data)
        self.x_max = data_x_max;
        self.x_min = data_x_min;

        // Y-axis: expand when data approaches boundaries
        let y_range = self.y_max - self.y_min;
        let threshold_distance = y_range * self.params.expansion_threshold;

        // Check if we need to expand upward
        if data_y_max > self.y_max - threshold_distance {
            self.y_max = data_y_max + threshold_distance;
        }

        // Check if we need to expand downward
        if data_y_min < self.y_min + threshold_distance {
            self.y_min = data_y_min - threshold_distance;
        }

        // Ensure minimum range
        if self.y_max - self.y_min < self.params.min_y_range {
            let center = (self.y_max + self.y_min) / 2.0;
            self.y_max = center + self.params.min_y_range / 2.0;
            self.y_min = center - self.params.min_y_range / 2.0;
        }
    }

    /// Convert data coordinates to screen coordinates
    fn to_screen(&self, x: f32, y: f32) -> Vec2 {
        let x_range = self.x_max - self.x_min;
        let y_range = self.y_max - self.y_min;

        let x_normalized = if x_range > 0.0 {
            (x - self.x_min) / x_range
        } else {
            0.5
        };
        let y_normalized = if y_range > 0.0 {
            (y - self.y_min) / y_range
        } else {
            0.5
        };

        Vec2::new(
            self.params.position.x + x_normalized * self.params.size.x,
            self.params.position.y - y_normalized * self.params.size.y,
        )
    }
}

/// System to draw the graph widget
pub fn draw_graph_widget(
    mut painter: ShapePainter,
    query: Query<&GraphWidget>,
) {
    for graph in query.iter() {
        draw_single_graph(&mut painter, graph);
    }
}

fn draw_single_graph(painter: &mut ShapePainter, graph: &GraphWidget) {
    let pos = graph.params.position;
    let size = graph.params.size;

    // Draw background rectangle
    painter.set_color(Color::srgba(0.1, 0.1, 0.1, 0.7));
    painter.thickness = 1.0;
    painter.hollow = true;
    painter.transform = Transform::from_translation(Vec3::new(pos.x + size.x / 2.0, pos.y - size.y / 2.0, 0.0));

    // painter.rect(Vec3::new(pos.x + size.x / 2.0, pos.y - size.y / 2.0, 0.0), Vec2::new(size.x, size.y));
    painter.rect(Vec2::new(size.x, size.y));

    painter.transform = Transform::IDENTITY;

    // Draw horizontal gridlines with labels
    painter.set_color(graph.params.grid_color);
    painter.thickness = 0.5;
    for i in 0..=graph.params.num_horizontal_lines {
        let t = i as f32 / graph.params.num_horizontal_lines as f32;
        let y_value = graph.y_min + t * (graph.y_max - graph.y_min);
        let screen_pos = graph.to_screen(graph.x_min, y_value);
        
        painter.line(
            Vec3::new(pos.x, screen_pos.y, 0.0),
            Vec3::new(pos.x + size.x, screen_pos.y, 0.0),
        );
    }

    // Draw vertical gridlines
    for i in 0..=graph.params.num_vertical_lines {
        let t = i as f32 / graph.params.num_vertical_lines as f32;
        let x_value = graph.x_min + t * (graph.x_max - graph.x_min);
        let screen_pos = graph.to_screen(x_value, graph.y_min);
        
        painter.line(
            Vec3::new(screen_pos.x, pos.y, 0.0),
            Vec3::new(screen_pos.x, pos.y - size.y, 0.0),
        );
    }

    // Draw the data line
    if graph.data.len() >= 2 {
        painter.set_color(graph.params.line_color);
        painter.thickness = 2.0;
        
        for i in 0..graph.data.len() - 1 {
            let (x1, y1) = graph.data[i];
            let (x2, y2) = graph.data[i + 1];
            
            let p1 = graph.to_screen(x1, y1);
            let p2 = graph.to_screen(x2, y2);
            
            painter.line(
                Vec3::new(p1.x, p1.y, 0.1),
                Vec3::new(p2.x, p2.y, 0.1),
            );
        }
    }
}

/// Spawn a graph widget entity
pub fn spawn_graph_widget(
    commands: &mut Commands,
    params: GraphParams,
) -> Entity {
    commands.spawn((
        GraphWidget::new(params),
        Name::new("GraphWidget"),
    )).id()
}

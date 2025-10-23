use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::asset::RenderAssetUsages;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct MeshRibbonParams {
    pub width: f32,
    pub max_points: usize,
    pub color: Color,
    pub fade_to_transparent: bool,
}

impl Default for MeshRibbonParams {
    fn default() -> Self {
        Self {
            width: 0.1,
            max_points: 100,
            color: Color::srgb(1.0, 0.3, 0.1),
            fade_to_transparent: true,
        }
    }
}

#[derive(Component)]
pub struct MeshRibbon {
    pub params: MeshRibbonParams,
    pub positions: VecDeque<Vec3>,
    pub mesh_handle: Handle<Mesh>,
    pub current_position: Vec3, // Track separately from Transform
}

/// Spawns a mesh-based ribbon entity
pub fn spawn_mesh_ribbon(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    name: String,
    params: MeshRibbonParams,
) -> Entity {
    let mesh = create_empty_ribbon_mesh();
    let mesh_handle = meshes.add(mesh);
    
    let material = materials.add(ColorMaterial {
        color: params.color,
        ..default()
    });

    commands.spawn((
        MeshRibbon {
            params: params.clone(),
            positions: VecDeque::with_capacity(params.max_points),
            mesh_handle: mesh_handle.clone(),
            current_position: Vec3::ZERO,
        },
        Mesh2d(mesh_handle),
        MeshMaterial2d(material),
        Transform::from_translation(Vec3::ZERO),
        Name::new(name),
    ))
    .id()
}

/// Creates an empty ribbon mesh
fn create_empty_ribbon_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

/// Updates the ribbon mesh based on its position history
pub fn update_ribbon_mesh(
    ribbon: &MeshRibbon,
    meshes: &mut Assets<Mesh>,
) {
    let positions = &ribbon.positions;
    if positions.len() < 2 {
        return;
    }

    let width = ribbon.params.width;
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices along the ribbon
    for (i, pos) in positions.iter().enumerate() {
        let progress = i as f32 / (positions.len() - 1) as f32;
        
        // Calculate perpendicular direction (in 2D, perpendicular to direction of motion)
        let tangent = if i < positions.len() - 1 {
            (positions[i + 1] - *pos).normalize_or_zero()
        } else if i > 0 {
            (*pos - positions[i - 1]).normalize_or_zero()
        } else {
            Vec3::X
        };

        // Get perpendicular vector (cross with up vector for 2D ribbons in XY plane)
        let perpendicular = Vec3::new(-tangent.y, tangent.x, 0.0).normalize_or_zero();
        if perpendicular.length_squared() < 0.01 {
            continue;
        }

        let half_width = width * 0.5;

        let left = (*pos + perpendicular * half_width * progress.powi(2));
        let right = (*pos - perpendicular * half_width * progress.powi(2));



        vertices.push([left.x, left.y, left.z]);
        vertices.push([right.x, right.y, right.z]);

        // Normals pointing toward camera (for 2D)
        normals.push([0.0, 0.0, 1.0]);
        normals.push([0.0, 0.0, 1.0]);

        // UVs
        uvs.push([0.0, progress]);
        uvs.push([1.0, progress]);

        // Colors with fade
        let alpha = if ribbon.params.fade_to_transparent {
            progress.powi(10) / 4.0
        } else {
            1.0 / 4.0
        };
        colors.push([1.0, 1.0, 1.0, alpha]);
        colors.push([1.0, 1.0, 1.0, alpha]);
    }

    // Generate indices for triangles
    for i in 0..(positions.len() - 1) {
        let base = (i * 2) as u32;
        // First triangle
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 1);
        // Second triangle
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    // Update the mesh
    if let Some(mesh) = meshes.get_mut(&ribbon.mesh_handle) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
    }
}

/// System to add new positions to ribbons
pub fn add_ribbon_position(
    mut query: Query<&mut MeshRibbon>,
    mut meshes: ResMut<Assets<Mesh>>,
    time_fixed: Res<Time<Fixed>>
) {
    if time_fixed.elapsed_secs() < 0.1 {
        return;
    }
    for mut ribbon in query.iter_mut() {
        let new_pos = ribbon.current_position;
        
        // Only add if position changed significantly
        if let Some(last_pos) = ribbon.positions.back() {
            if last_pos.distance(new_pos) < 0.001 {
                continue;
            }
        }

        ribbon.positions.push_back(new_pos);
        
        // Remove old positions
        if ribbon.positions.len() > ribbon.params.max_points {
            ribbon.positions.pop_front();
        }

        update_ribbon_mesh(&ribbon, &mut meshes);
    }
}

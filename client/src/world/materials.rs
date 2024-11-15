use crate::constants::{BASE_ROUGHNESS, BASE_SPECULAR_HIGHLIGHT};
use crate::world::GlobalMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, Face, TextureDimension, TextureFormat};
use shared::world::{BlockData, ItemData, Registry, RegistryId};
use std::collections::HashMap;

const TEXTURE_PATH: &str = "graphics/textures/";

#[derive(Resource, Default)]
pub struct MaterialResource {
    pub block_materials: HashMap<RegistryId, Handle<StandardMaterial>>,
    pub global_materials: HashMap<GlobalMaterial, Handle<StandardMaterial>>,
    pub item_textures: HashMap<RegistryId, Handle<Image>>,
    pub atlas_texture: Option<Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
pub struct AtlasHandles {
    pub handles: Vec<Handle<Image>>,
    pub loaded: bool,
}

pub fn setup_materials(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_resource: ResMut<MaterialResource>,
    mut atlas_handles: ResMut<AtlasHandles>,
    r_blocks: Res<Registry<BlockData>>,
    r_items: Res<Registry<ItemData>>,
) {
    let sun_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1., 0.95, 0.1),
        emissive: LinearRgba::new(1., 0.95, 0.1, 0.5),
        emissive_exposure_weight: 0.5,
        cull_mode: Some(Face::Front),
        ..Default::default()
    });

    let moon_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::WHITE,
        emissive_exposure_weight: 0.5,
        cull_mode: Some(Face::Front),
        ..Default::default()
    });

    // Root directory for asset server : /assets/
    // TODO : atlas textures (currently only supports 1 texture per cube, for all 6 faces)
    let grass_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(&(TEXTURE_PATH.to_owned() + "grass.png"))),
        base_color: Color::srgb(0.2, 0.85, 0.3),
        perceptual_roughness: BASE_ROUGHNESS,
        reflectance: BASE_SPECULAR_HIGHLIGHT,
        ..default()
    });
    // MC's grass texture is grey and tinted via a colormap according to biome
    // Don't have the knowledge to do that atm so used constant "grass green" color instead
    // Modifying color based on noise generation values could be interesting tho

    let dirt_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(&(TEXTURE_PATH.to_owned() + "dirt.png"))),
        perceptual_roughness: BASE_ROUGHNESS,
        reflectance: BASE_SPECULAR_HIGHLIGHT,
        ..default()
    });
    let stone_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(&(TEXTURE_PATH.to_owned() + "stone.png"))),
        perceptual_roughness: BASE_ROUGHNESS,
        reflectance: BASE_SPECULAR_HIGHLIGHT,
        ..default()
    });
    let bedrock_material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(&(TEXTURE_PATH.to_owned() + "bedrock.png"))),
        perceptual_roughness: BASE_ROUGHNESS,
        reflectance: BASE_SPECULAR_HIGHLIGHT,
        ..default()
    });

    material_resource
        .block_materials
        .insert(*r_blocks.get_id("dirt").unwrap(), dirt_material);
    material_resource
        .block_materials
        .insert(*r_blocks.get_id("grass").unwrap(), grass_material);
    material_resource
        .block_materials
        .insert(*r_blocks.get_id("stone").unwrap(), stone_material);
    material_resource
        .block_materials
        .insert(*r_blocks.get_id("bedrock").unwrap(), bedrock_material);

    material_resource
        .global_materials
        .insert(GlobalMaterial::Sun, sun_material);
    material_resource
        .global_materials
        .insert(GlobalMaterial::Moon, moon_material);

    material_resource.item_textures.insert(
        *r_items.get_id("grass").unwrap(),
        asset_server.load(&(TEXTURE_PATH.to_owned() + "grass.png")),
    );
    material_resource.item_textures.insert(
        *r_items.get_id("dirt").unwrap(),
        asset_server.load(&(TEXTURE_PATH.to_owned() + "dirt.png")),
    );
    material_resource.item_textures.insert(
        *r_items.get_id("stone").unwrap(),
        asset_server.load(&(TEXTURE_PATH.to_owned() + "stone.png")),
    );
    material_resource.item_textures.insert(
        *r_items.get_id("bedrock").unwrap(),
        asset_server.load(&(TEXTURE_PATH.to_owned() + "bedrock.png")),
    );

    let image_paths = ["moss.png", "dirt.png", "stone.png", "bedrock.png"];

    // Load the images asynchronously
    let handles: Vec<Handle<Image>> = image_paths
        .iter()
        .map(|filename| asset_server.load(&(TEXTURE_PATH.to_owned() + filename)))
        .collect();

    atlas_handles.handles = handles;
}

pub fn build_atlas(
    mut atlas_handles: ResMut<AtlasHandles>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_resource: ResMut<MaterialResource>,
) {
    if atlas_handles.loaded {
        // should just refactor to remove the system later
        return;
    }
    // Check if all images have been loaded
    let loaded_images: Vec<Image> = atlas_handles
        .handles
        .iter()
        .filter_map(|handle| images.get(handle))
        .cloned()
        .collect();

    if loaded_images.len() != atlas_handles.handles.len() {
        // Not all images are loaded yet
        return;
    }

    // Assuming each image is 16x16 and there are `n` images
    let image_count: usize = loaded_images.len();
    let atlas_width: u32 = (image_count * 16) as u32;
    let atlas_height: u32 = 16;
    let mut atlas_data: Vec<u8> = vec![0u8; (atlas_width * atlas_height * 4) as usize]; // RGBA format

    // Copy the pixels of each image into the correct position in the atlas
    for (i, image) in loaded_images.iter().enumerate() {
        let offset_x = i * 16;
        for y in 0..16 {
            for x in 0..16 {
                let src_index = (y * 16 + x) * 4;
                let dest_index = ((y * atlas_width as usize) + (x + offset_x)) * 4;
                atlas_data[dest_index..dest_index + 4]
                    .copy_from_slice(&image.data[src_index..src_index + 4]);
            }
        }
    }

    // Create the atlas texture
    let atlas_texture = Image::new_fill(
        Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    // Add the atlas texture as an asset
    let atlas_handle = images.add(atlas_texture);

    let atlas_material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_handle),
        perceptual_roughness: BASE_ROUGHNESS,
        reflectance: BASE_SPECULAR_HIGHLIGHT,
        ..default()
    });

    material_resource.atlas_texture = Some(atlas_material);

    atlas_handles.loaded = true;
}

use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use ndarray::Array2;
use noise::{NoiseFn, Perlin};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::utils::inv_lerp;

pub fn perlin_noise(app: &mut App) {
    app.init_resource::<NoiseConfig>()
        .register_type::<NoiseConfig>();
}

#[derive(Reflect, Resource, InspectorOptions, Clone)]
#[reflect(Resource, InspectorOptions)]
pub struct NoiseConfig {
    width: u16,
    height: u16,
    #[inspector(min = 0.0)]
    scale: f64,
    seed: u32,
    #[inspector(min = 0, max = 6)]
    level_of_detail: u16,
    octaves: usize,
    #[inspector(min = 0.0, max = 1.0)]
    persistance: f64,
    lacunarity: f64,
    offset: Vec2,
}

impl Default for NoiseConfig {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            scale: 50.0,
            seed: Default::default(),
            level_of_detail: 2,
            octaves: 4,
            persistance: 0.5,
            lacunarity: 2.0,
            offset: Default::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct NoiseMapGenerator {
    noise: Perlin,
    // noise_config: Option<&'a NoiseConfig>,
    offset: Vec2,
}

pub struct NoiseMap {
    // noise_config: &'a NoiseConfig,
    inner: Array2<f64>,
}

impl NoiseMapGenerator {
    pub fn new(config: &NoiseConfig) -> Self {
        Self {
            noise: Perlin::new(config.seed),
            offset: config.offset,
        }
    }

    pub fn with_offset(self, offset: Vec2) -> Self {
        Self { offset, ..self }
    }

    pub fn generate(&self, config: &NoiseConfig) -> NoiseMap {
        let mut noise_map = ndarray::Array2::zeros((config.width.into(), config.height.into()));

        let mut prng = StdRng::seed_from_u64(config.seed.into());
        let mut octave_offsets = vec![Vec2::ZERO; config.octaves];

        for i in octave_offsets.iter_mut() {
            let offset_x = prng.gen_range(-100_000.0..100_000.0) + self.offset.x;
            let offset_y = prng.gen_range(-100_000.0..100_000.0) + self.offset.y;
            *i = Vec2::new(offset_x, offset_y);
        }

        let mut max_noise_height = f64::MIN;
        let mut min_noise_height = f64::MAX;

        let half_width = (config.height / 2) as f64;
        let half_height = (config.width / 2) as f64;

        for y in 0..config.height {
            for x in 0..config.width {
                let mut amplitude = 1.0;
                let mut frequency = 1.0;
                let mut noise_height = 0.0;

                for i in &octave_offsets {
                    let sample_x =
                        (x as f64 - half_width) / config.scale * frequency + (i.x as f64);
                    let sample_y =
                        (y as f64 - half_height) / config.scale * frequency + (i.y as f64);

                    let perlin_value = self.noise.get([sample_x, sample_y]) * 2.0 - 1.0;
                    noise_height += perlin_value * amplitude;

                    amplitude *= config.persistance;
                    frequency *= config.lacunarity;
                }

                if noise_height > max_noise_height {
                    max_noise_height = noise_height;
                } else if noise_height < min_noise_height {
                    min_noise_height = noise_height;
                }
                noise_map[[x.into(), y.into()]] = noise_height;
            }
        }

        for y in 0..config.height {
            for x in 0..config.width {
                noise_map[[x.into(), y.into()]] = inv_lerp(
                    min_noise_height,
                    max_noise_height,
                    noise_map[[x.into(), y.into()]],
                )
            }
        }

        NoiseMap { inner: noise_map }
    }
}

impl NoiseMap {
    pub fn colorize_map(&self, config: &NoiseConfig) -> Vec<u8> {
        let mut colors = vec![[0.0, 0.0, 0.0]; (config.height * config.width).into()];

        for y in 0..config.height {
            for x in 0..config.width {
                let val = self.inner[[x.into(), y.into()]];
                let cl = crate::utils::color_lerp(Color::BLACK, Color::WHITE, val as f32);
                colors[(x + y * config.width) as usize] = {
                    let c = cl.as_rgba_f32();
                    [c[0], c[1], c[2]]
                };
            }
        }

        let data = bytemuck::cast_slice::<_, u8>(&colors);

        data.to_vec()
    }
}

use serde::Deserialize;

use crate::seed::WorldSeed;

#[derive(Debug, Clone, Deserialize)]
pub struct NoiseLayerConfig {
    pub name: String,
    pub frequency: f64,
    pub octaves: usize,
    pub persistence: f64,
    pub lacunarity: f64,
}

pub struct NoiseGenerator {
    seed: u32,
}

impl NoiseGenerator {
    pub fn new(seed: WorldSeed) -> Self {
        Self {
            seed: seed.0 as u32,
        }
    }

    pub fn generate_layer(
        &self,
        config: &NoiseLayerConfig,
        width: u32,
        height: u32,
        seed_offset: u32,
    ) -> Vec<f32> {
        use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};

        let fbm = Fbm::<OpenSimplex>::new(self.seed.wrapping_add(seed_offset))
            .set_octaves(config.octaves)
            .set_frequency(config.frequency)
            .set_persistence(config.persistence)
            .set_lacunarity(config.lacunarity);

        let mut result = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let val = fbm.get([x as f64, y as f64]);
                let normalized = ((val + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
                result.push(normalized);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> NoiseLayerConfig {
        NoiseLayerConfig {
            name: "test".into(),
            frequency: 0.01,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }

    #[test]
    fn seed_determinism() {
        let seed = WorldSeed(42);
        let noise_gen = NoiseGenerator::new(seed);
        let config = test_config();

        let map1 = noise_gen.generate_layer(&config, 10, 10, 0);
        let map2 = noise_gen.generate_layer(&config, 10, 10, 0);

        assert_eq!(map1, map2);
    }

    #[test]
    fn noise_range() {
        let seed = WorldSeed(42);
        let noise_gen = NoiseGenerator::new(seed);
        let config = test_config();

        let map = noise_gen.generate_layer(&config, 100, 100, 0);
        for val in &map {
            assert!(*val >= 0.0 && *val <= 1.0, "Noise value out of range: {val}");
        }
    }

    #[test]
    fn different_seeds_differ() {
        let noise1 = NoiseGenerator::new(WorldSeed(1));
        let noise2 = NoiseGenerator::new(WorldSeed(2));
        let config = test_config();

        let map1 = noise1.generate_layer(&config, 10, 10, 0);
        let map2 = noise2.generate_layer(&config, 10, 10, 0);

        assert_ne!(map1, map2);
    }
}

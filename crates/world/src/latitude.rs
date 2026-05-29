pub fn apply_latitude_modifier(
    temperature: &mut [f32],
    width: u32,
    height: u32,
    equator_weight: f32,
) {
    let equator = height as f32 / 2.0;
    let max_dist = equator;
    if max_dist == 0.0 {
        return;
    }
    for y in 0..height {
        let latitude_factor = 1.0 - ((y as f32 - equator).abs() / max_dist);
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let noise_val = temperature[idx];
            temperature[idx] = noise_val * (1.0 - equator_weight) + latitude_factor * equator_weight;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equator_hotter() {
        let width = 10u32;
        let height = 100u32;
        let mut temp = vec![0.5f32; (width * height) as usize];
        apply_latitude_modifier(&mut temp, width, height, 0.5);

        let equator_idx = (50 * width + 5) as usize;
        let pole_idx = (5 * width + 5) as usize;
        assert!(
            temp[equator_idx] > temp[pole_idx],
            "equator should be hotter than poles"
        );
    }

    #[test]
    fn test_symmetric() {
        let width = 10u32;
        let height = 100u32;
        let mut temp = vec![0.5f32; (width * height) as usize];
        apply_latitude_modifier(&mut temp, width, height, 0.5);

        let top = (40 * width + 5) as usize;
        let bottom = (60 * width + 5) as usize;
        assert!(
            (temp[top] - temp[bottom]).abs() < 0.01,
            "symmetric rows should have similar temperature"
        );
    }

    #[test]
    fn test_zero_weight() {
        let width = 10u32;
        let height = 10u32;
        let original = vec![0.7f32; (width * height) as usize];
        let mut temp = original.clone();
        apply_latitude_modifier(&mut temp, width, height, 0.0);
        assert_eq!(temp, original, "zero weight should not modify");
    }

    #[test]
    fn test_full_weight() {
        let width = 10u32;
        let height = 100u32;
        let mut temp = vec![0.0f32; (width * height) as usize];
        apply_latitude_modifier(&mut temp, width, height, 1.0);

        let equator_idx = (50 * width + 5) as usize;
        assert!(
            (temp[equator_idx] - 1.0).abs() < 0.01,
            "equator should be ~1.0 with full weight"
        );
    }
}

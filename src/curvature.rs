const R_EARTH: f64 = 6371000.0;
const R_EFF: f64 = R_EARTH * 1.13;

pub fn curvature_drop(t: f64, ray_length_in_meters: f64) -> f64 {
    let distance_from_start = t * ray_length_in_meters;
    let distance_from_end = (1.0 - t) * ray_length_in_meters;
    distance_from_start * distance_from_end * const { 0.5 / R_EFF }
}

#[cfg(test)]
#[test]
fn example_values() {
    assert_eq!(curvature_drop(0.0, 1_000.0), 0.0);
    assert_eq!(curvature_drop(1.0, 1_000.0), 0.0);
    assert!((curvature_drop(0.5, 1_000.0) - 0.04) <= 0.01);
    assert!((curvature_drop(0.5, 10_000.0) - 1.7) <= 0.05);
}

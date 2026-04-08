use crate::combined_pixel_traversal::CombinedPixelTraversal;
use crate::is_line_free;
use crate::matrix::{ArrayMatrix, Matrix};
use crate::ray_z::RayZ;
use image::{DynamicImage, Rgb};
use rand::distr::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::SeedableRng;
use rand_distr::Exp1;

#[test]
fn single_obstacle() {
    let mut matrix = ArrayMatrix::<f32>::new(3, 3);
    matrix.set(1, 1, 1.0);

    let ray = RayZ {
        start_x: 0.0,
        start_y: 0.0,
        start_z: 0.5,
        diff_x: 3.0,
        diff_y: 3.0,
        diff_z: 0.0,
    };
    let free = is_line_free(&matrix, ray);
    assert_eq!(free, false);

    let ray = RayZ {
        start_x: 0.0,
        start_y: 0.0,
        start_z: 0.5,
        diff_x: 3.0,
        diff_y: 1.0,
        diff_z: 0.0,
    };
    let free = is_line_free(&matrix, ray);
    assert_eq!(free, true);

    let ray = RayZ {
        start_x: 0.0,
        start_y: 0.0,
        start_z: 0.5,
        diff_x: 3.0,
        diff_y: 2.0,
        diff_z: 0.0,
    };
    let free = is_line_free(&matrix, ray);
    assert_eq!(free, false);

    let ray = RayZ {
        start_x: 0.0,
        start_y: 1.5,
        start_z: 2.0,
        diff_x: 3.0,
        diff_y: 0.0,
        diff_z: -2.0,
    };
    let free = is_line_free(&matrix, ray);
    assert_eq!(free, false);
}

#[test]
fn random() {
    let n_rays = 100;
    let x_size = 2000;
    let y_size = 2000;
    let z_size = 15;
    let x_distribution = Uniform::new(0.0, x_size as f32).unwrap();
    let y_distribution = Uniform::new(0.0, y_size as f32).unwrap();
    let z_distribution = Uniform::new(0.0, z_size as f32).unwrap();
    let height_distribution = Exp1;

    let mut rng = SmallRng::seed_from_u64(0);
    let rays: Vec<_> = (0..n_rays)
        .map(|_| {
            let start_x = x_distribution.sample(&mut rng);
            let start_y = y_distribution.sample(&mut rng);
            let start_z = z_distribution.sample(&mut rng);
            let end_x = x_distribution.sample(&mut rng);
            let end_y = y_distribution.sample(&mut rng);
            let end_z = z_distribution.sample(&mut rng);
            RayZ {
                start_x,
                start_y,
                start_z,
                diff_x: end_x - start_x,
                diff_y: end_y - start_y,
                diff_z: end_z - start_z,
            }
        })
        .collect();
    let mut matrix = ArrayMatrix::<f32>::random(x_size, y_size, height_distribution, &mut rng);

    // matrix.save_as_image(10.0, "out.png");

    /*let mut image = DynamicImage::ImageLuma8(matrix.as_image(10.0)).into_rgb8();

    for &ray_z in &rays {
        let ray = ray_z.as_ray();
        let mut still_free = true;
        for segment in CombinedPixelTraversal::new(ray) {
            let free = if ray_z.diff_z >= 0.0 {
                matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                    < (ray_z.start_z + segment.start_t * ray_z.diff_z)
            } else {
                matrix.get(segment.pixel_x as usize, segment.pixel_y as usize)
                    < ray_z.start_z + segment.end_t * ray_z.diff_z
            };
            still_free &= free;
            if still_free {
                image.put_pixel(
                    segment.pixel_x as u32,
                    segment.pixel_y as u32,
                    Rgb([0, 255, 0]),
                );
            } else {
                image.put_pixel(
                    segment.pixel_x as u32,
                    segment.pixel_y as u32,
                    Rgb([255, 0, 0]),
                );
            }
        }
    }

    image.save("out.png");*/

    let n_free = rays
        .iter()
        .filter(|&&ray| is_line_free(&matrix, ray))
        .count();
    println!("n_free = {n_free}");
}

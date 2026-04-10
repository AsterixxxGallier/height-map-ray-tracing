use super::*;
use crate::ray::Ray2;

#[test]
fn only_x_increasing() {
    let ray = Ray2 {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 4.0,
        diff_y: 0.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.25,
            pixel_x: 1,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.5,
            pixel_x: 2,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.75,
            pixel_x: 3,
            pixel_y: 0,
        })
    );
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_x_increasing_starts_out_of_bounds() {
    let ray = Ray2 {
        start_x: -1.0,
        start_y: 0.0,
        diff_x: 5.0,
        diff_y: 0.0,
    };

    let mut traversal = BoundaryTraversal::<f32>::new(ray);

    dbg!(&traversal);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.2,
            pixel_x: 0,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.4,
            pixel_x: 1,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.6,
            pixel_x: 2,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.8,
            pixel_x: 3,
            pixel_y: 0,
        })
    );
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_x_decreasing() {
    let ray = Ray2 {
        start_x: 4.0,
        start_y: 0.0,
        diff_x: -4.0,
        diff_y: 0.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.25,
            pixel_x: 2,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.5,
            pixel_x: 1,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.75,
            pixel_x: 0,
            pixel_y: 0,
        })
    );
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_y_increasing() {
    let ray = Ray2 {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 0.0,
        diff_y: 4.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::Y,
            t: 0.25,
            pixel_x: 0,
            pixel_y: 1,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::Y,
            t: 0.5,
            pixel_x: 0,
            pixel_y: 2,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::Y,
            t: 0.75,
            pixel_x: 0,
            pixel_y: 3,
        })
    );
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_y_decreasing() {
    let ray = Ray2 {
        start_x: 0.0,
        start_y: 4.0,
        diff_x: 0.0,
        diff_y: -4.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::Y,
            t: 0.25,
            pixel_x: 0,
            pixel_y: 2,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::Y,
            t: 0.5,
            pixel_x: 0,
            pixel_y: 1,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::Y,
            t: 0.75,
            pixel_x: 0,
            pixel_y: 0,
        })
    );
    assert_eq!(traversal.next(), None);
}

#[test]
fn perfectly_diagonal_increasing() {
    let ray = Ray2 {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 4.0,
        diff_y: 4.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.25,
            pixel_x: 1,
            pixel_y: 1,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.5,
            pixel_x: 2,
            pixel_y: 2,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.75,
            pixel_x: 3,
            pixel_y: 3,
        })
    );

    assert_eq!(traversal.next(), None);
}

#[test]
fn perfectly_diagonal_decreasing() {
    let ray = Ray2 {
        start_x: 4.0,
        start_y: 4.0,
        diff_x: -4.0,
        diff_y: -4.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.25,
            pixel_x: 2,
            pixel_y: 2,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.5,
            pixel_x: 1,
            pixel_y: 1,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.75,
            pixel_x: 0,
            pixel_y: 0,
        })
    );

    assert_eq!(traversal.next(), None);
}

#[test]
fn half_diagonal_increasing() {
    let ray = Ray2 {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 4.0,
        diff_y: 2.0,
    };

    let mut traversal = BoundaryTraversal::new(ray);

    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.25,
            pixel_x: 1,
            pixel_y: 0,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::XY,
            t: 0.5,
            pixel_x: 2,
            pixel_y: 1,
        })
    );
    assert_eq!(
        traversal.next(),
        Some(BoundaryCrossing {
            boundary_type: BoundaryType::X,
            t: 0.75,
            pixel_x: 3,
            pixel_y: 1,
        })
    );

    assert_eq!(traversal.next(), None);
}

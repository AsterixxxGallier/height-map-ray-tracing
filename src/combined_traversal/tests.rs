use crate::bounds::Bounds;
use crate::combined_traversal::{BoundaryCrossing, CombinedBoundaryTraversal};
use crate::ray::Ray;

#[test]
fn only_x_increasing() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 4.0,
        diff_y: 0.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.25,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.5,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.75,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_x_increasing_starts_out_of_bounds() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: -1.0,
        start_y: 0.0,
        diff_x: 5.0,
        diff_y: 0.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    dbg!(&traversal);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.2,
        last_x_index: -1,
        next_x_index: 0,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.4,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.6,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.8,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_x_decreasing() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: 4.0,
        start_y: 0.0,
        diff_x: -4.0,
        diff_y: 0.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.25,
        last_x_index: 3,
        next_x_index: 2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.5,
        last_x_index: 2,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.75,
        last_x_index: 1,
        next_x_index: 0,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_y_increasing() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 0.0,
        diff_y: 4.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.25,
        x_index: 0,
        last_y_index: 0,
        next_y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.5,
        x_index: 0,
        last_y_index: 1,
        next_y_index: 2,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.75,
        x_index: 0,
        last_y_index: 2,
        next_y_index: 3,
    }));
    assert_eq!(traversal.next(), None);
}

#[test]
fn only_y_decreasing() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: 0.0,
        start_y: 4.0,
        diff_x: 0.0,
        diff_y: -4.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.25,
        x_index: 0,
        last_y_index: 3,
        next_y_index: 2,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.5,
        x_index: 0,
        last_y_index: 2,
        next_y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.75,
        x_index: 0,
        last_y_index: 1,
        next_y_index: 0,
    }));
    assert_eq!(traversal.next(), None);
}

#[test]
fn perfectly_diagonal_increasing() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 4.0,
        diff_y: 4.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.25,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.25,
        x_index: 1,
        last_y_index: 0,
        next_y_index: 1,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.5,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.5,
        x_index: 2,
        last_y_index: 1,
        next_y_index: 2,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.75,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 2,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.75,
        x_index: 3,
        last_y_index: 2,
        next_y_index: 3,
    }));

    assert_eq!(traversal.next(), None);
}

#[test]
fn perfectly_diagonal_decreasing() {
    let bounds = Bounds::origin_square(4);
    let ray = Ray {
        start_x: 4.0,
        start_y: 4.0,
        diff_x: -4.0,
        diff_y: -4.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.25,
        last_x_index: 3,
        next_x_index: 2,
        y_index: 3,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.25,
        x_index: 2,
        last_y_index: 3,
        next_y_index: 2,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.5,
        last_x_index: 2,
        next_x_index: 1,
        y_index: 2,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.5,
        x_index: 1,
        last_y_index: 2,
        next_y_index: 1,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.75,
        last_x_index: 1,
        next_x_index: 0,
        y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.75,
        x_index: 0,
        last_y_index: 1,
        next_y_index: 0,
    }));

    assert_eq!(traversal.next(), None);
}

#[test]
fn half_diagonal_increasing() {
    let bounds = Bounds::origin_rectangle(4, 2);
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 4.0,
        diff_y: 2.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray, bounds);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.25,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.5,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.5,
        x_index: 2,
        last_y_index: 0,
        next_y_index: 1,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.75,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 1,
    }));

    assert_eq!(traversal.next(), None);
}

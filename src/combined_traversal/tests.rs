use crate::combined_traversal::{BoundaryCrossing, CombinedBoundaryTraversal};
use crate::ray::Ray;

#[test]
fn only_x_increasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 1.0,
        diff_y: 0.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.0,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 1.0,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 2.0,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 0,
    }));
}

#[test]
fn only_x_decreasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: -1.0,
        diff_y: 0.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.0,
        last_x_index: 0,
        next_x_index: -1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 1.0,
        last_x_index: -1,
        next_x_index: -2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 2.0,
        last_x_index: -2,
        next_x_index: -3,
        y_index: 0,
    }));
}

#[test]
fn only_y_increasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 0.0,
        diff_y: 1.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.0,
        x_index: 0,
        last_y_index: 0,
        next_y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 1.0,
        x_index: 0,
        last_y_index: 1,
        next_y_index: 2,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 2.0,
        x_index: 0,
        last_y_index: 2,
        next_y_index: 3,
    }));
}

#[test]
fn only_y_decreasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 0.0,
        diff_y: -1.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.0,
        x_index: 0,
        last_y_index: 0,
        next_y_index: -1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 1.0,
        x_index: 0,
        last_y_index: -1,
        next_y_index: -2,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 2.0,
        x_index: 0,
        last_y_index: -2,
        next_y_index: -3,
    }));
}

#[test]
fn perfectly_diagonal_increasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 1.0,
        diff_y: 1.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.0,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.0,
        x_index: 1,
        last_y_index: 0,
        next_y_index: 1,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 1.0,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 1.0,
        x_index: 2,
        last_y_index: 1,
        next_y_index: 2,
    }));
}

#[test]
fn perfectly_diagonal_decreasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: -1.0,
        diff_y: -1.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.0,
        last_x_index: 0,
        next_x_index: -1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.0,
        x_index: -1,
        last_y_index: 0,
        next_y_index: -1,
    }));

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 1.0,
        last_x_index: -1,
        next_x_index: -2,
        y_index: -1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 1.0,
        x_index: -2,
        last_y_index: -1,
        next_y_index: -2,
    }));
}

#[test]
fn half_diagonal_increasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 2.0,
        diff_y: 1.0,
    };

    let mut traversal = CombinedBoundaryTraversal::new(ray);

    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 0.0,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 0.0,
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
    assert_eq!(traversal.next(), Some(BoundaryCrossing::X {
        t: 1.0,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 1,
    }));
    assert_eq!(traversal.next(), Some(BoundaryCrossing::Y {
        t: 1.0,
        x_index: 3,
        last_y_index: 1,
        next_y_index: 2,
    }));
}
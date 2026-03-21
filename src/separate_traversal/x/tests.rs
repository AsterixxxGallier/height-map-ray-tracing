use crate::ray::Ray;
use crate::separate_traversal::{XBoundaryCrossing, XBoundaryTraversal};

#[test]
fn only_x_increasing() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 1.0,
        diff_y: 0.0,
    };
    
    let mut traversal = XBoundaryTraversal::new(ray).unwrap();

    assert_eq!(traversal.next(), Some(XBoundaryCrossing {
        t: 0.0,
        last_x_index: 0,
        next_x_index: 1,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(XBoundaryCrossing {
        t: 1.0,
        last_x_index: 1,
        next_x_index: 2,
        y_index: 0,
    }));
    assert_eq!(traversal.next(), Some(XBoundaryCrossing {
        t: 2.0,
        last_x_index: 2,
        next_x_index: 3,
        y_index: 0,
    }));
}
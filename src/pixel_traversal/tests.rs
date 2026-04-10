use crate::pixel_traversal::CombinedPixelTraversal;
use crate::ray::Ray;
use crate::boundary_traversal::CombinedBoundaryTraversal;

#[test]
fn edge_case() {
    let ray = Ray {
        start_x: 0.0,
        start_y: 2048.0,
        diff_x: 2048.0,
        diff_y: 46852776000.0,
    };

    // let mut traversal = ThinCombinedBoundaryTraversal::new(ray);
    // traversal.for_each(|crossing| {
    //     dbg!(crossing);
    // });

    let mut traversal = CombinedPixelTraversal::new(ray);
    traversal.for_each(|segment| {
        assert!(segment.pixel_x >= 0, "{segment:?}");
        assert!(segment.pixel_y >= 0, "{segment:?}");
        assert!(segment.pixel_x < 2048, "{segment:?}");
        assert!(segment.pixel_y < 2048, "{segment:?}");
    });
}

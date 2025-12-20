use geo::{BoundingRect, Distance, Euclidean};
use geo::{Line, Point};
use rstar::RTreeObject;
use rstar::AABB;

// Currently unused
// enum CrsType {
//     Projected,
//     Geographic,
// }

/// Represents a component `Line` of a target `LineString`
///
/// The tuple stores the `Line` struct and the distance buffer to be used.
/// It's [rstar::Envelope] method grows the [rstar::AABB] in x and y directions
/// by the distance.
#[derive(Debug, Clone)]
pub struct TarLine(pub Line<f64>, pub f64);
impl TarLine {
    /// Create an AABB from the contained `Line`
    pub fn envelope(&self) -> AABB<Point> {
        let padding_dist = self.1;
        let bb = self.0.bounding_rect();
        let (ll_x, ll_y) = bb.min().x_y();
        let (ur_x, ur_y) = bb.max().x_y();
        let ll = Point::new(ll_x - padding_dist, ll_y - padding_dist);
        let ur = Point::new(ur_x + padding_dist, ur_y + padding_dist);
        AABB::from_corners(ll, ur)
    }

    /// Calculate distance between a target and source line string
    // Using geographic coordinate systems should be avoided with this algorithm.
    // Measuring distance in geographic space between two lines finds the minimum
    // distance between vertices whereas the euclidean distance between two lines
    // considers all possible distances.
    // Geographic distance may create false negatives.
    pub fn distance(&self, other: &Line) -> f64 {
        Euclidean::distance(&self.0, other)
    }
}

impl RTreeObject for TarLine {
    type Envelope = AABB<Point>;
    fn envelope(&self) -> Self::Envelope {
        self.envelope()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::coord;

    #[test]
    fn test_tarline_envelope_basic() {
        let line = Line::new(coord! {x: 0.0, y: 0.0}, coord! {x: 10.0, y: 10.0});
        let padding = 2.0;
        let tarline = TarLine(line, padding);

        let envelope = tarline.envelope();

        // Lower left should be (-2, -2) and upper right should be (12, 12)
        assert_eq!(envelope.lower().x(), -2.0);
        assert_eq!(envelope.lower().y(), -2.0);
        assert_eq!(envelope.upper().x(), 12.0);
        assert_eq!(envelope.upper().y(), 12.0);
    }

    #[test]
    fn test_tarline_envelope_horizontal_line() {
        let line = Line::new(coord! {x: 0.0, y: 5.0}, coord! {x: 10.0, y: 5.0});
        let padding = 1.0;
        let tarline = TarLine(line, padding);

        let envelope = tarline.envelope();

        assert_eq!(envelope.lower().x(), -1.0);
        assert_eq!(envelope.lower().y(), 4.0);
        assert_eq!(envelope.upper().x(), 11.0);
        assert_eq!(envelope.upper().y(), 6.0);
    }

    #[test]
    fn test_tarline_envelope_vertical_line() {
        let line = Line::new(coord! {x: 5.0, y: 0.0}, coord! {x: 5.0, y: 10.0});
        let padding = 1.5;
        let tarline = TarLine(line, padding);

        let envelope = tarline.envelope();

        assert_eq!(envelope.lower().x(), 3.5);
        assert_eq!(envelope.lower().y(), -1.5);
        assert_eq!(envelope.upper().x(), 6.5);
        assert_eq!(envelope.upper().y(), 11.5);
    }

    #[test]
    fn test_tarline_envelope_no_padding() {
        let line = Line::new(coord! {x: 1.0, y: 2.0}, coord! {x: 3.0, y: 4.0});
        let padding = 0.0;
        let tarline = TarLine(line, padding);

        let envelope = tarline.envelope();

        assert_eq!(envelope.lower().x(), 1.0);
        assert_eq!(envelope.lower().y(), 2.0);
        assert_eq!(envelope.upper().x(), 3.0);
        assert_eq!(envelope.upper().y(), 4.0);
    }

    #[test]
    fn test_tarline_distance() {
        let tarline = TarLine(
            Line::new(coord! {x: 0.0, y: 0.0}, coord! {x: 10.0, y: 0.0}),
            1.0
        );
        let other = Line::new(coord! {x: 0.0, y: 3.0}, coord! {x: 10.0, y: 3.0});

        let distance = tarline.distance(&other);

        // Parallel horizontal lines 3 units apart
        assert_eq!(distance, 3.0);
    }
}

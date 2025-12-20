pub mod get_matches;
pub mod interpolate;
mod overlap;
pub mod structs;

use crate::{
    overlap::*, overlap_range, solve_no_x_overlap, solve_no_y_overlap, structs::*, x_range,
    y_range, TarLine,
};
use geo::{BoundingRect, Distance, Euclidean, Length};
use rstar::primitives::{CachedEnvelope, GeomWithData};
use std::{cell::OnceCell, collections::BTreeMap, error::Error, fmt::Display};

/// Anime Error Type
#[derive(Debug, Clone)]
pub enum AnimeError {
    IncorrectLength,
    MatchesNotFound,
    AlreadyMatched(MatchesMap),
    ContainsNull,
}

impl Display for AnimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimeError::IncorrectLength => write!(f, "Variable to interpolate must have the same number of observations as the `target` lines"),
            AnimeError::MatchesNotFound => write!(f, "`matches` needs to be instantiated with `self.find_matches()`"),
            AnimeError::AlreadyMatched(_) => write!(f, "matches already found."),
            AnimeError::ContainsNull => write!(f, "cannot interpolate null values"),
        }
    }
}

impl Error for AnimeError {}

/// R* Tree for source geometries
pub type SourceTree = rstar::RTree<GeomWithData<CachedEnvelope<geo_types::Line>, (usize, f64)>>;

/// R* Tree for target geometries
pub type TargetTree = rstar::RTree<GeomWithData<CachedEnvelope<TarLine>, (usize, f64)>>;

/// Represents a partial source <-> target match
#[derive(Debug, Clone)]
pub struct MatchCandidate {
    /// The index of the source geometry
    pub source_index: usize,
    /// The amount of shared length between two geometries
    pub shared_len: f64,
}

/// Stores match length
///
/// The BTreeMap key is the index of the target geometry
/// whereas the entry contains
pub type TargetIndex = usize;
pub type MatchesMap = BTreeMap<TargetIndex, Vec<MatchCandidate>>;
pub type Matches = OnceCell<MatchesMap>;

/// Approximate Network Matching, Integration, and Enrichment
///
/// This struct contains all of the information needed to perform
/// and store the results of the ANIME algorithm.
///
/// The `source_tree` and `target_tree` are used to perform the
/// partial matching based on the `distance_tolerance` and
/// `angle_tolerance`. The results of the matching
/// are stored in the `BTreeMap`.
///
/// The lengths, represented as `Vec<f64>` are required for the
/// integration of attributes.
#[derive(Clone, Debug)]
pub struct Anime {
    pub distance_tolerance: f64,
    pub angle_tolerance: f64,
    pub source_tree: SourceTree,
    pub source_lens: Vec<f64>,
    pub target_tree: TargetTree,
    pub target_lens: Vec<f64>,
    pub matches: Matches,
}

impl Anime {
    /// Load source and target `LineString` geometries
    ///
    /// This creates two R* Trees using cached envelopes for each component
    /// line in a LineString. In addition to the envelope, the slope and
    /// index of the LineString is stored.
    pub fn load_geometries(
        source: impl Iterator<Item = geo_types::LineString>,
        target: impl Iterator<Item = geo_types::LineString>,
        distance_tolerance: f64,
        angle_tolerance: f64,
    ) -> Self {
        let mut source_lens = Vec::new();
        let mut target_lens = Vec::new();
        let source_tree = create_source_rtree(source, &mut source_lens);
        let target_tree = create_target_rtree(target, &mut target_lens, distance_tolerance);
        Self {
            distance_tolerance,
            angle_tolerance,
            source_tree,
            source_lens,
            target_tree,
            target_lens,
            matches: OnceCell::new(),
        }
    }

    /// Find candidate matches between source and target
    ///
    /// The matches can only be found once for each source and target pair.
    pub fn find_matches(&mut self) -> Result<&mut Anime, AnimeError> {
        let matches = find_candidate_matches(
            &self.source_tree,
            &self.target_tree,
            self.angle_tolerance,
            self.distance_tolerance,
        );
        self.matches
            .set(matches)
            .map_err(|e| AnimeError::AlreadyMatched(e))?;
        Ok(self)
    }

    /// Insert linestring geometries and find matches
    pub fn new(
        source: impl Iterator<Item = geo_types::LineString>,
        target: impl Iterator<Item = geo_types::LineString>,
        distance_tolerance: f64,
        angle_tolerance: f64,
    ) -> Self {
        let mut source_lens = Vec::new();
        let mut target_lens = Vec::new();
        let source_tree = create_source_rtree(source, &mut source_lens);
        let target_tree = create_target_rtree(target, &mut target_lens, distance_tolerance);
        let matches = find_candidate_matches(
            &source_tree,
            &target_tree,
            angle_tolerance,
            distance_tolerance,
        );
        Self {
            distance_tolerance,
            angle_tolerance,
            source_tree,
            source_lens,
            target_tree,
            target_lens,
            matches: OnceCell::from(matches),
        }
    }
}
fn find_candidate_matches(
    source_tree: &SourceTree,
    target_tree: &TargetTree,
    angle_tolerance: f64,
    distance_tolerance: f64,
) -> MatchesMap {
    let mut matches: MatchesMap = BTreeMap::new();
    let candidates = source_tree.intersection_candidates_with_other_tree(target_tree);

    candidates.for_each(|(cx, cy)| {
        let xbb = cx.geom().bounding_rect();
        let ybb = cy.geom().0.bounding_rect();

        // extract cached slopes and index positions
        let (i, x_slope) = cx.data;
        let (j, y_slope) = cy.data;

        // convert calculated slopes to degrees
        let x_deg = x_slope.atan().to_degrees();
        let y_deg = y_slope.atan().to_degrees();

        // compare slopes:
        let is_tolerant = (x_deg - y_deg).abs() < angle_tolerance;

        // if the slopes are within tolerance then we check for overlap
        if is_tolerant {
            let xx_range = x_range(&xbb);
            let xy_range = x_range(&ybb);
            let x_overlap = overlap_range(xx_range, xy_range);
            let y_overlap = overlap_range(y_range(&xbb), y_range(&ybb));

            // if theres overlap then we do a distance based check
            // following, check that they're within distance tolerance,
            // if so, calculate the shared length
            if x_overlap.is_some() || y_overlap.is_some() {
                // calculate the distance from the line segment
                // if its within our threshold we include it;
                let d = cy.geom().distance(cx.geom());

                // if distance is less than or equal to tolerance, add the key
                if d <= distance_tolerance {
                    let shared_len = if x_slope.atan().to_degrees() <= 45.0 {
                        if x_overlap.is_some() {
                            let (p1, p2) =
                                solve_no_y_overlap(x_overlap.unwrap(), cx.geom(), &x_slope);

                            Euclidean::distance(&p1, &p2)
                        } else {
                            0.0
                        }
                    } else if y_overlap.is_some() {
                        let (p1, p2) = solve_no_x_overlap(y_overlap.unwrap(), cx.geom(), &x_slope);
                        Euclidean::distance(&p1, &p2)
                    } else {
                        0.0
                    };
                    // add 1 for R indexing
                    // ensures that no duplicates are inserted. Creates a new empty vector is needed
                    let entry = matches.entry(j).or_default();

                    if let Some(tuple) = entry.iter_mut().find(|x| x.source_index == i) {
                        tuple.shared_len += shared_len;
                    } else {
                        entry.push(MatchCandidate {
                            source_index: i,
                            shared_len,
                        });
                    }
                }
            }
        }
    });
    matches
}

fn create_source_rtree(
    x: impl Iterator<Item = geo_types::LineString>,
    source_lens: &mut Vec<f64>,
) -> SourceTree {
    let to_insert = x
        .enumerate()
        .flat_map(|(i, xi)| {
            let xi_len = xi.length::<Euclidean>();
            source_lens.push(xi_len);
            let components = xi
                .lines()
                .map(|li| {
                    let slope = li.slope();
                    let env = CachedEnvelope::new(li);
                    GeomWithData::new(env, (i, slope))
                })
                .collect::<Vec<GeomWithData<_, _>>>();
            components
        })
        .collect::<Vec<_>>();

    rstar::RTree::bulk_load(to_insert)
}

fn create_target_rtree(
    y: impl Iterator<Item = geo_types::LineString>,
    target_lens: &mut Vec<f64>,
    dist: f64,
) -> TargetTree {
    let to_insert = y
        .enumerate()
        .flat_map(|(i, yi)| {
            let yi_len = yi.length::<Euclidean>();
            target_lens.push(yi_len);
            let components = yi
                .lines()
                .map(|li| {
                    let tl = TarLine(li, dist);
                    let slope = li.slope();
                    let env = CachedEnvelope::new(tl);
                    GeomWithData::new(env, (i, slope))
                })
                .collect::<Vec<GeomWithData<_, _>>>();
            components
        })
        .collect::<Vec<_>>();

    rstar::RTree::bulk_load(to_insert)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::{coord, LineString};

    fn create_simple_source_target() -> (Vec<LineString>, Vec<LineString>) {
        let source = vec![
            LineString::new(vec![coord! {x: 0.0, y: 0.0}, coord! {x: 10.0, y: 0.0}]),
            LineString::new(vec![coord! {x: 0.0, y: 5.0}, coord! {x: 10.0, y: 5.0}]),
        ];

        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.1},
            coord! {x: 10.0, y: 0.1},
        ])];

        (source, target)
    }

    #[test]
    fn test_anime_new() {
        let (source, target) = create_simple_source_target();

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        assert_eq!(anime.distance_tolerance, 0.5);
        assert_eq!(anime.angle_tolerance, 5.0);
        assert_eq!(anime.source_lens.len(), 2);
        assert_eq!(anime.target_lens.len(), 1);
        assert!(anime.matches.get().is_some());
    }

    #[test]
    fn test_anime_load_geometries() {
        let (source, target) = create_simple_source_target();

        let anime = Anime::load_geometries(source.into_iter(), target.into_iter(), 0.5, 5.0);

        assert_eq!(anime.distance_tolerance, 0.5);
        assert_eq!(anime.angle_tolerance, 5.0);
        assert_eq!(anime.source_lens.len(), 2);
        assert_eq!(anime.target_lens.len(), 1);
        assert!(anime.matches.get().is_none());
    }

    #[test]
    fn test_anime_find_matches() {
        let (source, target) = create_simple_source_target();

        let mut anime = Anime::load_geometries(source.into_iter(), target.into_iter(), 0.5, 5.0);

        assert!(anime.matches.get().is_none());

        let result = anime.find_matches();
        assert!(result.is_ok());
        assert!(anime.matches.get().is_some());
    }

    #[test]
    fn test_anime_find_matches_already_matched() {
        let (source, target) = create_simple_source_target();

        let mut anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        // Matches already exist from new()
        let result = anime.find_matches();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnimeError::AlreadyMatched(_)));
    }

    #[test]
    fn test_source_lens_calculated() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.1},
            coord! {x: 10.0, y: 0.1},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        assert_eq!(anime.source_lens.len(), 1);
        assert_eq!(anime.source_lens[0], 10.0);
    }

    #[test]
    fn test_target_lens_calculated() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.1},
            coord! {x: 5.0, y: 0.1},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        assert_eq!(anime.target_lens.len(), 1);
        assert_eq!(anime.target_lens[0], 5.0);
    }

    #[test]
    fn test_parallel_lines_match() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.1},
            coord! {x: 10.0, y: 0.1},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        let matches = anime.matches.get().unwrap();
        assert!(matches.contains_key(&0));
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_distant_lines_no_match() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 100.0},
            coord! {x: 10.0, y: 100.0},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        let matches = anime.matches.get().unwrap();
        assert!(matches.is_empty() || !matches.contains_key(&0));
    }

    #[test]
    fn test_perpendicular_lines_no_match() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];
        let target = vec![LineString::new(vec![
            coord! {x: 5.0, y: -5.0},
            coord! {x: 5.0, y: 5.0},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        let matches = anime.matches.get().unwrap();
        // Perpendicular lines should not match due to angle tolerance
        assert!(matches.is_empty() || !matches.contains_key(&0));
    }

    #[test]
    fn test_multiple_source_to_one_target() {
        let source = vec![
            LineString::new(vec![coord! {x: 0.0, y: 0.0}, coord! {x: 5.0, y: 0.0}]),
            LineString::new(vec![coord! {x: 5.0, y: 0.0}, coord! {x: 10.0, y: 0.0}]),
        ];
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.1},
            coord! {x: 10.0, y: 0.1},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        let matches = anime.matches.get().unwrap();
        if let Some(target_matches) = matches.get(&0) {
            // Target should potentially match both source lines
            assert!(!target_matches.is_empty());
        }
    }

    #[test]
    fn test_match_candidate_structure() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.1},
            coord! {x: 10.0, y: 0.1},
        ])];

        let anime = Anime::new(source.into_iter(), target.into_iter(), 0.5, 5.0);

        let matches = anime.matches.get().unwrap();
        if let Some(target_matches) = matches.get(&0) {
            for candidate in target_matches {
                assert!(candidate.shared_len >= 0.0);
            }
        }
    }

    #[test]
    fn test_anime_error_display() {
        let err = AnimeError::IncorrectLength;
        assert!(err.to_string().contains("same number"));

        let err = AnimeError::MatchesNotFound;
        assert!(err.to_string().contains("matches"));

        let err = AnimeError::ContainsNull;
        assert!(err.to_string().contains("null"));
    }

    #[test]
    fn test_create_source_rtree() {
        let source = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];

        let mut lens = Vec::new();
        let tree = create_source_rtree(source.into_iter(), &mut lens);

        assert_eq!(lens.len(), 1);
        assert_eq!(lens[0], 10.0);
        assert!(tree.size() > 0);
    }

    #[test]
    fn test_create_target_rtree() {
        let target = vec![LineString::new(vec![
            coord! {x: 0.0, y: 0.0},
            coord! {x: 10.0, y: 0.0},
        ])];

        let mut lens = Vec::new();
        let tree = create_target_rtree(target.into_iter(), &mut lens, 0.5);

        assert_eq!(lens.len(), 1);
        assert_eq!(lens[0], 10.0);
        assert!(tree.size() > 0);
    }
}

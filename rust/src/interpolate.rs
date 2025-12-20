use core::f64;

use arrow::array::{Array, Float64Array};

use crate::{Anime, AnimeError};

/// Intensive or Extensive Interpolation
///
/// Intensive interpolation weights the attribute by the
/// shared length divided by the length of the source geometry.
///
/// Extensive interpolation weights the attribute by the shared
/// length divided by the length of the target geometry.
pub enum Tensive {
    In,
    Ex,
}

impl Anime {
    /// Perform numeric attribute interpolation
    pub fn interpolate(
        &self,
        var: &Float64Array,
        tensive: Tensive,
    ) -> Result<Float64Array, AnimeError> {
        match tensive {
            Tensive::In => self.interpolate_intensive(var),
            Tensive::Ex => self.interpolate_extensive(var),
        }
    }

    /// Extensive Interpolation from the source to the target
    ///
    /// Extensive interpolation is a length weighted sum of a variable
    /// onto the target variable.
    ///
    /// Let the shared length between target j and source i be the variable $SL_{ij}$
    ///
    /// $$
    /// \hat{Y}_j = \sum_{i} \frac{SL_{ij}}{length(i)} \times Y_i
    /// $$
    pub fn interpolate_extensive(
        &self,
        source_var: &Float64Array,
    ) -> Result<Float64Array, AnimeError> {
        // Check if `source_var` matches the number of source geometries
        if source_var.len() != self.source_lens.len() {
            return Err(AnimeError::IncorrectLength);
        }

        if source_var.null_count() > 0 {
            return Err(AnimeError::ContainsNull);
        }

        // Retrieve matches (or return error if not found)
        let matches_map = self.matches.get().ok_or(AnimeError::MatchesNotFound)?;

        // Interpolate extensive variable
        let res = (0..self.target_lens.len()).map(|target_idx| {
            if let Some(matches) = matches_map.get(&target_idx) {
                let value = matches.iter().fold(0.0, |acc, mi| {
                    let source_idx = mi.source_index;
                    let shared_len = mi.shared_len;

                    // Weight = shared length / total length of source geometry
                    let wt = shared_len / self.source_lens[source_idx];
                    let sv = source_var.value(source_idx);

                    // here we handle NA values and NaN by skipping them
                    if sv.is_nan() || sv == f64::MAX {
                        return acc;
                    }

                    // Weighted contribution of source variable
                    acc + (sv * wt)
                });
                value
            } else {
                0.0
            }
        });
        Ok(Float64Array::from_iter_values(res))
    }

    /// Intensive Interpolation from the source to the target
    ///
    /// Intensive interpolation is a length weighted mean of a variable
    /// onto the target variable. Intensive variables, like densities, require
    /// averaging the source values based on the overlap with the target.
    ///
    /// Let the shared length between target j and source i be the variable $SL_{ij}$,
    /// and let the length of the target be $length(j)$.
    ///
    /// $$
    /// \hat{Y}_j = \frac{\sum_{i} \frac{SL_{ij}}{length(j)} \times Y_i}{\sum_{i} \frac{SL_{ij}}{length(j)}}
    /// $$
    ///
    /// In this formula:
    /// - $SL_{ij}$ is the shared length between target j and source i.
    /// - $Y_i$ is the source variable at index i.
    /// - $length(j)$ is the length of the target feature j.
    ///
    /// The result is a weighted mean of the source variable values, where the weight
    /// is based on the shared length between the source and the target, normalized by
    /// the length of the target.
    pub fn interpolate_intensive(
        &self,
        source_var: &Float64Array,
    ) -> Result<Float64Array, AnimeError> {
        let nv = source_var.len();
        let n_tar = self.source_lens.len(); // Assuming target_lens represent target lengths.

        if nv != n_tar {
            return Err(AnimeError::IncorrectLength);
        }

        if source_var.null_count() > 0 {
            return Err(AnimeError::ContainsNull);
        }

        // Ensure matches are loaded
        let matches_map = self.matches.get().ok_or(AnimeError::MatchesNotFound)?;

        let res = (0..self.target_lens.len()).map(|target_idx| {
            if let Some(matches) = matches_map.get(&target_idx) {
                // Calculate the weighted sum of the source variable values and normalize by the total weight
                let (numerator, denominator) =
                    matches.iter().fold((0.0, 0.0), |(acc_num, acc_den), mi| {
                        let source_idx = mi.source_index;

                        // Weight based on shared length and target length
                        let wt = mi.shared_len / self.target_lens[target_idx];
                        let sv = source_var.value(source_idx);

                        // here we handle NA values and NaN by skipping them
                        if sv.is_nan() || sv == f64::MAX {
                            return (acc_num, acc_den);
                        }

                        let weighted_value = sv * wt;

                        // Update the numerator (weighted sum) and denominator (total weight)
                        (acc_num + weighted_value, acc_den + wt)
                    });

                // If the total weight is greater than zero, compute the weighted mean
                if denominator > 0.0 {
                    numerator / denominator
                } else {
                    0.0
                }
            } else {
                0.0
            }
        });

        Ok(Float64Array::from_iter_values(res))
    }
}

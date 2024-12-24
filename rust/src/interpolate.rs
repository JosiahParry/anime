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
    pub fn interpolate(&self, var: &[f64], tensive: Tensive) -> Result<Vec<f64>, AnimeError> {
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
    pub fn interpolate_extensive(&self, source_var: &[f64]) -> Result<Vec<f64>, AnimeError> {
        // Check if `source_var` matches the number of source geometries
        if source_var.len() != self.source_lens.len() {
            return Err(AnimeError::IncorrectLength);
        }

        // Interpolate extensive variable
        let res = self
            .matches
            .iter()
            .map(|(_, matches)| {
                matches.iter().fold(0.0, |acc, mi| {
                    let source_idx = mi.source_index;
                    let shared_len = mi.shared_len;

                    // Weight = shared length / total length of source geometry
                    let wt = shared_len / self.source_lens[source_idx];

                    // Weighted contribution of source variable
                    acc + (source_var[source_idx] * wt)
                })
            })
            .collect::<Vec<f64>>();

        Ok(res)
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
    pub fn interpolate_intensive(&self, source_var: &[f64]) -> Result<Vec<f64>, AnimeError> {
        let nv = source_var.len();
        let n_tar = self.source_lens.len(); // Assuming target_lens represent target lengths.

        if nv != n_tar {
            return Err(AnimeError::IncorrectLength);
        }

        let res = self
            .matches
            .iter()
            .map(|(target_idx, matches)| {
                // Calculate the weighted sum of the source variable values and normalize by the total weight
                let (numerator, denominator) =
                    matches.iter().fold((0.0, 0.0), |(acc_num, acc_den), mi| {
                        let source_idx = mi.source_index;

                        // Weight based on shared length and target length
                        let wt =
                            mi.shared_len / self.target_lens.get(*target_idx as usize).unwrap(); // Using target length for weight
                        let weighted_value = source_var[source_idx] * wt;

                        // Update the numerator (weighted sum) and denominator (total weight)
                        (acc_num + weighted_value, acc_den + wt)
                    });

                // If the total weight is greater than zero, compute the weighted mean
                if denominator > 0.0 {
                    numerator / denominator
                } else {
                    0.0 // If no overlap, return 0 or handle differently
                }
            })
            .collect::<Vec<f64>>();

        Ok(res)
    }
}

// rnet_aggregate_extensive <- function(
//     x, y, matches, ...,
//     y_len = as.numeric(sf::st_length(y))
//   ) {
//   # capture variables
//   vars <- rlang::ensyms(...)
//   # get var-names
//   var_names <- vapply(vars, rlang::as_string, character(1))
//   # TODO validate variables are in y before subsetting
//   # extract j index
//   j <- matches$j
//   # subset vars by j to get ij pairs
//   ij <- rlang::set_names(lapply(var_names, \(.x) y[[.x]][j]), var_names)
//   # combine into 1 df
//   dplyr::bind_cols(matches, ij) |>
//     dplyr::mutate(
//       wt = shared_len / as.numeric(y_len[j])
//     ) |>
//     dplyr::group_by(i) |>
//     dplyr::summarise(dplyr::across(
//       -all_of(c("j", "shared_len", "wt")),
//       ~ sum(.x * wt, na.rm = TRUE)
//     ))
// }

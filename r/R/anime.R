validate_lines <- function(x, error_call = rlang::caller_call()) {
  if (!wk::is_handleable(x)) {
    rlang::abort("Unable to process provided geometry as geoarrow linestring array")
  }
  geom_types <- unique(wk::wk_meta(x)[["geometry_type"]])
  if (!all(geom_types == 2L)) {
    rlang::abort(
      "Unexpected geometries. Expected linestrings.",
      footer = sprintf("Instead found %s", toString(wk::wk_geometry_type_label(geom_types))),
      call = error_call
    )
  }
  geoarrow::as_geoarrow_array(x)
}

#' Match two sets of lines
#'
#' @param source a linestring geometry. Must be handleable by `wk`.
#' @param target a linestring geometry. Must be handleable by `wk`.
#' @param distance_tolerance the maximum distance between two linestrings to be considered a match.
#' @param angle_tolerance the maximum angle difference between two lines to be considered a match.
#' @return an object of class `anime`
#' @export
anime <- function(source, target, distance_tolerance = 10, angle_tolerance = 5) {
  if (!rlang::is_bare_numeric(distance_tolerance, 1)) {
    rlang::abort("`distance_tolerance` must be a scalar numeric")
  }

  if (distance_tolerance <= 0) {
    rlang::abort("`distance_tolerance` must be a positive")
  }

  if (angle_tolerance <= 0) {
    rlang::abort("`angle_tolerance` must be a positive")
  }

  if (angle_tolerance >= 90) {
    rlang::abort("`angle_tolerance` must be less than 90 degrees")
  }

  if (!rlang::is_bare_numeric(angle_tolerance, 1)) {
    rlang::abort("`distance_tolerance` must be a scalar numeric")
  }

  source <- validate_lines(source)
  target <- validate_lines(target)

  init_anime(source, target, distance_tolerance, angle_tolerance)
}

#' @export
as.data.frame.anime <- function(x, ...) {
  get_matches(x)
}

#' Get Partial Matches
#'
#' Extract the partial matches from the `anime` object
#' as a `data.frame`.
#'
#' @param x an `anime` object as created with `anime()`.
#'
#' @returns
#' A data.frame with 5 columns:
#' - `target_id`: the 1-based index of the target linestring
#' - `source_id`: the 1-based index of the source linestring
#' - `shared_len`: the shared length between the `source` and `target` in the CRS's units
#' - `source_weighted`: the `shared_len` divided by the length of the source linestring
#' - `target_weighted`: the `shared_len` divided by the length of the target linestring
#' @export
get_matches <- function(x) {
  if (!inherits(x, "anime")) {
    rlang::abort("Expected an `anime` object")
  }

  res <- get_matches_(x)

  structure(res, class = c("tbl", "data.frame"))
}

#' @export
print.anime <- function(x, ...) {
  .info <- anime_print_helper(x)
  to_print <- c(
    "<anime>",
    sprintf("matches: %i", .info$n_matches),
    sprintf("sources: %i", .info$source_fts),
    sprintf("targets: %i", .info$target_fts),
    sprintf("angle tolerance: %.1f", .info$angle_tolerance),
    sprintf("distance tolerance: %.1f", .info$distance_tolerance)
  )

  cat(to_print, sep = "\n")
  invisible(to_print)
}

#' Interpolate extensive variables
#'
#' Interpolate values from the source geometry to the target geometry.
#' Intensive properties are values which are independent of the geometry's size.
#' These are values such as a density or temperature.
#'
#' @param x a numeric variable with the same length as the source geometry
#' @param matches an `anime` object created with `anime()`
#'
#' @export
interpolate_extensive <- function(x, matches) {
  if (!inherits(matches, "anime")) {
    rlang::abort("Expected an `anime` object")
  }
  if (!rlang::is_bare_numeric(x)) {
    rlang::abort("`x` must be a numeric vector.")
  }

  # if (anyNA(x)) {
  #   rlang::abort("Cannot interpolate missing values.")
  # }

  interpolate_extensive_(as.double(x), matches)
}


#' Interpolate extensive variables
#'
#' Interpolate values from the source geometry to the target geometry.
#' Extensive properties are values which are dependent upon the geometry's size.
#' Extensive properties would be a population or length.
#'
#' @inheritParams interpolate_extensive
#' @export
interpolate_intensive <- function(x, matches) {
  if (!inherits(matches, "anime")) {
    rlang::abort("Expected an `anime` object")
  }
  if (!rlang::is_bare_numeric(x)) {
    rlang::abort("`x` must be a numeric vector.")
  }

  # if (anyNA(x)) {
  #   rlang::abort("Cannot interpolate missing values.")
  # }

  interpolate_intensive_(as.double(x), matches)
}

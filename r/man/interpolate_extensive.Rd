% Generated by roxygen2: do not edit by hand
% Please edit documentation in R/anime.R
\name{interpolate_extensive}
\alias{interpolate_extensive}
\title{Interpolate extensive variables}
\usage{
interpolate_extensive(x, matches)
}
\arguments{
\item{x}{a numeric variable with the same length as the source geometry}

\item{matches}{an \code{anime} object created with \code{anime()}}
}
\description{
Interpolate values from the source geometry to the target geometry.
Intensive properties are values which are independent of the geometry's size.
These are values such as a density or temperature.
}

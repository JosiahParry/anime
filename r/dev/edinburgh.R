library(sf)
library(dplyr)
library(anime)
library(mapview)

# See https://github.com/JosiahParry/anime/issues/28

target_minimal <- data.frame(
  x = 0:10,
  y = 3
) |>
  as.matrix() |>
  st_linestring() |>
  st_geometry() |>
  st_set_crs(27700) |>
  st_as_sf()
plot(target_minimal)

# Example 1: simple 2 lines above and below with length of 1 and value of 1 and 2
source_minimal_1 <- list(
  data.frame(
    x = 0:10,
    y = 1
  ),
  data.frame(
    x = 0:10,
    y = 4
  )
) |>
  lapply(function(x) {
    as.matrix(x) |>
      st_linestring()
  }) |>
  st_sfc() |>
  st_set_crs(27700) |>
  st_sf(geometry = _) |>
  mutate(v = 1:2)

mapview(source_minimal_1) +
  mapview(target_minimal, add = TRUE)



# Panics: bug?
# anime_1 = anime(
#   source = source_minimal_1,
#   target = target_minimal
# )

anime_1 <- anime(
  source = source_minimal_1,
  target = target_minimal,
  distance_tolerance = 5,
  angle_tolerance = 5
)

anime_1_matches <- anime::get_matches(anime_1)

v_extensive_1 <- anime::interpolate_extensive(
  source_minimal_1$v,
  anime_1
)
v_extensive_1
# [1] 3

source_minimal_2 <- list(
  data.frame(
    x = 0:9,
    y = 1
  ),
  data.frame(
    x = 9:10,
    y = 1
  ),
  data.frame(
    x = 0:10,
    y = 4
  )
) |>
  lapply(function(x) {
    as.matrix(x) |>
      st_linestring()
  }) |>
  st_sfc() |>
  st_set_crs(27700) |>
  st_sf(geometry = _) |>
  mutate(v = c(1, 10, 2))

mapview(source_minimal_2) +
  mapview(target_minimal, add = TRUE)

anime_2 <- anime(
  source = source_minimal_2,
  target = target_minimal,
  distance_tolerance = 5,
  angle_tolerance = 5
)

anime_2_matches <- anime::get_matches(anime_2)
anime_2_matches

v_extensive_2 <- anime::interpolate_extensive(
  source_minimal_2$v,
  anime_2
)
v_extensive_2
# [1] 3

source_minimal_2_matches <- bind_cols(
  source_minimal_2 |>
    st_drop_geometry(),
  anime_2_matches
)

source_minimal_2_aggregated <- source_minimal_2_matches |>
  summarise(
    # Based on the length of x
    # we could generalise by replacing 10 with
    # the length of the target (10 in this case):
    v_hardcoded = sum(v * shared_len) / 10,
    # Should this be the default?
    v_weighted = sum(v * target_weighted),
    v_max_shared = sum(v * shared_len) / max(shared_len)
  )
source_minimal_2_aggregated

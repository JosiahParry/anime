# see https://github.com/r-lib/pkgbuild/pull/157
if (!requireNamespace("tomledit")) {
  error("bootstrap.R requires `tomledit` to be installed.")
}

if (!requireNamespace("fs")) {
  error("bootstrap.R requires `fs` to be installed.")
}

if (!requireNamespace("rextendr")) {
  error("bootstrap.R requires `fs` to be installed.")
}


# Logging utils ----------------------------------------------------------

.style <- function(text, code) {
  if (Sys.getenv("TERM") != "dumb") {
    paste0("\033[", code, "m", text, "\033[0m")
  } else {
    text
  }
}

info <- function(...) {
  msg <- paste(..., collapse = " ")
  cat(.style("ℹ ", "1;36"), msg, "\n", sep = "")
  invisible(NULL)
}

warn <- function(...) {
  msg <- paste(..., collapse = " ")
  cat(.style("⚠ ", "1;33"), msg, "\n", sep = "")
  invisible(NULL)
}

error <- function(...) {
  msg <- paste(..., collapse = " ")
  stop(.style(paste0("✖ ", msg), "1;31"), call. = FALSE)
}

info("Bootstrapping rust dependencies")


library(tomledit)

info("Reading Cargo.toml")
# get R package crate name
# FIXME make this programattic
cargo_path <- "src/rust/Cargo.toml"
cargo_toml <- read_toml(cargo_path)
crate_name <- get_item(cargo_toml, c("package", "name"))

info("Fetching cargo metadata")
# get the pkg metadata as a list from cargo metadata
pkg_metadata <- rextendr:::read_cargo_metadata()

# get the index of the extendr package
crate_idx <- which(pkg_metadata$packages$name == crate_name)

# pkg dependencies
pkg_deps <- pkg_metadata$packages$dependencies[[crate_idx]]

# identify the dependencies that are not coming from a registry
non_reg_deps <- subset(pkg_deps, is.na(source))

# get package deps as a list
# we will need to modify the workspace / path in the Cargo.toml
pkg_deps <- get_item(cargo_toml, "dependencies")

# for each non_reg_dep:
# create a new directory src/rust/{dep}
workspace_toml <- read_toml(
  file.path(pkg_metadata$workspace_root, "Cargo.toml")
)

# extract workspace specific metadata from the manifest path
workspace_fields <- get_item(workspace_toml, "workspace")


# workspace members that are deps index:
workspace_idx <- which(
  pkg_metadata$packages$name %in% names(pkg_deps)
)

# get the member diirectory without absolute path
workspace_member_dep_paths <- basename(
  gsub(
    pkg_metadata$workspace_root,
    "",
    dirname(pkg_metadata$packages$manifest_path[workspace_idx])
  )
)

# "." is the R package workspace info
workspace_fields$members <- c(".", workspace_member_dep_paths)

# iterate through deps and copy them to the workspace
for (.dep in non_reg_deps$path) {
  dest <- file.path("src/rust", basename(.dep))
  info("Copying dep", .dep, " into ", dest)
  fs::dir_copy(.dep, dest, overwrite = TRUE)
}

# insert the workspace settings into the R package's Cargo.toml
info("Updating workspace settings!")
new_cargo_toml <- insert_items(
  cargo_toml,
  workspace = workspace_fields
)

# Remove workspace inheritance from package section since this file IS the workspace root
pkg_section <- get_item(new_cargo_toml, "package")
pkg_section$version <- workspace_fields$package$version
pkg_section$edition <- workspace_fields$package$edition
pkg_section$`rust-version` <- workspace_fields$package$`rust-version`
new_cargo_toml <- insert_items(new_cargo_toml, package = pkg_section)

# if the dependency is in the workspace members
# we need to set workspace = true and remove path if possible
workspace_dep_idx <- which(names(pkg_deps) %in% workspace_fields$members)

for (idx in workspace_dep_idx) {
  .dep <- pkg_deps[[idx]]
  .dep$path = NULL
  .dep$workspace = TRUE
  pkg_deps[[idx]] <- .dep
}

# insert the new deps
new_cargo_toml <- insert_items(
  new_cargo_toml,
  dependencies = pkg_deps
)

info("Updated Cargo.toml:")
cat(as.character(new_cargo_toml), sep = "\n")

# write the toml back
write_toml(new_cargo_toml, cargo_path)

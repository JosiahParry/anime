# Run as part of _targets.R to install packages
# rsgeo
install.packages(
  'rsgeo', 
  repos = c('https://josiahparry.r-universe.dev', 'https://cloud.r-project.org')
)

install.packages("pak")
pak::pak("JosiahParry/anime/r")


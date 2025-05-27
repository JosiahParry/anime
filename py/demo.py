`# uv tool run maturin develop
# uv pip install geoarrow-rust-core geoarrow-rust-compute geoarrow-rust-io
from anime import PyAnime
import pyarrow as pa
from geoarrow.rust.io import read_flatgeobuf

target = read_flatgeobuf("../r/inst/extdata/maine-osm-targets.fgb")
sources = read_flatgeobuf("../r/inst/extdata/maine-tigris-sources.fgb")

anime = PyAnime(target.column("").chunk(0), sources.column("").chunk(0), 10, 5)

matches = anime.get_matches()
pa.table(matches).to_pandas()


import random
li = random.sample(range(1, 101), 32)
pa.array(anime.interpolate_extensive(li))


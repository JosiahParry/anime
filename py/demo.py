# uv tool run maturin develop
# uv pip install geoarrow-rust-core geoarrow-rust-compute geoarrow-rust-io
from anime import PyAnime
import pyarrow as pa
import random
from geoarrow.rust.io import read_flatgeobuf

target = read_flatgeobuf("../r/inst/extdata/maine-osm-targets.fgb")
sources = read_flatgeobuf("../r/inst/extdata/maine-tigris-sources.fgb")

anime = PyAnime(target.column("").chunk(0), sources.column("").chunk(0), 10, 5)


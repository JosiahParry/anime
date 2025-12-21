# uv tool run maturin develop
# uv pip install geoarrow-rust-core geoarrow-rust-compute geoarrow-rust-io
import pyarrow as pa
from anime import PyAnime
from geoarrow.rust.io import read_flatgeobuf

target = read_flatgeobuf("../r/inst/extdata/maine-osm-targets.fgb")
sources = read_flatgeobuf("../r/inst/extdata/maine-tigris-sources.fgb")

anime = PyAnime(
    target.column("geometry").chunk(0), sources.column("geometry").chunk(0), 10, 5
)


print(anime.get_matches())

use pyo3::prelude::*;
pub mod anime_compat;
pub use anime_compat::*;

/// A Python module implemented in Rust.
#[pymodule]
fn py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAnime>()?;
    Ok(())
}

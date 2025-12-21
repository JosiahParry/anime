use pyo3::prelude::*;
pub mod anime_compat;
pub use anime_compat::*;

#[pymodule]
fn anime(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyAnime>()?;
    Ok(())
}

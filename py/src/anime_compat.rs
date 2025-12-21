use anime::Anime;
use arrow::{
    array::{Array, Float64Array},
    datatypes::Field,
};
use geo_traits::to_geo::ToGeoLineString;
use geoarrow::array::{from_arrow_array, AsGeoArrowArray, LineStringArray};
use pyo3::prelude::*;
use pyo3::{exceptions::PyTypeError, PyErr, PyResult};
use pyo3_arrow::{PyArray, PyTable};
use std::sync::Arc;

fn new_error(msg: String) -> PyErr {
    PyErr::new::<PyTypeError, _>(msg)
}

pub fn as_geoarrow_lines(x: PyArray) -> PyResult<LineStringArray> {
    let (array, field) = x.into_inner();
    let res =
        from_arrow_array(array.as_ref(), field.as_ref()).map_err(|e| new_error(e.to_string()))?;
    res.as_line_string_opt()
        .map(|l| l.to_owned())
        .ok_or_else(|| new_error("Expected native LineString array".to_string()))
}

#[pyclass(frozen)]
pub struct PyAnime(Anime);

unsafe impl Sync for PyAnime {}
unsafe impl Send for PyAnime {}

#[pymethods]
impl PyAnime {
    #[new]
    pub fn new(
        source: PyArray,
        target: PyArray,
        distance_tolerance: f64,
        angle_tolerance: f64,
    ) -> PyResult<Self> {
        use geoarrow_array::GeoArrowArrayAccessor;
        let source = as_geoarrow_lines(source)?;
        let target = as_geoarrow_lines(target)?;

        let res = Anime::new(
            source.iter_values().map(|xi| xi.unwrap().to_line_string()),
            target.iter_values().map(|xi| xi.unwrap().to_line_string()),
            distance_tolerance,
            angle_tolerance,
        );
        Ok(Self(res))
    }

    pub fn get_matches(&self) -> PyResult<PyTable> {
        let inner = self.0.get_matches().map_err(|e| new_error(e.to_string()))?;
        let schema = inner.schema();
        pyo3_arrow::PyTable::try_new(vec![inner], schema)
    }

    pub fn interpolate_intensive(&self, source_var: PyArray) -> PyResult<PyArray> {
        let d = source_var.array().into_data();
        let source_var = Float64Array::from(d);
        let res = Arc::new(
            self.0
                .interpolate_intensive(&source_var)
                .map_err(|e| new_error(e.to_string()))?,
        );

        let dt = res.data_type();
        let f = Field::new("interpolated_res", dt.clone(), true);
        let res = PyArray::new(res, Arc::new(f));
        Ok(res)
    }

    pub fn interpolate_extensive(&self, source_var: PyArray) -> PyResult<PyArray> {
        let d = source_var.array().into_data();
        let source_var = Float64Array::from(d);
        let res = Arc::new(
            self.0
                .interpolate_extensive(&source_var)
                .map_err(|e| new_error(e.to_string()))?,
        );

        let dt = res.data_type();
        let f = Field::new("interpolated_res", dt.clone(), true);
        let res = PyArray::new(res, Arc::new(f));
        Ok(res)
    }
}

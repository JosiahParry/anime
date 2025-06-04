use anime::Anime;
use arrow::{
    array::{Array, Float64Array},
    datatypes::Field,
};
use geoarrow::{
    array::{AsNativeArray, LineStringArray, NativeArrayDyn, WKBArray},
    datatypes::NativeType,
    io::wkb::from_wkb,
    trait_::ArrayAccessor,
    NativeArray,
};
use pyo3::prelude::*;
use pyo3::{exceptions::PyTypeError, PyErr, PyResult};
use pyo3_arrow::{PyArray, PyTable};
use std::sync::Arc;

fn new_error(msg: String) -> PyErr {
    PyErr::new::<PyTypeError, _>(msg)
}

pub fn as_geoarrow_lines(x: PyArray) -> PyResult<LineStringArray> {
    let (array, field) = x.into_inner();
    let nda = NativeArrayDyn::from_arrow_array(&array, &field);
    let nda = match nda {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e:?}");
            let wkb = WKBArray::<i32>::try_from(array.as_ref())
                .map_err(|_| new_error("failed to cast to wkb array".into()))?;

            let array = from_wkb(
                &wkb,
                NativeType::Geometry(geoarrow::array::CoordType::Separated),
                true,
            )
            .map_err(|_| new_error("Failed to convert from wkb".into()))?;

            let aa = NativeArrayDyn::try_from(array)
                .map_err(|_| new_error("Failed to convert wkb array to native array".into()))?;

            aa
        }
    };

    match nda.data_type() {
        NativeType::LineString(..) => {
            let aref = nda.as_ref();
            Ok(aref.as_line_string().to_owned())
        }
        _ => {
            return Err(new_error(format!(
                "Input must be LineString array not {:?}",
                nda.data_type()
            )))
        }
    }
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
        let source = as_geoarrow_lines(source)?;
        let target = as_geoarrow_lines(target)?;
        let res = Anime::new(
            source.iter_geo_values(),
            target.iter_geo_values(),
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

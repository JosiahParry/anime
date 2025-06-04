use anime::{interpolate::InterpolatedValue, Anime, MatchCandidate};
use arrow::{
    array::{Array, Float64Array, Int32Array},
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
        // create the schema
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("source_id", arrow::datatypes::DataType::Int32, false),
            arrow::datatypes::Field::new("target_id", arrow::datatypes::DataType::Int32, false),
            arrow::datatypes::Field::new("shared_len", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new(
                "target_weighted",
                arrow::datatypes::DataType::Float64,
                false,
            ),
            arrow::datatypes::Field::new(
                "source_weighted",
                arrow::datatypes::DataType::Float64,
                false,
            ),
        ]);

        let schema = Arc::new(schema);

        let inner = self
            .0
            .matches
            .get()
            .ok_or_else(|| new_error("Matches not yet instantiated.".to_string()))?;

        // count the resultant vector sizes
        let n: usize = inner.iter().map(|(_, eles)| eles.len() as usize).sum();

        // instantiate vectors to fill
        let mut source_idx_res = Int32Array::builder(n);
        let mut target_idx_res = Int32Array::builder(n);
        let mut shared_len_res = Float64Array::builder(n);
        let mut source_weighted_res = Float64Array::builder(n);
        let mut target_weighted_res = Float64Array::builder(n);

        for (target, items) in inner.iter() {
            let source_lens = &self.0.source_lens;
            let target_len = self.0.target_lens.get(*target as usize).unwrap();

            for MatchCandidate {
                source_index,
                shared_len,
            } in items.iter()
            {
                let source_len = *source_lens.get(*source_index).unwrap();
                let target_id = *target as i32;
                let source_id = *source_index as i32;
                let shared_len = shared_len;
                let source_weighted = shared_len / source_len;
                let target_weighted = shared_len / target_len;

                shared_len_res.append_value(*shared_len);
                source_idx_res.append_value(source_id);
                target_idx_res.append_value(target_id);
                source_weighted_res.append_value(source_weighted);
                target_weighted_res.append_value(target_weighted);
            }
        }

        let res = arrow::record_batch::RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(source_idx_res.finish()),
                Arc::new(target_idx_res.finish()),
                Arc::new(shared_len_res.finish()),
                Arc::new(source_weighted_res.finish()),
                Arc::new(target_weighted_res.finish()),
            ],
        )
        .unwrap();
        pyo3_arrow::PyTable::try_new(vec![res], schema.clone())
    }

    pub fn interpolate_intensive(&self, source_var: Vec<f64>) -> PyResult<PyArray> {
        let n = self.0.target_lens.len();
        let mut res_array = vec![0.0; n];
        match self.0.interpolate_intensive(&source_var) {
            Ok(r) => {
                for InterpolatedValue { target_id, value } in r {
                    res_array[target_id] = value;
                }
            }
            Err(e) => {
                return Err(new_error(e.to_string()));
            }
        };

        let res = Arc::new(Float64Array::from(res_array));
        let dt = res.data_type();
        let f = Field::new("interpolated_res", dt.clone(), true);
        let res = PyArray::new(res, Arc::new(f));
        Ok(res)
    }

    pub fn interpolate_extensive(&self, source_var: Vec<f64>) -> PyResult<PyArray> {
        let n = self.0.target_lens.len();
        let mut res_array = vec![0.0; n];
        match self.0.interpolate_extensive(&source_var) {
            Ok(r) => {
                for InterpolatedValue { target_id, value } in r {
                    res_array[target_id] = value;
                }
            }
            Err(e) => {
                return Err(new_error(e.to_string()));
            }
        };

        let res = Arc::new(Float64Array::from(res_array));
        let dt = res.data_type();
        let f = Field::new("interpolated_res", dt.clone(), true);
        let res = PyArray::new(res, Arc::new(f));
        Ok(res)
    }
}

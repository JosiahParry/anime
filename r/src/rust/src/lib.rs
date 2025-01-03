use anime::Anime;
use arrow::{
    array::{make_array, ArrayData},
    datatypes::Field,
    error::ArrowError,
};
use arrow_extendr::from::FromArrowRobj;
use extendr_api::prelude::*;
use geoarrow::{
    array::{LineStringArray, NativeArrayDyn},
    trait_::ArrayAccessor,
    ArrayBase,
};
pub type ErrGeoArrowRobj = ArrowError;

// wrapper functions around R functions to make converting from
// nanoarrow-geoarrow easier
pub fn infer_geoarrow_schema(robj: &Robj) -> Result<Robj> {
    R!("geoarrow::infer_geoarrow_schema")
        .expect("`geoarrow` must be installed")
        .as_function()
        .expect("`infer_geoarrow_schema()` must be available")
        .call(pairlist!(robj))
}

pub fn as_data_type(robj: &Robj) -> Result<Robj> {
    R!("arrow::as_data_type")
        .expect("`arrow` must be installed")
        .as_function()
        .expect("`as_data_type()` must be available")
        .call(pairlist!(robj))
}

pub fn new_field(robj: &Robj, name: &str) -> Result<Robj> {
    R!("arrow::field")
        .expect("`arrow` must be installed")
        .as_function()
        .expect("`new_field()` must be available")
        .call(pairlist!(name, robj))
}

fn read_geoarrow_r(robj: Robj) -> Result<NativeArrayDyn> {
    // extract datatype from R object
    let narrow_data_type = infer_geoarrow_schema(&robj).unwrap();
    let arrow_dt = as_data_type(&narrow_data_type).unwrap();

    // create and extract field
    let field = new_field(&arrow_dt, "geometry").unwrap();
    let field = Field::from_arrow_robj(&field).unwrap();

    // extract array data
    let x = make_array(ArrayData::from_arrow_robj(&robj).unwrap());

    // create geoarrow array
    let res = NativeArrayDyn::from_arrow_array(&x, &field).map_err(|e| e.to_string())?;
    Ok(res)
}

#[extendr]
fn init_anime(
    source: Robj,
    target: Robj,
    distance_tolerance: f64,
    angle_tolerance: f64,
) -> ExternalPtr<anime::Anime> {
    let source = read_geoarrow_r(source).unwrap();
    let target = read_geoarrow_r(target).unwrap();
    let source = source
        .as_any()
        .downcast_ref::<LineStringArray>()
        .unwrap()
        .clone();
    let target = target
        .as_any()
        .downcast_ref::<LineStringArray>()
        .unwrap()
        .clone();
    let mut anime = anime::Anime::load_geometries(
        source.iter_geo_values(),
        target.iter_geo_values(),
        distance_tolerance,
        angle_tolerance,
    );

    anime.find_matches().unwrap();

    let mut ptr = ExternalPtr::new(anime);
    ptr.set_class(["anime"]).unwrap();
    ptr
}

#[extendr]
fn anime_print_helper(x: ExternalPtr<Anime>) -> List {
    list!(
        source_fts = x.source_lens.len(),
        target_fts = x.target_lens.len(),
        angle_tolerance = x.angle_tolerance,
        distance_tolerance = x.distance_tolerance,
        n_matches = x.matches.get().unwrap().len()
    )
}

#[extendr]
fn interpolate_extensive_(source_var: &[f64], anime: ExternalPtr<Anime>) -> Doubles {
    let res = anime.interpolate_extensive(source_var);
    match res {
        Ok(r) => {
            let mut res = (0..anime.target_lens.len())
                .into_iter()
                .map(|_| Rfloat::na())
                .collect::<Doubles>();
            for v in r {
                res.set_elt(v.target_id, Rfloat::from(v.value));
            }
            res
        }

        Err(e) => throw_r_error(format!(
            "Failed to perform extensive interpolation: {:?}",
            e.to_string()
        )),
    }
}

#[extendr]
fn interpolate_intensive_(source_var: &[f64], anime: ExternalPtr<Anime>) -> Doubles {
    let res = anime.interpolate_intensive(source_var);
    match res {
        Ok(r) => {
            let mut res = (0..anime.target_lens.len())
                .into_iter()
                .map(|_| Rfloat::na())
                .collect::<Doubles>();
            for v in r {
                res.set_elt(v.target_id, Rfloat::from(v.value));
            }
            res
        }
        Err(e) => throw_r_error(format!(
            "Failed to perform extensive interpolation: {:?}",
            e.to_string()
        )),
    }
}

#[derive(IntoDataFrameRow)]
struct MatchRow {
    target_id: i32,
    source_id: i32,
    shared_len: f64,
    source_weighted: f64,
    target_weighted: f64,
}

#[extendr]
fn get_matches_(anime: ExternalPtr<Anime>) -> Robj {
    let inner = anime.matches.get().unwrap();
    let all_items = inner
        .into_iter()
        .flat_map(|(idx, cands)| {
            let source_lens = &anime.source_lens;
            let target_len = anime.target_lens.get(*idx as usize).unwrap();

            cands.into_iter().map(move |ci| {
                let source_len = source_lens.get(ci.source_index).unwrap();

                MatchRow {
                    target_id: *idx as i32,
                    source_id: ci.source_index as i32,
                    shared_len: ci.shared_len,
                    source_weighted: ci.shared_len / source_len,
                    target_weighted: ci.shared_len / target_len,
                }
            })
        })
        .collect::<Vec<_>>();
    let df = Dataframe::try_from_values(all_items).unwrap();
    df.into()
}

// Macro to generate exports.
// This ensures exported functions are registered with R.
// See corresponding C code in `entrypoint.c`.
extendr_module! {
    mod anime;
    fn init_anime;
    fn interpolate_extensive_;
    fn interpolate_intensive_;
    fn get_matches_;
    fn anime_print_helper;
}

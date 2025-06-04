use crate::{Anime, AnimeError, MatchCandidate};
use arrow::array::{Float64Array, Int32Array, RecordBatch};
use std::sync::Arc;

impl Anime {
    pub fn get_matches(&self) -> Result<RecordBatch, AnimeError> {
        // create the schema
        let schema = arrow::datatypes::Schema::new(vec![
            arrow::datatypes::Field::new("source_id", arrow::datatypes::DataType::Int32, false),
            arrow::datatypes::Field::new("target_id", arrow::datatypes::DataType::Int32, false),
            arrow::datatypes::Field::new("shared_len", arrow::datatypes::DataType::Float64, false),
            arrow::datatypes::Field::new(
                "source_weighted",
                arrow::datatypes::DataType::Float64,
                false,
            ),
            arrow::datatypes::Field::new(
                "target_weighted",
                arrow::datatypes::DataType::Float64,
                false,
            ),
        ]);

        let schema = Arc::new(schema);

        let inner = self
            .matches
            .get()
            .ok_or_else(|| AnimeError::MatchesNotFound)?;

        // count the resultant vector sizes
        let n: usize = inner.iter().map(|(_, eles)| eles.len() as usize).sum();

        // instantiate vectors to fill
        let mut source_idx_res = Int32Array::builder(n);
        let mut target_idx_res = Int32Array::builder(n);
        let mut shared_len_res = Float64Array::builder(n);
        let mut source_weighted_res = Float64Array::builder(n);
        let mut target_weighted_res = Float64Array::builder(n);

        for (target, items) in inner.iter() {
            let source_lens = &self.source_lens;
            let target_len = self.target_lens.get(*target as usize).unwrap();

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
        .expect("All arrays should be identical lengths");
        Ok(res)
    }
}

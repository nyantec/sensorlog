use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct StoreMeasurementRequest {
	time: Option<u64>,
	sensor_id: String,
	data: Vec<u8>,
}

pub struct StoreMeasurementResponse {}

pub fn store_measurement(req: &StoreMeasurementRequest) -> Result<StoreMeasurementResponse, ::Error> {
	return Ok(StoreMeasurementResponse{});
}


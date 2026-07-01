use crate::generate::generate_image::image_generation_cost_estimate::ImageGenerationCostEstimate;
use crate::generate::generate_image::providers::runninghub::nano_banana_2::request::RunninghubNanoBanana2RequestState;

pub struct RunninghubNanoBanana2CostState {
  cost_in_usd_cents: u64,
}

impl RunninghubNanoBanana2CostState {
  pub fn from_request(_request: &RunninghubNanoBanana2RequestState) -> Self {
    Self { cost_in_usd_cents: 8 }
  }

  pub fn estimate_cost(&self) -> ImageGenerationCostEstimate {
    ImageGenerationCostEstimate {
      cost_in_credits: Some(self.cost_in_usd_cents),
      cost_in_usd_cents: Some(self.cost_in_usd_cents),
      is_free: false,
      is_unlimited: false,
      is_rate_limited: false,
      has_watermark: false,
      failures_are_refunded: None,
    }
  }
}

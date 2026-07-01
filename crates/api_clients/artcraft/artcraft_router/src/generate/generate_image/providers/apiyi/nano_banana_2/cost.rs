use crate::generate::generate_image::image_generation_cost_estimate::ImageGenerationCostEstimate;
use crate::generate::generate_image::providers::apiyi::nano_banana_2::request::ApiyiNanaBanana2RequestState;

pub struct ApiyiNanaBanana2CostState {
  cost_in_usd_cents: u64,
}

impl ApiyiNanaBanana2CostState {
  pub fn from_request(_request: &ApiyiNanaBanana2RequestState) -> Self {
    Self { cost_in_usd_cents: 6 }
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

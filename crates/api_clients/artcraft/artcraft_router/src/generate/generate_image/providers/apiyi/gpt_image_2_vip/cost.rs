use crate::generate::generate_image::image_generation_cost_estimate::ImageGenerationCostEstimate;
use crate::generate::generate_image::providers::apiyi::gpt_image_2_vip::request::ApiyiGptImage2VipRequestState;

pub struct ApiyiGptImage2VipCostState {
  cost_in_usd_cents: u64,
}

impl ApiyiGptImage2VipCostState {
  pub fn from_request(_request: &ApiyiGptImage2VipRequestState) -> Self {
    Self { cost_in_usd_cents: 3 }
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

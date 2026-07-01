use crate::api::router_resolution::RouterResolution;

/// Maps a router resolution to the `"1k"`/`"2k"`/`"4k"` string RunningHub's
/// Nano Banana 2 and GPT Image 2.0 image endpoints expect.
///
/// Variants with no exact RunningHub tier (e.g. video-native `FourEightyP`)
/// snap to the nearest tier rather than being rejected outright.
pub fn plan_runninghub_resolution(res: Option<RouterResolution>) -> Option<String> {
  let s = match res? {
    RouterResolution::HalfK => "1k",
    RouterResolution::OneK => "1k",
    RouterResolution::TwoK => "2k",
    RouterResolution::ThreeK => "2k",
    RouterResolution::FourK => "4k",
    RouterResolution::FourEightyP => "1k",
    RouterResolution::SevenTwentyP => "1k",
    RouterResolution::TenEightyP => "2k",
  };
  Some(s.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn none_input_yields_none() {
    assert_eq!(plan_runninghub_resolution(None), None);
  }

  #[test]
  fn one_k_maps_through() {
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::OneK)), Some("1k".to_string()));
  }

  #[test]
  fn two_k_maps_through() {
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::TwoK)), Some("2k".to_string()));
  }

  #[test]
  fn four_k_maps_through() {
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::FourK)), Some("4k".to_string()));
  }

  #[test]
  fn half_k_snaps_to_one_k() {
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::HalfK)), Some("1k".to_string()));
  }

  #[test]
  fn video_native_resolutions_snap_to_nearest_tier() {
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::FourEightyP)), Some("1k".to_string()));
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::SevenTwentyP)), Some("1k".to_string()));
    assert_eq!(plan_runninghub_resolution(Some(RouterResolution::TenEightyP)), Some("2k".to_string()));
  }
}

use crate::api::router_aspect_ratio::RouterAspectRatio;

/// Maps a router aspect ratio to the `"W:H"` string RunningHub's image and
/// video endpoints expect (e.g. `"16:9"`). Shared by RunningHub's image
/// build path and the desktop Grok Video handler so both agree on the same
/// wire format.
///
/// `Auto*` variants bake resolution into the ratio (Seedream-specific) and
/// have no RunningHub equivalent, so they resolve to `None` (RunningHub uses
/// its own default).
pub fn plan_runninghub_aspect_ratio(ar: Option<RouterAspectRatio>) -> Option<String> {
  let s = match ar? {
    RouterAspectRatio::Auto => return None,
    RouterAspectRatio::Square | RouterAspectRatio::SquareHd => "1:1",
    RouterAspectRatio::WideThreeByTwo | RouterAspectRatio::Wide => "3:2",
    RouterAspectRatio::WideFourByThree => "4:3",
    RouterAspectRatio::WideFiveByFour => "5:4",
    RouterAspectRatio::WideSixteenByNine => "16:9",
    RouterAspectRatio::WideTwentyOneByNine => "21:9",
    RouterAspectRatio::TallTwoByThree | RouterAspectRatio::Tall => "2:3",
    RouterAspectRatio::TallThreeByFour => "3:4",
    RouterAspectRatio::TallFourByFive => "4:5",
    RouterAspectRatio::TallNineBySixteen => "9:16",
    RouterAspectRatio::TallNineByTwentyOne => "9:21",
    RouterAspectRatio::Auto2k | RouterAspectRatio::Auto3k | RouterAspectRatio::Auto4k => return None,
  };
  Some(s.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn none_input_yields_none() {
    assert_eq!(plan_runninghub_aspect_ratio(None), None);
  }

  #[test]
  fn auto_yields_none() {
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::Auto)), None);
  }

  #[test]
  fn square_and_square_hd_map_to_one_by_one() {
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::Square)), Some("1:1".to_string()));
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::SquareHd)), Some("1:1".to_string()));
  }

  #[test]
  fn wide_sixteen_by_nine_maps_through() {
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::WideSixteenByNine)), Some("16:9".to_string()));
  }

  #[test]
  fn tall_nine_by_sixteen_maps_through() {
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::TallNineBySixteen)), Some("9:16".to_string()));
  }

  #[test]
  fn seedream_auto_variants_yield_none() {
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::Auto2k)), None);
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::Auto3k)), None);
    assert_eq!(plan_runninghub_aspect_ratio(Some(RouterAspectRatio::Auto4k)), None);
  }
}

//! Client-side checks shared by `text_to_image` and `image_to_image`.
//!
//! Only documented, model-specific server rejections are reproduced here; the
//! numeric limits mirror the Tripo text-to-image / image-to-image docs. An
//! unset model, or one this SDK does not recognize, skips these checks so the
//! server decides and newly added models keep working.

use crate::enums::{ImageBackground, ImageOutputFormat, ImageQuality};
use crate::error::{Error, Result};
use crate::versions::image as models;

/// Model families that share parameter rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    Seedream,
    Banana,
    /// `chat_image_1` / `chat_image_1.5`.
    ChatImageLegacy,
    /// `chat_image_2`.
    ChatImage2,
    /// `chat_image_2.5_flare` / `chat_image_2.5_sunburst`.
    ChatImage25,
}

#[allow(deprecated)] // Matching the retiring models is how they get classified.
fn family(model: &str) -> Option<Family> {
    Some(match model {
        models::SEEDREAM_V5 | models::SEEDREAM_V4 => Family::Seedream,
        models::BANANA | models::BANANA_PRO | models::BANANA2 => Family::Banana,
        models::CHAT_IMAGE_1 | models::CHAT_IMAGE_1_5 => Family::ChatImageLegacy,
        models::CHAT_IMAGE_2 => Family::ChatImage2,
        models::CHAT_IMAGE_2_5_FLARE | models::CHAT_IMAGE_2_5_SUNBURST => Family::ChatImage25,
        _ => return None,
    })
}

/// Parameters shared by both image-generation endpoints.
pub(crate) struct ImageParams<'a> {
    /// Requested model. `None` (server default) skips model-specific checks.
    pub model: Option<&'a str>,
    /// Number of reference images; 0 for text-to-image.
    pub inputs: usize,
    pub size: Option<&'a str>,
    pub quality: Option<ImageQuality>,
    pub background: Option<ImageBackground>,
    pub output_format: Option<ImageOutputFormat>,
}

pub(crate) fn validate(p: &ImageParams<'_>) -> Result<()> {
    // Unset and unrecognized models are left to the server.
    let Some(model) = p.model else {
        return Ok(());
    };
    let Some(family) = family(model) else {
        return Ok(());
    };
    let max_inputs = match family {
        Family::Seedream => 4,
        Family::Banana => 10,
        Family::ChatImageLegacy | Family::ChatImage2 | Family::ChatImage25 => 16,
    };
    if p.inputs > max_inputs {
        return Err(Error::InvalidRequest(format!(
            "model {model} accepts at most {max_inputs} inputs, got {}",
            p.inputs
        )));
    }
    match (family, p.quality) {
        (Family::ChatImage2, Some(ImageQuality::Xhigh | ImageQuality::Max)) => {
            return Err(Error::InvalidRequest(format!(
                "model {model} accepts quality low, medium, or high; xhigh and max need a chat_image_2.5 model"
            )));
        }
        (Family::Seedream | Family::Banana | Family::ChatImageLegacy, Some(_)) => {
            return Err(Error::InvalidRequest(format!(
                "model {model} does not accept quality; use {}, {}, or {}",
                models::CHAT_IMAGE_2,
                models::CHAT_IMAGE_2_5_FLARE,
                models::CHAT_IMAGE_2_5_SUNBURST,
            )));
        }
        _ => {}
    }
    // Other models ignore `background`, so only the 2.5 models reject this.
    if family == Family::ChatImage25
        && p.background == Some(ImageBackground::Transparent)
        && p.output_format == Some(ImageOutputFormat::Jpeg)
    {
        return Err(Error::InvalidRequest(
            "background `transparent` requires output_format `png`; JPEG has no alpha channel"
                .into(),
        ));
    }
    if let (Some(size), Family::ChatImage2 | Family::ChatImage25) = (p.size, family) {
        validate_custom_size(model, size)?;
    }
    Ok(())
}

/// Custom `WIDTHxHEIGHT` limits for `chat_image_2` and the 2.5 models.
/// Keywords (`2K`, `4K`) and anything else that is not `WxH` pass through.
///
/// Every `WxH` string is checked, presets included. This assumes the model
/// presets (`1024x1024`, `1536x1024`, `1024x1536`) satisfy the custom-size
/// limits, which they do.
fn validate_custom_size(model: &str, size: &str) -> Result<()> {
    let Some((w, h)) = size
        .split_once('x')
        .and_then(|(w, h)| Some((w.parse::<u64>().ok()?, h.parse::<u64>().ok()?)))
    else {
        return Ok(());
    };
    let (short, long) = (w.min(h), w.max(h));
    let ok = long <= 3840
        && w % 16 == 0
        && h % 16 == 0
        && long <= 3 * short
        // Both edges are at most 3840 here, so the product cannot overflow.
        && (655_360..=8_294_400).contains(&(w * h));
    if ok {
        Ok(())
    } else {
        Err(Error::InvalidRequest(format!(
            "model {model} rejects size {size}: each edge must be a multiple of 16 and at most 3840, \
             the long edge at most 3x the short edge, and total pixels between 655360 and 8294400"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(model: &str) -> ImageParams<'_> {
        ImageParams {
            model: Some(model),
            inputs: 0,
            size: None,
            quality: None,
            background: None,
            output_format: None,
        }
    }

    fn err_of(p: &ImageParams<'_>) -> String {
        validate(p).unwrap_err().to_string()
    }

    #[test]
    fn quality_tiers_per_model() {
        let all = [
            ImageQuality::Low,
            ImageQuality::Medium,
            ImageQuality::High,
            ImageQuality::Xhigh,
            ImageQuality::Max,
        ];
        for model in [
            models::CHAT_IMAGE_2_5_FLARE,
            models::CHAT_IMAGE_2_5_SUNBURST,
        ] {
            for q in all {
                validate(&ImageParams {
                    quality: Some(q),
                    ..params(model)
                })
                .unwrap();
            }
        }
        for (q, ok) in all.into_iter().zip([true, true, true, false, false]) {
            let p = ImageParams {
                quality: Some(q),
                ..params(models::CHAT_IMAGE_2)
            };
            assert_eq!(validate(&p).is_ok(), ok, "{q:?}");
        }
    }

    #[test]
    #[allow(deprecated)]
    fn quality_rejected_for_models_without_tiers() {
        for model in [
            models::SEEDREAM_V5,
            models::SEEDREAM_V4,
            models::BANANA,
            models::BANANA_PRO,
            models::BANANA2,
            models::CHAT_IMAGE_1,
            models::CHAT_IMAGE_1_5,
        ] {
            let p = ImageParams {
                quality: Some(ImageQuality::Low),
                ..params(model)
            };
            assert!(err_of(&p).contains("does not accept quality"), "{model}");
        }
    }

    #[test]
    fn unset_or_unknown_model_passes_through() {
        for model in [None, Some("chat_image_3")] {
            validate(&ImageParams {
                model,
                inputs: 100,
                size: Some("1x1"),
                quality: Some(ImageQuality::Max),
                background: Some(ImageBackground::Transparent),
                output_format: Some(ImageOutputFormat::Jpeg),
            })
            .unwrap();
        }
    }

    #[test]
    fn transparent_background_requires_png() {
        let model = models::CHAT_IMAGE_2_5_FLARE;
        let bg = Some(ImageBackground::Transparent);
        assert!(
            err_of(&ImageParams {
                background: bg,
                output_format: Some(ImageOutputFormat::Jpeg),
                ..params(model)
            })
            .contains("png")
        );
        for output_format in [None, Some(ImageOutputFormat::Png)] {
            validate(&ImageParams {
                background: bg,
                output_format,
                ..params(model)
            })
            .unwrap();
        }
        // Opaque JPEG is fine.
        validate(&ImageParams {
            background: Some(ImageBackground::Opaque),
            output_format: Some(ImageOutputFormat::Jpeg),
            ..params(model)
        })
        .unwrap();
        // Other models ignore background server-side, so transparent + JPEG passes.
        for model in [
            models::SEEDREAM_V4,
            models::BANANA,
            models::CHAT_IMAGE_2,
            "chat_image_3",
        ] {
            validate(&ImageParams {
                background: bg,
                output_format: Some(ImageOutputFormat::Jpeg),
                ..params(model)
            })
            .unwrap();
        }
    }

    #[test]
    fn custom_sizes_for_chat_image_2_family() {
        for size in [
            "1024x1024",
            "3840x2160",
            "2160x3840",
            "2048x1024",
            "2K",
            "4K",
        ] {
            for model in [models::CHAT_IMAGE_2, models::CHAT_IMAGE_2_5_SUNBURST] {
                validate(&ImageParams {
                    size: Some(size),
                    ..params(model)
                })
                .unwrap();
            }
        }
        for size in [
            "3856x2160",             // edge > 3840
            "1000x1008",             // not a multiple of 16
            "3840x1264",             // long edge > 3x short edge
            "800x800",               // < 655360 pixels
            "3840x2176",             // > 8294400 pixels
            "4294967296x4294967296", // product overflows u64
        ] {
            let p = ImageParams {
                size: Some(size),
                ..params(models::CHAT_IMAGE_2_5_FLARE)
            };
            assert!(err_of(&p).contains(size), "{size}");
        }
        // Other models take their own preset sizes.
        validate(&ImageParams {
            size: Some("800x800"),
            ..params(models::BANANA)
        })
        .unwrap();
    }

    #[test]
    fn inputs_count_limits() {
        for (model, max) in [
            (models::SEEDREAM_V5, 4),
            (models::BANANA2, 10),
            (models::CHAT_IMAGE_2, 16),
        ] {
            validate(&ImageParams {
                inputs: max,
                ..params(model)
            })
            .unwrap();
            let msg = err_of(&ImageParams {
                inputs: max + 1,
                ..params(model)
            });
            assert!(msg.contains(&format!("at most {max}")), "{msg}");
        }
    }
}

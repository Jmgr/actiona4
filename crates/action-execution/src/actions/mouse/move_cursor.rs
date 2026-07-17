use action_definition::{
    actions::mouse::move_cursor::{MoveCursor, Tween},
    parameters::ParameterKind,
    post_run::PostRun,
};
use actiona_core::api::mouse::{MoveOptions, Tween as CoreTween};

use crate::{
    ExecutionContext, ResolveParam, RunError, Runnable,
    resolve_param::{ScriptableParamValue, ValidateParamValue, ValidationError},
};

const fn to_core_tween(tween: Tween) -> CoreTween {
    match tween {
        Tween::BackIn => CoreTween::BackIn,
        Tween::BackInOut => CoreTween::BackInOut,
        Tween::BackOut => CoreTween::BackOut,
        Tween::BounceIn => CoreTween::BounceIn,
        Tween::BounceInOut => CoreTween::BounceInOut,
        Tween::BounceOut => CoreTween::BounceOut,
        Tween::CircIn => CoreTween::CircIn,
        Tween::CircInOut => CoreTween::CircInOut,
        Tween::CircOut => CoreTween::CircOut,
        Tween::CubicIn => CoreTween::CubicIn,
        Tween::CubicInOut => CoreTween::CubicInOut,
        Tween::CubicOut => CoreTween::CubicOut,
        Tween::ElasticIn => CoreTween::ElasticIn,
        Tween::ElasticInOut => CoreTween::ElasticInOut,
        Tween::ElasticOut => CoreTween::ElasticOut,
        Tween::ExpoIn => CoreTween::ExpoIn,
        Tween::ExpoInOut => CoreTween::ExpoInOut,
        Tween::ExpoOut => CoreTween::ExpoOut,
        Tween::Linear => CoreTween::Linear,
        Tween::QuadIn => CoreTween::QuadIn,
        Tween::QuadInOut => CoreTween::QuadInOut,
        Tween::QuadOut => CoreTween::QuadOut,
        Tween::QuartIn => CoreTween::QuartIn,
        Tween::QuartInOut => CoreTween::QuartInOut,
        Tween::QuartOut => CoreTween::QuartOut,
        Tween::QuintIn => CoreTween::QuintIn,
        Tween::QuintInOut => CoreTween::QuintInOut,
        Tween::QuintOut => CoreTween::QuintOut,
        Tween::SineIn => CoreTween::SineIn,
        Tween::SineInOut => CoreTween::SineInOut,
        Tween::SineOut => CoreTween::SineOut,
    }
}

const fn from_core_tween(tween: CoreTween) -> Tween {
    match tween {
        CoreTween::BackIn => Tween::BackIn,
        CoreTween::BackInOut => Tween::BackInOut,
        CoreTween::BackOut => Tween::BackOut,
        CoreTween::BounceIn => Tween::BounceIn,
        CoreTween::BounceInOut => Tween::BounceInOut,
        CoreTween::BounceOut => Tween::BounceOut,
        CoreTween::CircIn => Tween::CircIn,
        CoreTween::CircInOut => Tween::CircInOut,
        CoreTween::CircOut => Tween::CircOut,
        CoreTween::CubicIn => Tween::CubicIn,
        CoreTween::CubicInOut => Tween::CubicInOut,
        CoreTween::CubicOut => Tween::CubicOut,
        CoreTween::ElasticIn => Tween::ElasticIn,
        CoreTween::ElasticInOut => Tween::ElasticInOut,
        CoreTween::ElasticOut => Tween::ElasticOut,
        CoreTween::ExpoIn => Tween::ExpoIn,
        CoreTween::ExpoInOut => Tween::ExpoInOut,
        CoreTween::ExpoOut => Tween::ExpoOut,
        CoreTween::Linear => Tween::Linear,
        CoreTween::QuadIn => Tween::QuadIn,
        CoreTween::QuadInOut => Tween::QuadInOut,
        CoreTween::QuadOut => Tween::QuadOut,
        CoreTween::QuartIn => Tween::QuartIn,
        CoreTween::QuartInOut => Tween::QuartInOut,
        CoreTween::QuartOut => Tween::QuartOut,
        CoreTween::QuintIn => Tween::QuintIn,
        CoreTween::QuintInOut => Tween::QuintInOut,
        CoreTween::QuintOut => Tween::QuintOut,
        CoreTween::SineIn => Tween::SineIn,
        CoreTween::SineInOut => Tween::SineInOut,
        CoreTween::SineOut => Tween::SineOut,
    }
}

impl ScriptableParamValue for Tween {
    type ScriptValue = CoreTween;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        from_core_tween(value)
    }
}

impl ValidateParamValue for Tween {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl Runnable for MoveCursor {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let position = self.position.resolve(context).await?;
        let speed = self.speed.resolve(context).await?;
        let tween = self.tween.resolve(context).await?;
        let perlin_scale = self.perlin_scale.resolve(context).await?;
        let perlin_amplitude = self.perlin_amplitude.resolve(context).await?;
        let target_randomness = self.target_randomness.resolve(context).await?;
        let interval = self.interval.resolve(context).await?;

        let mut options = MoveOptions::default();

        if let Some(speed) = speed {
            options.speed = speed;
        }

        if let Some(tween) = tween {
            options.tween = to_core_tween(tween);
        }

        if let Some(perlin_scale) = perlin_scale {
            options.perlin_scale = perlin_scale;
        }

        if let Some(perlin_amplitude) = perlin_amplitude {
            options.perlin_amplitude = perlin_amplitude;
        }

        if let Some(target_randomness) = target_randomness {
            options.target_randomness = target_randomness;
        }

        if let Some(interval) = interval {
            options.interval = (*interval).into();
        }

        let mouse = context.runtime.mouse()?;
        mouse
            .move_(
                position,
                context.cancellation_token.clone(),
                options,
                context.runtime.rng(),
            )
            .await?;

        Ok(PostRun::default())
    }
}

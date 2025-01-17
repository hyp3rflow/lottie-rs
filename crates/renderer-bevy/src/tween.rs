use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::Transform;
use bevy_tweening::{Delay, EaseMethod, Lens, Sequence, Tween, TweeningType};
use flo_curves::bezier::{curve_intersects_line, Curve};
use flo_curves::{BezierCurveFactory, Coord2};
use lottie_core::{KeyFrame, Transform as LottieTransform};

use crate::lens::TransformLens;

pub(crate) trait TweenProducer<T, L>
where
    L: Lens<T> + Send + Sync + 'static,
    T: 'static,
{
    type Key;
    fn tween<F>(&self, frame_rate: f32, producer: F) -> Sequence<T>
    where
        F: Fn(Self::Key, Self::Key) -> L;
}

impl<L, T, V> TweenProducer<T, L> for Vec<KeyFrame<V>>
where
    L: Lens<T> + Send + Sync + 'static,
    T: 'static,
    V: Clone,
{
    type Key = V;
    fn tween<F>(&self, frame_rate: f32, producer: F) -> Sequence<T>
    where
        F: Fn(Self::Key, Self::Key) -> L,
    {
        let mut seq = Sequence::with_capacity(self.len() + 1);
        if self[0].start_frame.is_sign_positive() {
            seq = seq.then(Delay::new(Duration::from_secs_f32(
                self[0].start_frame / (frame_rate as f32),
            )));
        }
        for k in self.iter() {
            let start = k.start_value.clone();
            let end = k.end_value.clone();
            let ease_out = k.easing_out.clone().unwrap();
            let ease_in = k.easing_in.clone().unwrap();
            let frames = k.end_frame - k.start_frame;
            let secs = frames as f32 / frame_rate as f32;
            debug_assert!(secs > 0.0);
            let curve = Curve::from_points(
                Coord2(0.0, 0.0),
                (
                    Coord2(ease_out.x[0] as f64, ease_out.y[0] as f64),
                    Coord2(ease_in.x[0] as f64, ease_in.y[0] as f64),
                ),
                Coord2(1.0, 1.0),
            );
            let t = Tween::new(
                EaseMethod::CustomFunction(Arc::new(move |x| {
                    let intersection = curve_intersects_line(
                        &curve,
                        &(Coord2(x as f64, 0.0), Coord2(x as f64, 1.0)),
                    );
                    if intersection.is_empty() {
                        x
                    } else {
                        intersection[0].2 .1 as f32
                    }
                })),
                TweeningType::Once,
                Duration::from_secs_f32(secs),
                producer(start, end),
            );
            seq = seq.then(t);
        }
        seq
    }
}

impl TweenProducer<Transform, TransformLens> for LottieTransform {
    type Key = LottieTransform;

    fn tween<F>(&self, frame_rate: f32, producer: F) -> Sequence<Transform>
    where
        F: Fn(Self::Key, Self::Key) -> TransformLens,
    {
        let frames = self.frames();
        let secs = frames as f32 / frame_rate as f32;
        let mut transform = producer(self.clone(), self.clone());
        transform.frames = frames;
        let tween = Tween::new(
            EaseMethod::Linear,
            TweeningType::Once,
            Duration::from_secs_f32(secs),
            transform,
        );
        Sequence::from_single(tween)
    }
}

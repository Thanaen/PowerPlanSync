//! Durées et courbes du mouvement, reprises de `theme::motion` de Bientôt
//! (`crates/bientot-app/src/theme/motion.rs`, licence MIT). `with_animation` et `with_spring`
//! montrent leur état final quand `App::reduce_motion` est actif ; [`Entrance`] aussi.

use std::time::{Duration, Instant};

use gpui::{App, Pixels, Styled, Window};

/// Durée d'un type d'animation. En debug, `POWERPLANSYNC_MOTION_SLOWDOWN` la multiplie.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timing(Duration);

pub const PRESS: Timing = Timing(Duration::from_millis(140));
pub const TOOLTIP: Timing = Timing(Duration::from_millis(150));
pub const MENU: Timing = Timing(Duration::from_millis(200));
pub const DIALOG: Timing = Timing(Duration::from_millis(250));

impl Timing {
    pub fn duration(self) -> Duration {
        scaled(self.0, slowdown())
    }
}

#[cfg(debug_assertions)]
fn slowdown() -> f64 {
    static SLOWDOWN: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *SLOWDOWN.get_or_init(|| {
        std::env::var("POWERPLANSYNC_MOTION_SLOWDOWN")
            .ok()
            .and_then(|value| parse_slowdown(&value))
            .unwrap_or(1.)
    })
}

#[cfg(not(debug_assertions))]
fn slowdown() -> f64 {
    1.
}

/// Un facteur fini et positif, borné à 0,01..=100.
#[cfg(debug_assertions)]
fn parse_slowdown(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|factor| factor.is_finite() && *factor > 0.)
        .map(|factor| factor.clamp(0.01, 100.))
}

fn scaled(duration: Duration, factor: f64) -> Duration {
    Duration::try_from_secs_f64(duration.as_secs_f64() * factor).unwrap_or(Duration::MAX)
}

/// Avancement d'une entrée depuis sa première image, sur l'horloge de l'exécuteur, que les
/// tests peuvent avancer.
#[derive(Debug, Default)]
pub struct Entrance(Phase);

#[derive(Clone, Copy, Debug, Default)]
enum Phase {
    #[default]
    Hidden,
    Coming(Instant),
    /// Jusqu'à `reset` : réactiver le mouvement pendant l'affichage ne rejoue pas l'entrée.
    AtRest,
}

impl Entrance {
    /// Pour ce qui est affiché dès le départ : l'entrée jouera après le prochain `reset`.
    pub fn at_rest() -> Self {
        Self(Phase::AtRest)
    }

    /// Part de `timing` écoulée, linéaire de 0 à 1, en demandant l'image suivante d'ici là.
    pub fn time(&mut self, timing: Timing, window: &mut Window, cx: &App) -> f32 {
        let now = cx.background_executor().now();
        let shown = match self.0 {
            Phase::Hidden => now,
            Phase::Coming(shown) => shown,
            Phase::AtRest => return 1.,
        };
        match share(
            now.saturating_duration_since(shown),
            timing.duration(),
            cx.reduce_motion(),
        ) {
            Some(time) => {
                self.0 = Phase::Coming(shown);
                window.request_animation_frame();
                time
            }
            None => {
                self.0 = Phase::AtRest;
                1.
            }
        }
    }

    /// Pour ce qui n'est plus affiché : l'entrée rejouera la prochaine fois.
    pub fn reset(&mut self) {
        self.0 = Phase::Hidden;
    }
}

/// Part écoulée pendant l'entrée, `None` une fois au repos ou en mouvement réduit.
fn share(elapsed: Duration, duration: Duration, reduce_motion: bool) -> Option<f32> {
    (!reduce_motion && elapsed < duration).then(|| elapsed.as_secs_f32() / duration.as_secs_f32())
}

/// Part de l'entrée à laquelle ce qui passe au-dessus du contenu devient opaque : une image à
/// 60 Hz vaut 8 % d'un menu et 7 % d'un dialogue.
pub const OPAQUE_AT: f32 = 0.1;

/// Pose d'une image : opacité et décalage par rapport à la place finale.
#[derive(Debug, PartialEq)]
pub struct Entering {
    pub opacity: f32,
    pub offset: Pixels,
}

impl Entering {
    /// Décale `element` vers le bas sans déplacer ses voisins, et l'estompe.
    pub fn apply<E: Styled>(self, element: E) -> E {
        element.relative().top(self.offset).opacity(self.opacity)
    }
}

/// Entrée dans le flux : fondu sur toute l'entrée, glissement depuis `from`.
pub fn slide_in(time: f32, from: Pixels) -> Entering {
    entering(ease_out()(time), time, from)
}

/// Entrée au-dessus du contenu : même glissement, opaque dès [`OPAQUE_AT`].
pub fn slide_over(time: f32, from: Pixels) -> Entering {
    entering(ease_out()((time / OPAQUE_AT).min(1.)), time, from)
}

fn entering(opacity: f32, time: f32, from: Pixels) -> Entering {
    Entering {
        opacity,
        offset: from * (1. - ease_out()(time)),
    }
}

/// Pour ce qui entre ou sort.
pub fn ease_out() -> impl Fn(f32) -> f32 {
    cubic_bezier(0.23, 1., 0.32, 1.)
}

/// Pour ce qui se déplace à l'écran.
pub fn ease_in_out() -> impl Fn(f32) -> f32 {
    cubic_bezier(0.77, 0., 0.175, 1.)
}

/// `cubic-bezier(x1, y1, x2, y2)` de CSS, avec `x1` et `x2` dans 0..=1.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> impl Fn(f32) -> f32 {
    debug_assert!(
        (0. ..=1.).contains(&x1) && (0. ..=1.).contains(&x2),
        "cubic-bezier x control points lie within 0..=1"
    );
    let axis = |p1: f32, p2: f32, t: f32| {
        let u = 1. - t;
        3. * u * u * t * p1 + 3. * u * t * t * p2 + t * t * t
    };
    move |x| {
        // Extrémités exactes : la dernière image et le mouvement réduit tombent au repos.
        if x <= 0. {
            return 0.;
        }
        if x >= 1. {
            return 1.;
        }
        // L'axe x est croissant : 24 dichotomies suffisent, bien sous le pixel.
        let (mut low, mut high) = (0_f32, 1_f32);
        for _ in 0..24 {
            let middle = (low + high) / 2.;
            if axis(x1, x2, middle) < x {
                low = middle;
            } else {
                high = middle;
            }
        }
        axis(y1, y2, (low + high) / 2.)
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-4,
            "{actual} is not {expected}"
        );
    }

    #[test]
    fn curves_start_and_end_exactly_at_rest() {
        for curve in [
            Box::new(ease_out()) as Box<dyn Fn(f32) -> f32>,
            Box::new(ease_in_out()),
        ] {
            assert_eq!(curve(0.), 0.);
            assert_eq!(curve(1.), 1.);
        }
    }

    #[test]
    fn curves_follow_their_control_points() {
        let ease_out = ease_out();
        assert_close(ease_out(0.25), 0.775382);
        assert_close(ease_out(0.5), 0.965983);
        let ease_in_out = ease_in_out();
        assert_close(ease_in_out(0.25), 0.052889);
        assert_close(ease_in_out(0.5), 0.595971);
        assert_close(cubic_bezier(0.25, 0.1, 0.25, 1.)(0.5), 0.802403);
    }

    #[test]
    fn slowdown_stretches_a_duration() {
        assert_eq!(
            scaled(Duration::from_millis(200), 4.),
            Duration::from_millis(800)
        );
        assert_eq!(scaled(Duration::from_millis(200), 1e300), Duration::MAX);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn slowdown_is_a_finite_factor_above_zero() {
        assert_eq!(parse_slowdown(" 2.5 "), Some(2.5));
        assert_eq!(parse_slowdown("1e9"), Some(100.));
        assert_eq!(parse_slowdown("1e-9"), Some(0.01));
        for invalid in ["", "fast", "0", "-2", "inf", "NaN", "4x"] {
            assert_eq!(parse_slowdown(invalid), None, "{invalid:?}");
        }
    }

    #[test]
    fn tokens_keep_their_budgets_without_a_slowdown() {
        if std::env::var_os("POWERPLANSYNC_MOTION_SLOWDOWN").is_some() {
            return;
        }
        assert_eq!(PRESS.duration(), Duration::from_millis(140));
        assert_eq!(TOOLTIP.duration(), Duration::from_millis(150));
        assert_eq!(MENU.duration(), Duration::from_millis(200));
        assert_eq!(DIALOG.duration(), Duration::from_millis(250));
    }

    #[test]
    fn an_entrance_runs_to_rest_and_reduced_motion_skips_it() {
        let duration = Duration::from_millis(250);
        let at = |millis, reduce| share(Duration::from_millis(millis), duration, reduce);
        assert_eq!(at(0, false), Some(0.));
        assert_close(at(125, false).unwrap(), 0.5);
        assert_eq!(at(250, false), None);
        assert_eq!(at(0, true), None);
        assert_eq!(at(125, true), None);
    }

    #[test]
    fn slides_come_in_from_their_offset_and_over_content_are_opaque_early() {
        assert_eq!(
            slide_in(0., px(4.)),
            Entering {
                opacity: 0.,
                offset: px(4.),
            }
        );
        let quarter = slide_in(0.25, px(4.));
        assert_close(quarter.opacity, 0.775382);
        assert_close(f32::from(quarter.offset), 0.898473);
        assert_eq!(slide_over(OPAQUE_AT, px(6.)).opacity, 1.);
        assert_close(f32::from(slide_over(0.5, px(6.)).offset), 0.204102);
        assert_eq!(
            slide_over(1., px(6.)),
            Entering {
                opacity: 1.,
                offset: px(0.),
            }
        );
    }
}

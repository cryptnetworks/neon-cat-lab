use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

const REACH: f32 = 30.0;
pub(crate) const BURST_LIFETIME: f32 = 0.48;

pub(crate) struct Flight {
    pub(crate) position: Pos2,
    pub(crate) facing: f32,
    pub(crate) time: f32,
    pub(crate) burst: Option<Burst>,
    max_speed: f32,
    velocity: Vec2,
    caught_pointer: bool,
}

pub(crate) struct Burst {
    pub(crate) position: Pos2,
    pub(crate) age: f32,
}

impl Flight {
    pub(crate) fn new(bounds: Rect, max_speed: f32) -> Self {
        Self {
            position: pos2(bounds.left() + bounds.width() * 0.18, bounds.center().y),
            facing: 1.0,
            time: 0.0,
            burst: None,
            max_speed,
            velocity: Vec2::ZERO,
            caught_pointer: false,
        }
    }

    pub(crate) fn enter_display(&mut self, bounds: Rect, entry_point: Pos2) {
        self.position = visible_bounds(bounds).clamp(entry_point);
        self.velocity = Vec2::ZERO;
        self.caught_pointer = false;
        self.burst = None;
    }

    pub(crate) fn advance(&mut self, bounds: Rect, pointer: Option<Pos2>, dt: f32) {
        let dt = dt.min(0.05);
        self.time += dt;
        if let Some(burst) = &mut self.burst {
            burst.age += dt;
            if burst.age >= BURST_LIFETIME {
                self.burst = None;
            }
        }

        let center = bounds.center();
        let patrol = center
            + vec2(
                (self.time * 0.39).sin() * bounds.width() * 0.32,
                (self.time * 0.57).sin() * bounds.height() * 0.19,
            );
        let pointer = pointer.filter(|point| bounds.contains(*point));
        let safe_bounds = visible_bounds(bounds);
        let goal = pointer.unwrap_or(patrol);
        let offset = goal - self.position;
        let distance = offset.length();
        let speed = ((distance - if pointer.is_some() { REACH } else { 0.0 }).max(0.0) * 6.0)
            .min(self.max_speed);
        let desired_velocity = offset.normalized() * speed;
        self.velocity += (desired_velocity - self.velocity) * (1.0 - (-8.0 * dt).exp());
        self.position = safe_bounds.clamp(self.position + self.velocity * dt);
        if self.velocity.x.abs() > 22.0 {
            self.facing = self.velocity.x.signum();
        }

        if let Some(pointer) = pointer {
            let distance = self.position.distance(pointer);
            let edge_gap = safe_bounds.clamp(pointer).distance(pointer);
            let catch_radius = (edge_gap + 7.0).max(56.0);
            if distance > catch_radius + 20.0 {
                self.caught_pointer = false;
            }
            if !self.caught_pointer && distance < catch_radius {
                self.caught_pointer = true;
                self.burst = Some(Burst {
                    position: pointer,
                    age: 0.0,
                });
            }
        } else {
            self.caught_pointer = false;
        }
    }
}

fn visible_bounds(bounds: Rect) -> Rect {
    bounds.shrink2(vec2(
        60.0_f32.min(bounds.width() / 2.0),
        47.0_f32.min(bounds.height() / 2.0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1920.0, 1080.0));

    #[test]
    fn catches_the_pointer_and_bursts_once_until_it_moves_away() {
        let mut flight = Flight::new(BOUNDS, 800.0);
        let pointer = pos2(1500.0, 745.0);
        for _ in 0..240 {
            flight.advance(BOUNDS, Some(pointer), 1.0 / 120.0);
            if flight.burst.is_some() {
                break;
            }
        }
        assert_eq!(
            flight.burst.as_ref().map(|burst| burst.position),
            Some(pointer)
        );
        for _ in 0..240 {
            flight.advance(BOUNDS, Some(pointer), 1.0 / 120.0);
        }
        assert!(flight.burst.is_none());

        let next = pos2(400.0, 200.0);
        for _ in 0..240 {
            flight.advance(BOUNDS, Some(next), 1.0 / 120.0);
            if flight.burst.is_some() {
                break;
            }
        }
        assert_eq!(
            flight.burst.as_ref().map(|burst| burst.position),
            Some(next)
        );
        assert_eq!(flight.facing, -1.0);
    }

    #[test]
    fn remains_visible_when_the_pointer_leaves_the_monitor() {
        let mut flight = Flight::new(BOUNDS, 800.0);
        let other_monitor = Some(pos2(2500.0, 800.0));
        let start = flight.position;
        for _ in 0..1800 {
            flight.advance(BOUNDS, other_monitor, 1.0 / 60.0);
            assert!(BOUNDS.shrink(47.0).contains(flight.position));
        }
        assert!(flight.position.distance(start) > 100.0);
    }

    #[test]
    fn enters_a_new_display_at_the_nearest_visible_edge() {
        let mut flight = Flight::new(BOUNDS, 800.0);
        flight.enter_display(BOUNDS, pos2(-10.0, 390.0));
        assert_eq!(flight.position, pos2(60.0, 390.0));
    }
}

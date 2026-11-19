use eframe::egui::{
    Color32, Mesh, Painter, Pos2, Shape, Stroke, Vec2, epaint::CubicBezierShape, vec2,
};

use crate::flight::{BURST_LIFETIME, Burst, Flight};

const PINK: Color32 = Color32::from_rgb(255, 99, 216);
const VIOLET: Color32 = Color32::from_rgb(172, 120, 255);
const ICE: Color32 = Color32::from_rgb(117, 255, 236);
const INK: Color32 = Color32::from_rgb(22, 13, 38);
const LIGHT: Color32 = Color32::from_rgb(255, 218, 248);

pub(crate) fn draw(painter: &Painter, flight: &Flight) {
    let wave = flight.time * 4.1;
    let sprite = Sprite {
        painter,
        origin: flight.position + vec2(0.0, wave.sin() * 5.0),
        facing: flight.facing,
    };

    for star in 0..4 {
        let phase = (flight.time * 1.2 + star as f32 * 0.27).fract();
        let point = sprite.at(vec2(-48.0 - phase * 34.0, (phase * 9.0).sin() * 11.0));
        let color = PINK.gamma_multiply((1.0 - phase) * 0.8);
        painter.circle_filled(point, 0.8 + (1.0 - phase) * 1.5, color);
    }

    sprite.curve(
        [
            (-24.0, 5.0),
            (-46.0, 15.0),
            (-65.0, -3.0 + wave.cos() * 9.0),
            (-56.0, -19.0 + wave.sin() * 6.0),
        ],
        PINK,
    );

    sprite.paw(-18.0, wave + 1.0, VIOLET);
    sprite.paw(9.0, wave + 3.1, VIOLET);
    let body = sprite.at(vec2(-9.0, 4.0));
    let radii = vec2(25.0, 14.0);
    painter.add(Shape::ellipse_stroke(
        body,
        radii,
        Stroke::new(8.0_f32, PINK.gamma_multiply(0.19)),
    ));
    painter.add(Shape::ellipse_filled(body, radii, INK));
    painter.add(Shape::ellipse_stroke(
        body,
        radii,
        Stroke::new(2.5_f32, PINK),
    ));
    sprite.paw(-20.0, wave + 3.5, PINK);
    sprite.paw(10.0, wave, PINK);

    sprite.outlined_head();
    let blink = (flight.time + 1.0).rem_euclid(4.1) < 0.14;
    for x in [18.0, 33.0] {
        let eye = sprite.at(vec2(x, -10.0));
        painter.circle_filled(eye, 5.0, ICE.gamma_multiply(0.26));
        painter.add(Shape::ellipse_filled(
            eye,
            vec2(2.0, if blink { 0.8 } else { 3.4 }),
            ICE,
        ));
    }
    painter.circle_filled(sprite.at(vec2(26.0, -3.0)), 1.5, LIGHT);
    sprite.line(&[(21.0, 0.0), (26.0, 2.5), (31.0, 0.0)], 1.3, PINK);
    for whisker in [
        [(34.0, -4.0), (46.0, -8.0)],
        [(35.0, 0.0), (47.0, 0.0)],
        [(16.0, -4.0), (7.0, -7.0)],
        [(16.0, 0.0), (7.0, 1.0)],
    ] {
        sprite.line(&whisker, 1.1, ICE);
    }

    if let Some(burst) = &flight.burst {
        draw_burst(painter, burst);
    }
}

struct Sprite<'a> {
    painter: &'a Painter,
    origin: Pos2,
    facing: f32,
}

impl Sprite<'_> {
    fn at(&self, local: Vec2) -> Pos2 {
        self.origin + vec2(local.x * self.facing, local.y)
    }

    fn line(&self, local: &[(f32, f32)], width: f32, color: Color32) {
        let points = local.iter().map(|&(x, y)| self.at(vec2(x, y))).collect();
        self.painter
            .add(Shape::line(points, Stroke::new(width, color)));
    }

    fn curve(&self, local: [(f32, f32); 4], color: Color32) {
        let points = local.map(|(x, y)| self.at(vec2(x, y)));
        for (width, color) in [
            (10.0_f32, color.gamma_multiply(0.18)),
            (4.8, color),
            (1.35, LIGHT),
        ] {
            self.painter.add(CubicBezierShape::from_points_stroke(
                points,
                false,
                Color32::TRANSPARENT,
                Stroke::new(width, color),
            ));
            self.painter.circle_filled(points[3], width * 0.5, color);
        }
    }

    fn paw(&self, x: f32, phase: f32, color: Color32) {
        self.curve(
            [
                (x, 10.0),
                (x + 2.0, 16.0),
                (x + phase.sin() * 6.0, 20.0),
                (x + 5.0 + phase.sin() * 8.0, 22.0 + phase.cos() * 2.0),
            ],
            color,
        );
    }

    fn outlined_head(&self) {
        let points = [
            (2.0, -14.0),
            (4.0, -34.0),
            (7.0, -36.0),
            (17.0, -24.0),
            (29.0, -24.0),
            (39.0, -35.0),
            (42.0, -33.0),
            (43.0, -14.0),
            (46.0, -7.0),
            (44.0, 2.0),
            (38.0, 8.0),
            (25.0, 11.0),
            (12.0, 8.0),
            (4.0, 3.0),
            (0.0, -5.0),
        ]
        .map(|(x, y)| self.at(vec2(x, y)));
        let mut fill = Mesh::default();
        fill.colored_vertex(self.at(vec2(24.0, -10.0)), INK);
        for (index, &point) in points.iter().enumerate() {
            fill.colored_vertex(point, INK);
            fill.add_triangle(0, index as u32 + 1, ((index + 1) % points.len()) as u32 + 1);
        }
        self.painter.add(fill);
        self.painter.add(Shape::closed_line(
            points.to_vec(),
            Stroke::new(12.0_f32, PINK.gamma_multiply(0.17)),
        ));
        self.painter.add(Shape::closed_line(
            points.to_vec(),
            Stroke::new(4.4_f32, PINK),
        ));
        self.painter.add(Shape::closed_line(
            points.to_vec(),
            Stroke::new(1.5_f32, LIGHT),
        ));
        self.line(&[(7.0, -29.0), (8.0, -20.0), (13.0, -23.0)], 1.4, VIOLET);
        self.line(&[(38.0, -28.0), (34.0, -23.0), (39.0, -21.0)], 1.4, VIOLET);
    }
}

fn draw_burst(painter: &Painter, burst: &Burst) {
    let progress = (burst.age / BURST_LIFETIME).clamp(0.0, 1.0);
    let fade = (1.0 - progress).powi(2);
    let radius = 8.0 + progress * 39.0;
    painter.circle_stroke(
        burst.position,
        radius,
        Stroke::new(5.0_f32, ICE.gamma_multiply(fade * 0.23)),
    );
    painter.circle_stroke(
        burst.position,
        radius,
        Stroke::new(1.7_f32, ICE.gamma_multiply(fade)),
    );
    for ray in 0..7 {
        let angle = (ray as f32 + 0.3) * std::f32::consts::TAU / 7.0;
        let direction = vec2(angle.cos(), angle.sin());
        let start = burst.position + direction * (radius + 3.0);
        let end = start + direction * (4.0 + fade * 8.0);
        painter.line_segment(
            [start, end],
            Stroke::new(2.0_f32, PINK.gamma_multiply(fade)),
        );
    }
}

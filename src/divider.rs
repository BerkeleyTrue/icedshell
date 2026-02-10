use std::f32::consts::{FRAC_PI_2, PI};

use iced::{
    Color, Element, Length, Point, Radians, Size, Theme,
    mouse::Cursor,
    widget::{
        Canvas,
        canvas::{Cache, Geometry, Path, Program, path::Arc},
    },
};

// The state for your triangle
pub struct Angled {
    color: Color,
    cache: Cache,
    direction: Direction,
    heading: Heading,
    height: f32,
}

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy)]
pub enum Heading {
    North,
    South,
}

impl Angled {
    pub fn new(color: Color, direction: Direction, heading: Heading, height: f32) -> Self {
        Self {
            color,
            cache: Cache::new(),
            direction,
            heading,
            height,
        }
    }
}

impl<Message> Program<Message> for Angled {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geo = self.cache.draw(renderer, bounds.size(), move |frame| {
            let Size { width, height } = bounds.size();

            let path = Path::new(|builder| {
                match (self.heading, self.direction) {
                    (Heading::North, Direction::Left) => {
                        builder.move_to(Point::new(width, 0.0));
                        builder.line_to(Point::new(width, height));
                        builder.line_to(Point::new(0.0, height));
                    }
                    (Heading::North, Direction::Right) => {
                        builder.move_to(Point::new(0.0, 0.0));
                        builder.line_to(Point::new(width, height));
                        builder.line_to(Point::new(0.0, height));
                    }
                    (Heading::South, Direction::Left) => {
                        builder.move_to(Point::new(width, height));
                        builder.line_to(Point::new(0.0, 0.0));
                        builder.line_to(Point::new(width, 0.0));
                    }
                    (Heading::South, Direction::Right) => {
                        builder.move_to(Point::new(0.0, height));
                        builder.line_to(Point::new(width, 0.0));
                        builder.line_to(Point::new(0.0, 0.0));
                    }
                }
                builder.close();
            });

            frame.fill(&path, self.color);
        });

        vec![geo]
    }
}

impl<'a, Message: 'a> From<Angled> for Element<'a, Message> {
    fn from(divider: Angled) -> Self {
        let height = divider.height;
        let width = (height / 1.75).round_ties_even();
        Canvas::new(divider)
            .height(Length::Fixed(height))
            .width(Length::Fixed(width))
            .into()
    }
}

pub struct Semi {
    color: Color,
    cache: Cache,
    direction: Direction,
    height: f32,
}

impl Semi {
    pub fn new(color: Color, direction: Direction, height: f32) -> Self {
        Self {
            color,
            cache: Cache::new(),
            direction,
            height,
        }
    }
}

impl<Message> Program<Message> for Semi {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geo = self.cache.draw(renderer, bounds.size(), move |frame| {
            let radius = frame.height() / 2.0;

            let (center, start, end) = match self.direction {
                Direction::Right => (
                    Point::new(0.0, radius),
                    Radians(-FRAC_PI_2),
                    Radians(FRAC_PI_2),
                ),
                Direction::Left => (
                    Point::new(frame.width(), radius),
                    Radians(FRAC_PI_2),
                    Radians(FRAC_PI_2 + PI),
                ),
            };

            let semi = Path::new(|b| {
                b.arc(Arc {
                    center,
                    radius,
                    start_angle: start,
                    end_angle: end,
                });
                b.close();
            });

            frame.fill(&semi, self.color);
        });

        vec![geo]
    }
}

impl<'a, Message: 'a> From<Semi> for Element<'a, Message> {
    fn from(semi: Semi) -> Self {
        let height = semi.height;
        let width = height / 2.0;
        Canvas::new(semi).height(height).width(width).into()
    }
}

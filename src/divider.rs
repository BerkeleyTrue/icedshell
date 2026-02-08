use iced::{
    Color, Element, Length, Point, Size, Theme,
    mouse::Cursor,
    widget::{
        Canvas,
        canvas::{Frame, Geometry, Path, Program},
    },
};

// The state for your triangle
pub struct Divider {
    color: Color,
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

impl Divider {
    pub fn new(color: Color, direction: Direction, heading: Heading, height: f32) -> Self {
        Self {
            color,
            direction,
            heading,
            height,
        }
    }
}

impl<Message> Program<Message> for Divider {
    type State = ();

    // TODO: add caching
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

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

        vec![frame.into_geometry()]
    }
}

impl<'a, Message: 'a> From<Divider> for Element<'a, Message> {
    fn from(divider: Divider) -> Self {
        let height = divider.height;
        let width = (height / 1.75).round_ties_even();
        Canvas::new(divider)
            .height(Length::Fixed(height))
            .width(Length::Fixed(width))
            .into()
    }
}

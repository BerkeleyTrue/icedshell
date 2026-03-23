use iced::{
    Element, Length, Rectangle, Size,
    advanced::{Layout, layout, renderer, widget::Tree, widget::Widget},
    mouse,
};

pub struct AtlasFrame<'a> {
    handle: &'a iced::advanced::image::Handle,
    cols: u32,
    tile_w: f32,
    tile_h: f32,
    index: u32,
}

impl<'a> AtlasFrame<'a> {
    pub fn new(
        handle: &'a iced::advanced::image::Handle,
        cols: u32,
        tile_w: f32,
        tile_h: f32,
        index: u32,
    ) -> Self {
        Self {
            handle,
            cols,
            tile_w,
            tile_h,
            index,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for AtlasFrame<'a>
where
    Renderer: iced::advanced::Renderer
        + iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.tile_w), Length::Fixed(self.tile_h))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        // Tile selection
        let col = self.index % self.cols;
        let row = self.index / self.cols;

        let crop = iced::Rectangle::<u32> {
            x: col * self.tile_w as u32, // if tile_w is f32, store u32 instead
            y: row * self.tile_h as u32,
            width: self.tile_w as u32,
            height: self.tile_h as u32,
        };

        iced::widget::image::draw(
            renderer,
            layout,
            self.handle,
            Some(crop),
            iced::border::Radius::from(0.0),
            iced::ContentFit::Contain,
            iced::widget::image::FilterMethod::Nearest,
            iced::Rotation::default(),
            1.0, // opacity
            1.0, // scale
        );
    }
}

impl<'a, Message, Theme, Renderer> From<AtlasFrame<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer
        + iced::advanced::image::Renderer<Handle = iced::advanced::image::Handle>,
{
    fn from(w: AtlasFrame<'a>) -> Self {
        Element::new(w)
    }
}

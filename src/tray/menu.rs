#[derive(Debug, Clone)]
pub enum MenuMessage {
}
pub struct Menu {}

impl Comp for Menu {
    type Message = MenuMessage;
    type Init = ();

    fn new(_input: Self::Init) -> Self {
        Self {}
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        text!("Menu").into()
    }
}

impl Feature for Menu {
    fn layer(&self) -> iced_layershell::reexport::NewLayerShellSettings {
        NewLayerShellSettings {
            layer: Layer::Top,
            size: Some((0, 20 as u32)),
            anchor: Anchor::empty(),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            exclusive_zone: Some(32 as i32),
            output_option: OutputOption::LastOutput,
            events_transparent: false,
            namespace: Some("DeloraMainBar".into()),
            margin: None,
        }
    }
}

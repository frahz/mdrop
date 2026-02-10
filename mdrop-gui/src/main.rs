use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream};
use iced::widget::{column, container, pick_list, slider, svg, text};
use iced::{Center, Element, Fill, Size, Subscription, Theme, stream};
use mdrop::filter::Filter;
use mdrop::gain::Gain;
use mdrop::indicator_state::IndicatorState;
use mdrop::volume::Volume;
use mdrop::{Moondrop, MoondropInfo};

const WIDTH: u32 = 300;

pub fn main() -> iced::Result {
    env_logger::init();

    iced::application(MdropGui::default, MdropGui::update, MdropGui::view)
        .title("mdrop")
        .window(iced::window::Settings {
            size: Size::new(300.0, 300.0),
            min_size: Some(Size::new(300.0, 300.0)),
            ..Default::default()
        })
        .subscription(MdropGui::subscription)
        .theme(MdropGui::theme)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    SetVolume,
    VolumeChanged(u32),
    SelectFilter(Filter),
    SelectIndicator(IndicatorState),
    SelectGain(Gain),
    UpdateDevice(Option<MoondropInfo>),
}

pub struct MdropGui {
    moondrop: Moondrop,
    info: Option<MoondropInfo>,
}

impl MdropGui {
    fn update(&mut self, message: Message) {
        match message {
            Message::SetVolume => {
                if let Some(info) = self.info.as_ref() {
                    self.moondrop.set_volume(info.volume);
                }
            }
            Message::VolumeChanged(value) => {
                if let Some(info) = self.info.as_mut() {
                    info.volume = Volume::new(value);
                }
            }
            Message::SelectFilter(filter) => {
                if let Some(info) = self.info.as_mut() {
                    info.filter = filter;
                    self.moondrop.set_filter(filter);
                }
            }
            Message::SelectIndicator(indicator_state) => {
                if let Some(info) = self.info.as_mut() {
                    info.indicator_state = indicator_state;
                    self.moondrop.set_indicator_state(indicator_state);
                }
            }
            Message::SelectGain(gain) => {
                if let Some(info) = self.info.as_mut() {
                    info.gain = gain;
                    self.moondrop.set_gain(gain);
                }
            }
            Message::UpdateDevice(moondrop_info) => {
                log::debug!("app update: {:?}", moondrop_info);
                self.info = moondrop_info;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.info {
            Some(info) => {
                let name = text(&info.name);

                let filter_list =
                    pick_list(&Filter::ALL[..], Some(info.filter), Message::SelectFilter)
                        .width(WIDTH);
                let gain_list =
                    pick_list(&Gain::ALL[..], Some(info.gain), Message::SelectGain).width(WIDTH);
                let indicator_list = pick_list(
                    &IndicatorState::ALL[..],
                    Some(info.indicator_state),
                    Message::SelectIndicator,
                )
                .width(WIDTH);
                let h_slider = container(
                    slider(1..=100, info.volume.inner(), Message::VolumeChanged)
                        .on_release(Message::SetVolume)
                        .shift_step(5u32),
                )
                .width(WIDTH);

                let text = text(info.volume.inner());

                column![name, gain_list, indicator_list, filter_list, h_slider, text,]
                    .width(Fill)
                    .align_x(Center)
                    .spacing(20)
                    .padding(20)
                    .into()
            }
            None => {
                let handle = svg::Handle::from_memory(include_bytes!("../res/dongle.svg"));
                let svg = svg(handle)
                    .width(Fill)
                    .style(|theme: &Theme, _| svg::Style {
                        color: Some(theme.palette().text),
                    });
                let text = text("No Moondrop dongle detected.\nPlease attach dongle.").center();
                column![svg, text,]
                    .width(Fill)
                    .align_x(Center)
                    .spacing(20)
                    .padding(20)
                    .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(worker).map(Message::UpdateDevice)
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinMocha
    }
}

fn worker() -> impl Stream<Item = Option<MoondropInfo>> {
    stream::channel(
        1,
        async move |mut output: mpsc::Sender<Option<MoondropInfo>>| {
            let mut moondrop = Moondrop::new();
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                moondrop.watch(tx);
            });

            loop {
                let data = rx.recv().unwrap();
                output.send(None).await.expect("dummy send");
                output.send(data).await.expect("failed to send data");
            }
        },
    )
}

impl Default for MdropGui {
    fn default() -> Self {
        let moondrop = Moondrop::new();
        let info = moondrop.get_all();
        Self { moondrop, info }
    }
}

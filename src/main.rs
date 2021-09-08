use iced::{
    button, executor, Align, Application, Button, Column, Command, Element, HorizontalAlignment,
    Length, Row, Settings, Subscription, Text,
};
use iced_futures::{self, futures};
use std::time::{Duration, Instant};

const FPS: u64 = 30;
const MILLISEC: u64 = 1000;
const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Stop,
    Reset,
    Update,
}

pub enum TickState {
    Stopped,
    Ticking,
}

struct RusTimer {
    last_update: Instant,
    total_duration: Duration,
    tick_state: TickState,
    start_stop_button_state: button::State,
    reset_button_state: button::State,
}

impl Application for RusTimer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            RusTimer {
                last_update: Instant::now(),
                total_duration: Duration::default(),
                tick_state: TickState::Stopped,
                start_stop_button_state: button::State::new(),
                reset_button_state: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("RusTimer")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Start => {
                self.tick_state = TickState::Ticking;
                self.last_update = Instant::now();
            }
            Message::Stop => {
                self.tick_state = TickState::Stopped;
                self.last_update += Instant::now() - self.last_update;
            }
            Message::Reset => {
                self.last_update = Instant::now();
                self.total_duration = Duration::default();
            }
            Message::Update => {
                if let TickState::Ticking = self.tick_state {
                    let now_update = Instant::now();
                    self.total_duration += now_update - self.last_update;
                    self.last_update = now_update;
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let seconds = self.total_duration.as_secs();
        let duration_text = format!(
            "{:0>2}:{:0>2}:{:0>2}.{:0>2}",
            seconds / HOUR,
            (seconds % HOUR) / MINUTE,
            seconds % MINUTE,
            self.total_duration.subsec_millis() / 10,
        );

        let start_stop_text = match self.tick_state {
            TickState::Stopped => {
                Text::new("Start").horizontal_alignment(HorizontalAlignment::Center)
            }
            TickState::Ticking => {
                Text::new("Stop").horizontal_alignment(HorizontalAlignment::Center)
            }
        };

        let start_stop_message = match self.tick_state {
            TickState::Stopped => Message::Start,
            TickState::Ticking => Message::Stop,
        };

        let tick_text = Text::new(duration_text).size(60);
        let start_stop_button = Button::new(&mut self.start_stop_button_state, start_stop_text)
            .min_width(80)
            .on_press(start_stop_message);
        let reset_button = Button::new(
            &mut self.reset_button_state,
            Text::new("Reset").horizontal_alignment(HorizontalAlignment::Center),
        )
        .min_width(80)
        .on_press(Message::Reset);

        Column::new()
            .push(tick_text)
            .push(
                Row::new()
                    .push(start_stop_button)
                    .push(reset_button)
                    .spacing(10),
            )
            .spacing(10)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let timer = Timer::new(Duration::from_millis(MILLISEC / FPS));
        iced::Subscription::from_recipe(timer).map(|_| Message::Update)
    }
}

pub struct Timer {
    duration: Duration,
}

impl Timer {
    fn new(duration: Duration) -> Timer {
        Timer { duration }
    }
}

impl<H, E> iced_native::subscription::Recipe<H, E> for Timer
where
    H: std::hash::Hasher,
{
    type Output = Instant;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
        self.duration.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::BoxStream<E>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use futures::stream::StreamExt;
        async_std::stream::interval(self.duration)
            .map(|_| Instant::now())
            .boxed()
    }
}

fn main() {
    let mut settings = Settings::default();
    settings.window.size = (400u32, 120u32);
    RusTimer::run(settings);
}

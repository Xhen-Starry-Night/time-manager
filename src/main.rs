fn theme_fn(_: &time_manager::app::App) -> iced::Theme {
    iced::Theme::TokyoNight
}

fn main() -> iced::Result {
    iced::application(time_manager::app::App::new, time_manager::app::App::update, time_manager::app::App::view)
        .subscription(time_manager::app::App::subscription)
        .title("Time Manager")
        .theme(theme_fn)
        .run()
}
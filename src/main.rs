use tracing_subscriber::{fmt, EnvFilter};

fn theme_fn(_: &time_manager::app::App) -> iced::Theme {
    iced::Theme::TokyoNight
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // 默认配置：项目自身 debug，fsrs 库 debug，其他依赖 info
            EnvFilter::builder()
                .parse_lossy("time_manager=debug,tmd_cli=debug,fsrs=debug,info")
        });

    fmt()
        .with_env_filter(filter)
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .compact()
        .init();

    tracing::info!("Time Manager GUI starting");
}

fn main() -> iced::Result {
    init_tracing();
    
    let result = iced::application(time_manager::app::App::new, time_manager::app::App::update, time_manager::app::App::view)
        .subscription(time_manager::app::App::subscription)
        .title("Time Manager")
        .theme(theme_fn)
        .run();

    match &result {
        Ok(_) => tracing::info!("Time Manager GUI exited normally"),
        Err(e) => tracing::error!("Time Manager GUI error: {:?}", e),
    }

    result
}

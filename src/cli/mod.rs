use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tmd")]
#[command(about = "Time Manager CLI - File-based learning management")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true)]
    pub data_dir: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "init")]
    Init,

    #[command(name = "tree-create")]
    TreeCreate {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short, long)]
        import: Option<String>,
    },

    #[command(name = "tree-list")]
    TreeList {
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(name = "card-create")]
    CardCreate {
        path: String,
        #[arg(short, long)]
        preset: Option<String>,
    },

    #[command(name = "card-list")]
    CardList { path: String },

    #[command(name = "card-predict")]
    CardPredict { path: String },

    #[command(name = "card-link")]
    CardLink {
        timer: String,
        path: String,
        #[arg(short, long)]
        quality: String,
    },

    #[command(name = "preset-create")]
    PresetCreate {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
    },

    #[command(name = "preset-list")]
    PresetList {
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(name = "preset-train")]
    PresetTrain { name: String },

    #[command(name = "timer-start")]
    TimerStart {
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(name = "timer-pause")]
    TimerPause,

    #[command(name = "timer-stop")]
    TimerStop,

    #[command(name = "timer-get")]
    TimerGet { name: String },

    #[command(name = "todo-create")]
    TodoCreate { content: String },

    #[command(name = "todo-list")]
    TodoList,

    #[command(name = "todo-to-schedule")]
    TodoToSchedule {
        id: String,
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        end: String,
    },

    #[command(name = "schedule-create")]
    ScheduleCreate {
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        end: String,
        #[arg(short, long)]
        summary: String,
    },

    #[command(name = "schedule-list")]
    ScheduleList,

    #[command(name = "review-list")]
    ReviewList {
        #[arg(short, long)]
        urgency: Option<String>,
    },
}

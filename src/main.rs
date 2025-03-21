use attendance::manager::AttendanceManager;
use clap::{Args, Parser, Subcommand, ValueEnum};
use diesel::QueryResult;
use std::io::{self, BufRead};

/// A parser for the command line interface for this attendance application.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The different kinds of commands that can be run for this application.
    #[command(subcommand)]
    command: Command,
}

/// The different subcommands that can be run for this attendance application.
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Runs setup for a semester's attendance. ONLY RUN ONCE!
    Setup,
    /// Updates the roster of students via the [`ROSTER_PATH`].
    UpdateRoster,
    /// Show the roster of students.
    ShowRoster {
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show every student who has been absent at least once, after a certain week.
    Absences {
        #[arg(short, long)]
        after_week: i32,
    },
    /// Show the info and attendance of a specific student,
    StudentInfo { id: String },
    /// Actions to perform specific to a given week.
    Week(WeekArgs),
}

/// The command-line arguments for doing actions given a specific week.
#[derive(Args, Debug, Clone)]
struct WeekArgs {
    /// The current week.
    week: i32,
    /// The action to perform for the given week.
    #[arg(value_enum)]
    command: WeekCommand,
}

/// The different kinds of actions that can be done for a specific week.
#[derive(ValueEnum, Debug, Clone)]
enum WeekCommand {
    /// Reads student emails from stdin and marks those students as present for the given week.
    MarkPresent,
    /// Reads student emails from stdin and marks those students as excused for the given week.
    MarkExcused,
    /// Marks any remaining students as absent.
    MarkAbsent,
    /// Displays the attendance for the given week.
    ShowWeek,
    /// Resets / deletes all attendance records for the given week.
    Reset,
}

fn main() -> QueryResult<()> {
    let args = Cli::parse();

    match args.command {
        Command::Setup => attendance::setup(),
        Command::UpdateRoster => attendance::update_roster(),
        Command::ShowRoster { verbose } => attendance::display::show_roster(verbose),
        Command::Absences { after_week } => attendance::display::show_absences(after_week),
        Command::StudentInfo { id } => attendance::display::show_student_info(&id),
        Command::Week(week_args) => run_week_command(week_args),
    }
}

/// A helper function for running the week-specific subcommands.
fn run_week_command(week_args: WeekArgs) -> QueryResult<()> {
    let curr_week = week_args.week;

    match week_args.command {
        WeekCommand::ShowWeek => {
            attendance::display::show_week_attendance(curr_week)?;
            return Ok(());
        }
        WeekCommand::MarkAbsent => {
            AttendanceManager::connect().mark_remaining_absent(curr_week)?;
            return Ok(());
        }
        WeekCommand::Reset => {
            AttendanceManager::connect().delete_week_attendance(curr_week)?;
            return Ok(());
        }
        WeekCommand::MarkPresent | WeekCommand::MarkExcused => (),
    };

    let mut emails = vec![];

    // Read in data from stdin.
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("Unable to read from stdin");
        emails.push(line);
    }

    // Convert all email addresses to IDs (verifying them along the way).
    let ids: Vec<&str> = emails
        .iter()
        .map(|email| {
            assert!(
                email.trim().ends_with("@andrew.cmu.edu"),
                "email entry is invalid: '{email}'"
            );

            let i = email
                .find("@andrew.cmu.edu")
                .expect("We just checked that this is here");

            &email[0..i]
        })
        .collect();

    let mut manager = AttendanceManager::connect();

    match week_args.command {
        WeekCommand::MarkPresent => manager.mark_present(curr_week, &ids),
        WeekCommand::MarkExcused => manager.mark_excused(curr_week, &ids),
        _ => unreachable!("we checked for the other variants above"),
    }
    .map(|_| ())
}

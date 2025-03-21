use chrono::NaiveDate;
use csv::Reader;
use diesel::QueryResult;
use std::path::Path;

pub mod display;
pub mod manager;
pub mod models;
pub mod schema;

use manager::AttendanceManager;
use models::Student;

/// The path to the roster of students.
///
/// We hardcode this since this should only change once per semester.
const ROSTER_PATH: &str = "../roster-s25.csv";

/// The date of the first day of attendance.
///
/// We hardcode this since this should only change once per semester.
const START_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 1, 15).expect("date is not real");

/// The weeks that are valid.
///
/// We hardcode this since this should only change once per semester.
const VALID_WEEKS: [bool; 15] = [
    true, true, true, true, true, true, true, false, true, true, true, true, true, true, true,
];

/// A helper struct to carry information about a student's attendance.
#[derive(Debug, Clone)]
pub struct StudentAttendance {
    /// The dates where a student is present.
    pub present: Vec<(i32, NaiveDate)>,
    /// The dates where a student is excused.
    pub excused: Vec<(i32, NaiveDate)>,
    /// The dates where a student is absent.
    pub absent: Vec<(i32, NaiveDate)>,
}

/// Downloads a student roster given a path to a CSV file.
///
/// # Panics
///
/// This function will panic if it is unable to find or read the file specified by the path, as well
/// as if it is unable to deserialize any of the records in the file.
fn download_roster<P: AsRef<Path>>(path: P) -> Vec<Student> {
    let mut csv = Reader::from_path(path).expect("unable to read from csv");

    csv.deserialize()
        .collect::<Result<Vec<Student>, _>>()
        .expect("unable to deserialize record")
}

/// Runs setup for a semester's attendance.
///
/// This binary should ONLY be run once, at the beginning of the semester.
///
/// This binary will download the roster from the provided [`ROSTER_PATH`], and it will also set up
/// the `weeks` table with the correct starting date and weeks.
///
/// IMPORTANT: Depending on the semester, [`VALID_WEEKS`] might have to change.
pub fn setup() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Delete the entire roster before importing a new one.
    let _ = manager.delete_roster();

    // Insert the students from the given roster.
    let new_roster = download_roster(ROSTER_PATH);
    manager.insert_students(&new_roster)?;

    let roster = manager.get_roster()?;
    println!("{:#?}", roster);
    println!("{} students total", roster.len());

    // Initialize the weeks for this semester.
    manager.initialize_weeks(START_DATE, &VALID_WEEKS)?;

    Ok(())
}

/// Updates the roster of students.
///
/// This binary will look at the roster provided in [`ROSTER_PATH`] and look at the diff between the
/// current roster stored in the database. It will then add / delete students according to the CSV
/// roster from [`ROSTER_PATH`].
pub fn update_roster() -> QueryResult<()> {
    let mut manager = AttendanceManager::connect();

    // Insert the students from the given roster.
    let new_roster = download_roster(ROSTER_PATH);

    let curr_roster = manager.get_roster()?;

    let dropped: Vec<&Student> = curr_roster
        .iter()
        .filter(|student| !new_roster.contains(student))
        .collect();
    println!("Students dropped: {:#?}", dropped);

    for student in dropped {
        manager.delete_student(&student.id)?;
    }

    let added: Vec<Student> = new_roster
        .iter()
        .filter(|student| !curr_roster.contains(student))
        .cloned()
        .collect();
    println!("Students added: {:#?}", added);

    manager.insert_students(&added)?;

    Ok(())
}

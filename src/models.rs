use crate::schema::{attendance, students, weeks};
use chrono::NaiveDate;
use diesel::deserialize::FromSql;
use diesel::prelude::*;
use diesel::serialize::{Output, ToSql};
use diesel::sqlite::{Sqlite, SqliteValue};
use diesel::{AsExpression, FromSqlRow, sql_types::Text};
use serde::Deserialize;
use std::fmt::Display;
use tabled::Tabled;

/// The attendance record for a student for a specific week.
#[derive(Queryable, Selectable, Insertable, Tabled, Debug, Clone)]
#[diesel(table_name = attendance)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Attendance {
    /// A foreign-key reference to a student's ID in the students table.
    pub student: String,
    /// A foreign-key reference to a week's ID in the weeks table.
    pub week: i32,
    /// The status of a student for a given week.
    pub status: Status,
}

/// An entry in the roster of students, representing a student in the class.
///
/// This type is intended to be deserialized from the course CSV roster. You can get this roster by
/// downloading off of the S3 admin page.
///
/// Note that there are a lot more columns that the ones listed here, but the remaining columns
/// aren't super interesting and are usually the same among every student.
#[derive(
    Deserialize, Queryable, Selectable, Insertable, Tabled, Debug, Clone, PartialEq, Eq, Hash,
)]
#[diesel(table_name = students)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Student {
    #[serde(rename(deserialize = "Andrew ID"))]
    pub id: String,

    #[serde(rename(deserialize = "Email"))]
    pub email: String,

    #[serde(rename(deserialize = "Preferred/First Name"))]
    pub first_name: String,

    #[serde(rename(deserialize = "MI"))]
    pub middle_initial: String,

    #[serde(rename(deserialize = "Last Name"))]
    pub last_name: String,

    #[serde(rename(deserialize = "College"))]
    pub college: String,

    #[serde(rename(deserialize = "Department"))]
    pub department: String,

    #[serde(rename(deserialize = "Major"))]
    pub major: String,

    #[serde(rename(deserialize = "Class"))]
    pub class: i32,

    #[serde(rename(deserialize = "Graduation Semester"))]
    pub graduation_semester: String,
}

/// The actual date of a given week during the semester.
#[derive(
    Queryable, Selectable, Insertable, Debug, Tabled, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
#[diesel(table_name = weeks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Week {
    pub id: i32,
    pub date: NaiveDate,
}

#[derive(FromSqlRow, AsExpression, Debug, Clone, Copy)]
#[diesel(sql_type = Text)]
pub enum Status {
    Present,
    Excused,
    Absent,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Present => write!(f, "Present"),
            Status::Excused => write!(f, "Excused"),
            Status::Absent => write!(f, "Absent"),
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "Present" => Ok(Status::Present),
            "Excused" => Ok(Status::Excused),
            "Absent" => Ok(Status::Absent),
            _ => Err(format!("Unknown status: {}", s)),
        }
    }
}

impl FromSql<Text, Sqlite> for Status {
    fn from_sql(bytes: SqliteValue) -> diesel::deserialize::Result<Self> {
        let t = <String as FromSql<Text, Sqlite>>::from_sql(bytes)?;
        Ok(t.as_str().try_into()?)
    }
}

impl ToSql<Text, Sqlite> for Status {
    fn to_sql<'a>(&'a self, out: &mut Output<'a, '_, Sqlite>) -> diesel::serialize::Result {
        out.set_value(self.to_string());
        Ok(diesel::serialize::IsNull::No)
    }
}

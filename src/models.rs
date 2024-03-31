use sqlx::{mysql::MySqlPoolOptions, prelude::FromRow, MySqlPool};

#[derive(Debug, FromRow)]
pub struct Project {
    pub ProjectID: i32,
    pub Title: String,
    pub Description: String,
}

#[derive(Debug, FromRow)]
pub struct Member {
    pub MemberID: i32,
    pub firstName: String,
    pub lastName: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, FromRow)]
pub struct Sprint {
    pub SprintID: i32,
    pub Title: String,
    pub startDate: chrono::NaiveDate,
    pub endDate: chrono::NaiveDate,
}

#[derive(Debug, FromRow)]
pub struct Task {
    pub TaskID: i32,
    pub Title: String,
    pub Status: String,
    pub Description: String,
    pub commitedHours: i32,
    pub estimatedHours: i32,
}

#[derive(Debug, FromRow)]
pub struct PartOf {
    pub TaskID: i32,
    pub SprintID: i32,
}

#[derive(Debug, FromRow)]
pub struct AssignedTo {
    pub MemberID: i32,
    pub TaskID: i32,
}

#[derive(Debug, FromRow)]
pub struct ContributesTo {
    pub MemberID: i32,
    pub ProjectID: i32,
    pub role: String,
}
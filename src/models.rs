use chrono::NaiveDate;
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct Project {
    #[sqlx(rename = "ProjectID")]
    pub proj_id: i32,
    #[sqlx(rename = "Title")]
    pub title: String,
    #[sqlx(rename = "Description")]
    pub desc: String,
    pub sprints: Vec<Sprint>,
    pub members: Vec<Member>,
}

#[derive(Debug, FromRow, Clone)]
pub struct Member {
    #[sqlx(rename = "MemberID")]
    pub member_id: i32,
    #[sqlx(rename = "firstName")]
    pub first_name: String,
    #[sqlx(rename = "lastName")]
    pub last_name: String,
    #[sqlx(rename = "email")]
    pub email: String,
    #[sqlx(rename = "phone")]
    pub phone: String,
}

#[derive(Debug, Clone)]
pub struct Sprint {
    pub sprint_id: i32,
    pub title: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub tasks: Vec<Task>,
}

#[derive(Debug, FromRow)]
pub struct RawSprint {
    #[sqlx(rename = "SprintID")]
    pub sprint_id: i32,
    #[sqlx(rename = "Title")]
    pub title: String,
    #[sqlx(rename = "startDate")]
    pub start_date: NaiveDate,
    #[sqlx(rename = "endDate")]
    pub end_date: NaiveDate,
}

#[derive(Debug, FromRow, Clone)]
pub struct Task {
    #[sqlx(rename = "TaskID")]
    pub task_id: i32,
    #[sqlx(rename = "Title")]
    pub title: String,
    #[sqlx(rename = "Status")]
    pub status: String,
    #[sqlx(rename = "Description")]
    pub description: String,
    #[sqlx(rename = "commitedHours")]
    pub commited_hours: i32,
    #[sqlx(rename = "estimatedHours")]
    pub estimated_hours: i32,
}

#[derive(Debug, FromRow)]
pub struct RawProject {
    #[sqlx(rename = "ProjectID")]
    pub project_id: i32,
    #[sqlx(rename = "Title")]
    pub title: String,
    #[sqlx(rename = "Description")]
    pub description: String,
}

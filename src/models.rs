use chrono::NaiveDate;
use sqlx::{FromRow, MySql, Pool};

#[derive(Debug, FromRow, Clone)]
pub struct Project {
    #[sqlx(rename = "ProjectID")]
    pub ProjectID: i32,
    #[sqlx(rename = "Title")]
    pub Title: String,
    #[sqlx(rename = "Description")]
    pub Description: String,
    #[sqlx(rename = "Sprints")]
    pub Sprints: Vec<Sprint>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Sprint {
    #[sqlx(rename = "SprintID")]
    pub SprintID: i32,
    #[sqlx(rename = "Title")]
    pub Title: String,
    #[sqlx(rename = "startDate")]
    pub startDate: NaiveDate,
    #[sqlx(rename = "endDate")]
    pub endDate: NaiveDate,
    #[sqlx(rename = "Tasks")]
    pub Tasks: Vec<Task>,
}

#[derive(Debug, FromRow)]
pub struct RawSprint {
    #[sqlx(rename = "SprintID")]
    pub SprintID: i32,
    #[sqlx(rename = "Title")]
    pub Title: String,
    #[sqlx(rename = "startDate")]
    pub startDate: NaiveDate,
    #[sqlx(rename = "endDate")]
    pub endDate: NaiveDate,
}

#[derive(Debug, FromRow, Clone)]
pub struct Task {
    #[sqlx(rename = "TaskID")]
    pub TaskID: i32,
    #[sqlx(rename = "Title")]
    pub Title: String,
    #[sqlx(rename = "Status")]
    pub Status: String,
    #[sqlx(rename = "Description")]
    pub Description: String,
    #[sqlx(rename = "commitedHours")]
    pub commitedHours: i32,
    #[sqlx(rename = "estimatedHours")]
    pub estimatedHours: i32,
}

// Temporary struct to fetch project data
#[derive(Debug, FromRow)]
struct RawProject {
    #[sqlx(rename = "ProjectID")]
    ProjectID: i32,
    #[sqlx(rename = "Title")]
    Title: String,
    #[sqlx(rename = "Description")]
    Description: String,
}

pub async fn delete_project_by_id(pool: &Pool<MySql>, project_id: i32) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    sqlx::query("DELETE FROM contributesto WHERE ProjectID = ?")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    // Then, delete related entries from ProjectSprint.
    sqlx::query("DELETE FROM ProjectSprint WHERE ProjectID = ?")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    // Next, delete the sprints that are associated with the project.
    sqlx::query("DELETE FROM Sprint WHERE SprintID IN (SELECT SprintID FROM ProjectSprint WHERE ProjectID = ?)")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    // Finally, delete the project itself.
    sqlx::query("DELETE FROM Project WHERE ProjectID = ?")
        .bind(project_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}

pub async fn fetch_projects(pool: &Pool<MySql>) -> Result<Vec<Project>, sqlx::Error> {
    let raw_projects = sqlx::query_as::<_, RawProject>("SELECT * FROM Project")
        .fetch_all(pool)
        .await?;

    let mut projects = Vec::new();

    for raw_project in raw_projects {
        let raw_sprints = sqlx::query_as::<_, RawSprint>(
            "SELECT Sprint.* FROM Sprint
             INNER JOIN ProjectSprint ON Sprint.SprintID = ProjectSprint.SprintID
             WHERE ProjectSprint.ProjectID = ?",
        )
        .bind(raw_project.ProjectID)
        .fetch_all(pool)
        .await?;

        let mut sprints = Vec::new();

        for raw_sprint in raw_sprints {
            let tasks = sqlx::query_as::<_, Task>(
                "SELECT Task.* FROM Task
                 INNER JOIN PartOf ON Task.TaskID = PartOf.TaskID
                 WHERE PartOf.SprintID = ?",
            )
            .bind(raw_sprint.SprintID)
            .fetch_all(pool)
            .await?;

            sprints.push(Sprint {
                SprintID: raw_sprint.SprintID,
                Title: raw_sprint.Title,
                startDate: raw_sprint.startDate,
                endDate: raw_sprint.endDate,
                Tasks: tasks,
            });
        }

        projects.push(Project {
            ProjectID: raw_project.ProjectID,
            Title: raw_project.Title,
            Description: raw_project.Description,
            Sprints: sprints,
        });
    }

    Ok(projects)
}

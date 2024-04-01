use crate::models::*;
use sqlx::{MySql, Pool};

pub async fn delete_project_by_id(pool: &Pool<MySql>, project_id: i32) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    // First, delete related entries from MemberProject. (Assuming such a table exists)
    sqlx::query("DELETE FROM ContributesTo WHERE ProjectID = ?")
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
        .bind(raw_project.project_id)
        .fetch_all(pool)
        .await?;

        let mut sprints = Vec::new();

        for raw_sprint in raw_sprints {
            let tasks = sqlx::query_as::<_, Task>(
                "SELECT Task.* FROM Task
                 INNER JOIN PartOf ON Task.TaskID = PartOf.TaskID
                 WHERE PartOf.SprintID = ?",
            )
            .bind(raw_sprint.sprint_id)
            .fetch_all(pool)
            .await?;

            sprints.push(Sprint {
                sprint_id: raw_sprint.sprint_id,
                title: raw_sprint.title,
                start_date: raw_sprint.start_date,
                end_date: raw_sprint.end_date,
                tasks,
            });
        }

        let members = sqlx::query_as::<_, Member>(
            "SELECT Member.* FROM Member
             INNER JOIN ContributesTo ON Member.MemberID = ContributesTo.MemberID
             WHERE ContributesTo.ProjectID = ?",
        )
        .bind(raw_project.project_id)
        .fetch_all(pool)
        .await?;

        projects.push(Project {
            proj_id: raw_project.project_id,
            title: raw_project.title,
            desc: raw_project.description,
            sprints,
            members, // Added members to the project
        });
    }

    Ok(projects)
}

pub async fn delete_sprint_by_id(pool: &Pool<MySql>, sprint_id: i32) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    // First, delete related entries from ProjectSprint.
    sqlx::query("DELETE FROM ProjectSprint WHERE SprintID = ?")
        .bind(sprint_id)
        .execute(&mut *transaction)
        .await?;

    // Next, delete related tasks associated with the sprint.
    sqlx::query("DELETE FROM Task WHERE SprintID = ?")
        .bind(sprint_id)
        .execute(&mut *transaction)
        .await?;

    // Finally, delete the sprint itself.
    sqlx::query("DELETE FROM Sprint WHERE SprintID = ?")
        .bind(sprint_id)
        .execute(&mut *transaction)
        .await?;

    transaction.commit().await?;

    Ok(())
}

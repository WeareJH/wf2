use crate::recipes::m2::m2_env::{DB_NAME, DB_PASS, DB_USER};
use crate::{context::Context, task::Task};

pub fn exec(ctx: &Context) -> Vec<Task> {
    let container_name = format!("wf2__{}__db", ctx.name);
    let db_dump_command = format!(
        r#"docker exec -i {container} mysqldump -u{user} -p{pass} {db} > dump.sql"#,
        container = container_name,
        user = DB_USER,
        pass = DB_PASS,
        db = DB_NAME,
    );
    vec![
        Task::simple_command(db_dump_command),
        Task::notify("Written to file dump.sql"),
    ]
}

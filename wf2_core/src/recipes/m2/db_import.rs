use crate::{
    context::Context,
    recipes::magento_2::{DB_NAME, DB_PASS, DB_USER},
    task::Task,
    util::path_buf_to_string,
};
use std::path::PathBuf;

//
// Import a DB from a file
//
pub fn exec(ctx: &Context, path: PathBuf) -> Vec<Task> {
    let container_name = format!("wf2__{}__db", ctx.name);
    let db_import_command = match ctx.pv {
        Some(..) => format!(
            r#"pv -f {file} | docker exec -i {container} mysql -u{user} -p{pass} -D {db}"#,
            file = path_buf_to_string(&path),
            container = container_name,
            user = DB_USER,
            pass = DB_PASS,
            db = DB_NAME,
        ),
        None => format!(
            r#"docker exec -i {container} mysql -u{user} -p{pass} {db} < {file}"#,
            file = path_buf_to_string(&path),
            container = container_name,
            user = DB_USER,
            pass = DB_PASS,
            db = DB_NAME,
        ),
    };
    vec![
        Task::file_exists(path, "Ensure that the given DB file exists"),
        Task::simple_command(db_import_command),
    ]
}

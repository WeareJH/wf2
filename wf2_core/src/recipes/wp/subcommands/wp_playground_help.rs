use crate::recipes::wp::subcommands::wp_playground::WpPlayground;

pub fn help(wp: &WpPlayground) -> String {
    format!(
        r#"Next steps:

    Now start wf2 in the new directory:

       cd {}
       wf2 up --clean

    Then run the following:

       wf2 composer install

    That's it - you should find the site running at

       https://{}

    Have fun :)

    "#,
        wp.dir.file_name().unwrap().to_string_lossy(),
        wp.domain
    )
}

use crate::recipes::m2::subcommands::m2_playground::M2Playground;

pub fn help(pg: &M2Playground) -> String {
    format!(
        r#"Next steps:

    Stop existing docker containers:

       docker stop $(docker ps -qa) && docker rm $(docker ps -qa)

    Now start wf2 in the new directory:

       cd {}
       wf2 up

    Then, once it's up an running, in a new tab, run the following:

       wf2 doctor
       wf2 composer install
       wf2 exec magento-install

    That's it - you should find the site running at

       https://local.m2

    Admin credentials:

       admin
       password123

    Have fun :)

    "#,
        pg.dir.file_name().unwrap().to_string_lossy()
    )
}

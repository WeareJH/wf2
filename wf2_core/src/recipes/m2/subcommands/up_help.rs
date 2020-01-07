use crate::context::Context;

use crate::recipes::m2::services::mail::MailService;
use crate::recipes::m2::services::rabbit_mq::RabbitMqService;
use ansi_term::Color::{Cyan, Green};

pub fn up_help(ctx: &Context) -> String {
    // An iterator over just the domains
    let all_domains = ctx.domains.clone().into_iter().chain(
        vec![
            MailService::DOMAIN.to_string(),
            RabbitMqService::DOMAIN.to_string(),
        ]
        .into_iter(),
    );

    // All the domains prefixed with 0.0.0.0
    let domains = all_domains
        .clone()
        .map(|domain| format!("0.0.0.0 {}", domain))
        .collect::<Vec<String>>();

    let hosts = || {
        format!(
            "    If you're not using DNSMasq, you'll need to make sure
    these lines are in your hosts file:

        {}",
            Cyan.paint(domains.join("\n        "))
        )
    };

    let host_command = || {
        format!(
            "    You can copy/paste this:

        {}",
            Green.paint(format!(
                r#"echo "0.0.0.0 {}" | sudo tee -a /etc/hosts"#,
                all_domains.collect::<Vec<String>>().join(" ")
            ))
        )
    };

    let init = || {
        format!(
            r#"    Initial Setup:

        {}
        {}
        {}
        {}"#,
            // initial setup
            Green.paint("wf2 doctor"),
            Green.paint("wf2 composer install"),
            Green.paint("wf2 db-import recent-db-dump.sql"),
            Green.paint("wf2 m setup:upgrade"),
        )
    };

    let items = vec![
        site(&ctx),
        extra_services(),
        hosts(),
        host_command(),
        init(),
    ];

    items.join("\n\n")
}

fn site(ctx: &Context) -> String {
    format!(
        "    The site should be running at: {}",
        Cyan.paint(urls(ctx.domains.clone()))
    )
}

fn extra_services() -> String {
    format!(
        "    Extra Services:

        {}
        {}",
        Cyan.paint(format!("{}", MailService)),
        Cyan.paint(format!("{}", RabbitMqService)),
    )
}

fn urls(ds: Vec<String>) -> String {
    ds.iter()
        .map(|d| format!("https://{}", d))
        .collect::<Vec<String>>()
        .join(", ")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::recipes::recipe_kinds::RecipeKinds;

    #[test]
    fn single_url() {
        let ctx = Context {
            recipe: Some(RecipeKinds::M2),
            ..Context::default()
        };
        println!("{}", up_help(&ctx))
    }

    #[test]
    fn multi_urls() {
        let ctx = Context {
            recipe: Some(RecipeKinds::M2),
            domains: vec![String::from("example.com"), String::from("example.org")],
            ..Context::default()
        };
        println!("{}", up_help(&ctx))
    }
}

use crate::context::Context;
use crate::file::File;
use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::php_debug::PhpDebugService;
use crate::recipes::m2::services::M2Service;
use std::path::PathBuf;

///
/// This file is written alongside the main nginx conf.
/// It only contains `upstream` blocks
///
#[derive(Debug, Clone)]
pub struct NginxUpstream {
    file_path: PathBuf,
    backend: String,
    backend_debug: String,
}

impl File<NginxUpstream> for NginxUpstream {
    const DESCRIPTION: &'static str = "write the upstream file";
    const OUTPUT_PATH: &'static str = "nginx/sites/upstream.conf";

    fn from_ctx(ctx: &Context) -> Result<NginxUpstream, failure::Error> {
        Ok(NginxUpstream {
            file_path: ctx.file_path(Self::OUTPUT_PATH),
            backend: PhpService::NAME.to_string(),
            backend_debug: PhpDebugService::NAME.to_string(),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        format!(
            r#"upstream fastcgi_backend {{
  server {}:9000;
}}

upstream fastcgi_backend_debug {{
  server {}:9000;
}}"#,
            self.backend, self.backend_debug
        )
        .into_bytes()
    }
}

impl NginxUpstream {
    pub fn toggle_xdebug(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            self.backend = PhpDebugService::NAME.to_string();
        } else {
            self.backend = PhpService::NAME.to_string();
        }
        self
    }
    pub fn build(&self) -> NginxUpstream {
        NginxUpstream { ..self.clone() }
    }
}

#[test]
fn test_nginxupstream_xdebug_disabled() -> Result<(), failure::Error> {
    let ctx = Context {
        cwd: PathBuf::from("/Users/shane/acme"),
        ..Context::default()
    };
    let us = NginxUpstream::from_ctx(&ctx)?;
    let expected = "upstream fastcgi_backend {
  server php:9000;
}

upstream fastcgi_backend_debug {
  server php-debug:9000;
}";
    assert_eq!(expected, std::str::from_utf8(&us.bytes()).expect("test"));
    Ok(())
}

#[test]
fn test_nginxupstream_xdebug_enabled() -> Result<(), failure::Error> {
    let ctx = Context {
        cwd: PathBuf::from("/Users/shane/acme"),
        ..Context::default()
    };
    let us = NginxUpstream::from_ctx(&ctx)?.toggle_xdebug(true).build();
    let expected = "upstream fastcgi_backend {
  server php-debug:9000;
}

upstream fastcgi_backend_debug {
  server php-debug:9000;
}";
    assert_eq!(expected, std::str::from_utf8(&us.bytes()).expect("test"));
    Ok(())
}

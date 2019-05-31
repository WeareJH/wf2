workflow "Release" {
  on = "release"
  resolves = ["Upload to release"]
}

action "Build Release" {
  uses = "docker://wearejh/rust-macos-build"
  args = "pwd && cargo build --target x86_64-apple-darwin --release"
}

action "Upload to release" {
  uses = "JasonEtco/upload-to-release@master"
  args = "target/x86_64-apple-darwin/release/wf2"
  secrets = ["GITHUB_TOKEN"]
  needs = ["Build Release"]
}

use argh::FromArgs;

use self::{
    format::FormatTask,
    image::{BuildImageTask, PushImageTask, TagImageTask},
    lint::LintTask,
    server::ServerTask,
    test::TestTask,
    version::SetVersionTask,
};

mod format;
mod image;
mod lint;
mod server;
mod test;
mod version;

/// Tasks
#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub(crate) enum Tasks {
    Format(FormatTask),
    Lint(LintTask),
    Test(TestTask),
    Server(ServerTask),
    SetVersion(SetVersionTask),
    BuildImage(BuildImageTask),
    TagImage(TagImageTask),
    PushImage(PushImageTask),
}

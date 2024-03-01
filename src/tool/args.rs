use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// SysKaleido, provides
pub struct SysKaleidoCommand {
    #[argh(subcommand)]
    pub cmd: TopCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum TopCommand {
    List(ListCommand),
    Search(SearchCommand),
    Install(InstallCommand),
    Update(UpdateCommand),
    Uninstall(UninstallCommand),
    Config(ConfigCommand),
    Bindle(BindleCommand),
    Version(AppVersionCommand),
    Upgrade(AppUpgradeCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// show sys-kaleido version.
#[argh(subcommand, name="version")]
pub struct AppVersionCommand {
}

#[derive(FromArgs, PartialEq, Debug)]
/// upgrade sys-kaleido to latest version.
#[argh(subcommand, name = "upgrade")]
pub struct AppUpgradeCommand {}

#[derive(FromArgs, PartialEq, Debug)]
/// configuration for sys-kaleido.
#[argh(subcommand, name = "config")]
pub struct ConfigCommand {
    #[argh(subcommand)]
    pub command: ConfigSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ConfigSubCommand {
    Update(ConfigUpdateCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// update sys-kaleido.
#[argh(subcommand, name = "update")]
pub struct ConfigUpdateCommand {}

#[derive(FromArgs, PartialEq, Debug)]
/// bundles commands.
#[argh(subcommand, name = "bindle")]
pub struct BindleCommand {
    #[argh(subcommand)]
    pub command: BindleSubCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum BindleSubCommand {
    List(BindleListCommand),
    Install(BindleInstallCommand),
    Update(BindleUpdateCommand),
    Uninstall(BindleUninstallCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// list local installed packages.
#[argh(subcommand, name = "list")]
pub struct ListCommand {
    #[argh(switch)]
    /// list all packages, no matter it's installed or not
    pub all: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// search packages.
#[argh(subcommand, name = "search")]
pub struct SearchCommand {
    #[argh(positional)]
    pub package: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// install all package in a bindle.
#[argh(subcommand, name = "install")]
pub struct BindleInstallCommand {
    #[argh(option, arg_name="rust-abi")]
    /// for rust application, choose which abi to use
    /// by default, it's 'msvc' on Windows, and 'gnu' for other OS.
    pub rust_abi: Option<String>,

    #[argh(switch, arg_name="force")]
    /// if true, the installed version will be deleted, then install the package again, even it's the same version.
    /// otherwise, the package will be skipped if the given version has been installed.
    pub force: bool,

    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// list all package in a bindle.
#[argh(subcommand, name = "list")]
pub struct BindleListCommand {
    /// bindle name, optional, if provided, list all packages in the bindle, otherwise, list all bindles.
    #[argh(positional)]
    pub name: Option<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// update all package in a bindle.
#[argh(subcommand, name = "update")]
pub struct BindleUpdateCommand {
    #[argh(option, arg_name="rust-abi")]
    /// for rust application, choose which abi to use
    /// by default, it's 'msvc' on Windows, and 'gnu' for other OS.
    pub rust_abi: Option<String>,

    #[argh(switch, arg_name="force")]
    /// if true, the installed version will be deleted, then install the package again, even it's the same version.
    /// otherwise, the package will be skipped if the given version has been installed.
    pub force: bool,

    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// uninstall all package in a bindle.
#[argh(subcommand, name = "uninstall")]
pub struct BindleUninstallCommand {
    #[argh(positional)]
    pub name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// install a released version.
#[argh(subcommand, name = "install")]
pub struct InstallCommand {
    /// version to install, only valid for single package
    #[argh(option, short = 'v')]
    pub version: Option<String>,

    /// create an alias for installed package, only valid for single package
    #[argh(option)]
    pub alias: Option<String>,

    #[argh(option, arg_name="rust-abi")]
    /// for rust application, choose which abi to use
    /// by default, it's 'msvc' on Windows, and 'gnu' for other OS.
    pub rust_abi: Option<String>,

    #[argh(switch, arg_name="force")]
    /// if true, the installed version will be deleted, then install the package again, even it's the same version.
    /// otherwise, the package will be skipped if the given version has been installed.
    pub force: bool,

    /// package name list, separated by whitespace
    #[argh(positional)]
    pub packages: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// uninstall a version
#[argh(subcommand, name = "uninstall")]
pub struct UninstallCommand {
    /// package name list, separated by whitespace
    #[argh(positional)]
    pub packages: Vec<String>,
}

#[derive(FromArgs, PartialEq, Debug)]
/// update packages
#[argh(subcommand, name = "update")]
pub struct UpdateCommand {
    /// version to install, only valid for single package
    #[argh(option, short = 'v')]
    pub version: Option<String>,

    /// create an alias for installed package, only valid for single package
    #[argh(option)]
    pub alias: Option<String>,

    #[argh(option, arg_name="rust-abi")]
    /// for rust application, choose which abi to use
    /// by default, it's 'msvc' on Windows, and 'gnu' for other OS.
    pub rust_abi: Option<String>,

    #[argh(switch, arg_name="force")]
    /// if true, the installed version will be deleted, then install the package again, even it's the same version.
    /// otherwise, the package will be skipped if the given version has been installed.
    pub force: bool,

    /// package name list, separated by whitespace
    #[argh(positional)]
    pub packages: Vec<String>,
}

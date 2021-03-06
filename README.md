# release-manager

release-manager is a highly openionated (spelled wrong on purpose :-) ) tool, aimed at making releasing of software
on GitHub easier. release-manager currently operates on two different "scopes" the first is locally, the second is
GitHub. When you run release-manager, you'll notice it has two different subcommands (more may be added in the future).

```
release-manager

USAGE:
    release-manager [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quite      Only error output will be displayed
    -V, --version    Prints version information
    -v, --verbose    Enables more verbose output

SUBCOMMANDS:
    artifactory    Artifactory project operations.
    github         Upload artifacts to different sources.
    help           Prints this message or the help of the given subcommand(s)
    local          Local project operations.
```

## Local Operations
The local option allows you to update version numbers via a CLI. While not really useful for average development,
it can be extremely useful when working with a CI environment. Namely, if you want to always release "snapshot" versions
of the codebase, without having to update the "version description". More on that later.

So you can have you CI job, on a branch merge do `release-manager local update-version --snapshot`. Doing so will update
the version file to be a "SNAPSHOT" version. (Adds a Semver post fix of "-SNAPSHOT-<Unix Epoch>" to the version number) .

## GitHub Operations
> _Making Updates to GitHub Repos_

This sub-command makes changes to GitHub using the API. The `bump` and `release-and-bump` commands do not require SSH
credentials to make the changes, the changes are done via the API.

This is useful on CI jobs where you don't want to add a key to the project, but want to be able to bump version numbers
when a CI job passes successfully.

The `release` and `release-and-bump` commands create a release in GitHub based on the HEAD.

*Note:* On a CI Job, you should set the `GITHUB_TOKEN` environment variable. Most CI systemd will not expose the
environment expect for specific branches, so your token should be safe if the repo doesn't expose it.

```
release-manager-github
Upload artifacts to different sources.

USAGE:
    release-manager github [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quite      Only error output will be displayed
    -V, --version    Prints version information
    -v, --verbose    Enables more verbose output

SUBCOMMANDS:
    bump                Bump the current version on GitHub.
    help                Prints this message or the help of the given subcommand(s)
    release             Tag the current branch with the version in the metadata file for the project.
    release-and-bump    Tag the current branch with the version in the metadata file for the project then bump the
                        patch version.
```

## Artifactory
This subcommand makes it easy to upload into artifactory, and distribute into Bintray.

```
release-manager-artifactory
Artifactory project operations.

USAGE:
    release-manager artifactory [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quite      Only error output will be displayed
    -V, --version    Prints version information
    -v, --verbose    Enables more verbose output

SUBCOMMANDS:
    distribute    Pushes the current version in Artifactory to Bintray.
    help          Prints this message or the help of the given subcommand(s)
    publish       Uploads a directory into artifactory.
```

### Artifactory - Publish

Normally, this would publish a Maven Local Repo. It gathers the artifacts in a directory, uploads it into artifactory,
and prepares it for publish into Bintray

### Artifactory - Distribute

Pushes a "Build" into Bintray.
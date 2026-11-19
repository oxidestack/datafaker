# Security policy

Please report a vulnerability privately, so that it can be fixed before it is public: on the repository's **Security**
tab choose **Report a vulnerability**, or use
[the form](https://github.com/oxidestack/datafaker/security/advisories/new). Only the maintainers see it. Do not open
a public issue, discussion or pull request for one.

Say which version or commit, the input that triggers it, and what an attacker gets (a panic, memory or CPU exhausted,
a read outside the data). The fix is released with a GitHub security advisory and credits you unless you prefer not to
be named. Fixes go into the latest release.

In scope: the crate's parsing of locale files, expressions and regular expressions it is handed. Fake values are not
random in a cryptographic sense: they are Datafaker's, reproducible from a seed, and must not be used as secrets. A
difference from Datafaker's output is a bug, to report as one.

Only the latest release is supported.

# pkgbuilds

Fallback PKGBUILDs for the packages `restore.sh` needs but a distro's repos may not carry
(Pocknix's locked ARM64 snapshot has none of them; Arch and Manjaro have all of them, so these
never build there). `lib.sh` builds one only when the package is neither installed nor in the
repos, in the order `tllist libutf8proc fcft foot swaybg zsh` (each needs the ones before it).

They build the same upstream release as Arch, trimmed to what a minimal base can build: no man
pages (`scdoc`), no PGO for foot (LLVM + a headless sway), no regenerated zsh docs (`yodl`).

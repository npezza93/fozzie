<p align="center">
  <a href="https://github.com/npezza93/fozzie">
    <img src="./.github/logo.jpg" width="750">
  </a>
</p>

# fozzie
![Build Status](https://github.com/npezza93/fozzie/workflows/tests/badge.svg)

fozzie is a simple and quick fuzzy text selector for the terminal.

### Installation

##### macOS and Linux

```
cargo install fozzie
```

Binaries are also available on the releases page of the repo.
Download the tar file, and place the executable in your path.

##### From source
```
git clone https://github.com/npezza93/fozzie
cd fozzie
cargo build --release
find. -type f | ./target/release/fozzie
```

### Usage

Pipe a list of items to fozzie and start searching:

```
find . -type f | fozzie
```

### Scoring
fozzies scoring algorithm is currently heavily based on fzy's implementaion
which can be found [here](https://github.com/jhawthorn/fzy/blob/master/ALGORITHM.md).
It favors consecutive letters and start of word matches

`file` matches `file` over `filter`

`amp`  matches `app/models/posts.rb`

`abce` matches `abcdef` over `abc de`

`test` matches `tests` over `testing`

`appcont` matches `app/controllers/application_controller.rb` over `app/controllers/home_controller.rb`

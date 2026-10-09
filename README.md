# iamb-themegen

This repository contains the tool, template and data use to generate the
TOML files for the themes in the [iamb] repository.

## Building

In order to work with the data in this repo, you will need to build the
`iamb-themegen` binary:

```
cargo build --release
```

## Generating Themes

The color assignments for the packaged themes lives in `data/iamb.json`,
and can be converted into the various `theme.toml` files with:

```
./target/release/iamb-themegen generate ./data/iamb.json
```

This will output the files into `themes/iamb/iamb/`.

## Converting Additional Vim Colorschemes

You can use the `iamb-themegen convert` subcommand to extract the colors
from the colorscheme repositories tracked by the [vimcolorschemes]
website. A copy of the database lives in `data/repositories.json`, and
can be processed with:

```
./target/release/iamb-themegen convert ./data/repositories.json > out.json
```

This will try to automatically assign colors by how well they read against the
background that they will be drawn on when inserted into the theme template,
and will attempt to pick out red, yellow and green colors to use for the
encryption status indicators, errors, etc. (For the curious, this is done
mainly through calculating the [CIELAB delta E][cielab] distance from target
colors, and using the [WCAG 2.1 contrast ratio][wcag] guidelines to determine
whether text will be legible.)

Note that this is a best-effort attempt, and you may need to manually reassign
colors if the outcome wasn't what you were expecting.

If a colorscheme that is in the input `repositories.json` doesn't have enough
unique colors to assign then it will be skipped. You can use `--verbose`
to get more information about why.

If the `repositories.json` is too old to contain a more recently tracked
repository then (assuming the site hasn't changed) you can refetch it with:

```
$ curl -s https://vimcolorschemes.com/search/repositories.json > repositories.json
```

## License

`iamb-themegen` is released under the [Apache License, Version 2.0].

[Apache License, Version 2.0]: https://github.com/ulyssa/iamb/blob/master/LICENSE
[iamb]: https://github.com/ulyssa/iamb
[vimcolorschemes]: https://vimcolorschemes.com
[wcag]: https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio
[cielab]: https://en.wikipedia.org/wiki/Color_difference#CIELAB_%CE%94E*

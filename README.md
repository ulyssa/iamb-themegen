# iamb-themes

This repository contains a number of generated themes for [iamb], using the
list of themes and their primary colors available at the [vimcolorschemes]
website.

## Generating themes

The `vim2iamb.py` script contains the primary logic, and can be fed the
`repositories.json` file (or other similarly-shaped input) to generate
multiple valid themes for iamb.

For example:

```
$ ./vim2iamb.py repositories.json
```

The output will be put into a `target/` directory.

[iamb]: https://github.com/ulyssa/iamb
[vimcolorschemes]: https://vimcolorschemes.com

# Clean Code Shit brand assets

The mark is a terminal bin with a tilted lid and a cutout `>_` prompt. The wordmark uses hand-drawn block letters with stepped lower-right strokes that echo the ANSI Shadow banner.

## Choose an asset

| Asset | Use |
| --- | --- |
| `logo.svg` and `logo-light.svg` | Transparent horizontal lockup for light surfaces. |
| `logo-dark.svg` | Transparent horizontal lockup for dark surfaces. |
| `logo-mark.svg` and `logo-mark-light.svg` | Transparent square mark for light surfaces. |
| `logo-mark-dark.svg` | Transparent square mark for dark surfaces. |
| `logo-mark-{64,128,256,512}.png` | Transparent PNGs of the light-surface mark at four square sizes. |
| `favicon.svg` and `favicon.ico` | Fixed charcoal tile for browser chrome. The ICO contains 16, 32, and 48 px images. |
| `apple-touch-icon.png` | Opaque 180 px icon for Apple's home screen mask. |
| `icon-192.png` and `icon-512.png` | Square app icons with transparent rounded corners. |

Use the mark by itself at 16 px. The favicon prompt uses a separate path aligned to the 16 px pixel grid. Both dark-surface SVGs use a cool white foreground and an amber lid. The light-surface SVGs use a charcoal foreground and an amber handle inside the charcoal lid.

| Color | Hex |
| --- | --- |
| Charcoal | `#1B1C21` |
| Cool white | `#F3F5F8` |
| Amber | `#FFC83D` |

## Reproduce the exports

The SVGs are the vector masters. The lockup lettering is outlined, so rendering needs only the SVG. After editing a light-surface master, copy `logo-light.svg` to `logo.svg` or `logo-mark-light.svg` to `logo-mark.svg` so the default stays in sync. When modifying the mark geometry, carry the path changes across the lockups and `favicon.svg`.

From the repository root, install the pinned renderer into a temporary folder.

```sh
npm install --prefix .brand-export --no-audit --no-fund --save-exact @resvg/resvg-js@2.6.2
```

Save the following as `.brand-export/export.cjs`, then run `node .brand-export/export.cjs assets/brand`. The script renders each size directly from its SVG and packs the three favicon images into one ICO.

```js
const fs = require('node:fs');
const path = require('node:path');
const { Resvg } = require('@resvg/resvg-js');

const brand = path.resolve(process.argv[2]);

function render(svg, size, background) {
  const source = fs.readFileSync(path.join(brand, svg));
  return new Resvg(source, {
    fitTo: { mode: 'width', value: size },
    background,
    font: { loadSystemFonts: false },
  }).render().asPng();
}

function write(name, data) {
  fs.writeFileSync(path.join(brand, name), data);
}

for (const size of [64, 128, 256, 512]) {
  write(`logo-mark-${size}.png`, render('logo-mark.svg', size));
}
for (const size of [192, 512]) {
  write(`icon-${size}.png`, render('favicon.svg', size));
}
write('apple-touch-icon.png', render('favicon.svg', 180, '#1B1C21'));

const sizes = [16, 32, 48];
const directory = Buffer.alloc(6 + sizes.length * 16);
directory.writeUInt16LE(1, 2);
directory.writeUInt16LE(sizes.length, 4);
let offset = directory.length;
const images = sizes.map((size, index) => {
  const png = render('favicon.svg', size);
  const entry = 6 + index * 16;
  directory[entry] = directory[entry + 1] = size;
  directory.writeUInt16LE(1, entry + 4);
  directory.writeUInt16LE(32, entry + 6);
  directory.writeUInt32LE(png.length, entry + 8);
  directory.writeUInt32LE(offset, entry + 12);
  offset += png.length;
  return png;
});
write('favicon.ico', Buffer.concat([directory, ...images]));
```

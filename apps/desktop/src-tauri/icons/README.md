# Tauri Icons

This directory should contain the application icons for various platforms:

- `32x32.png` - 32x32 pixel icon
- `128x128.png` - 128x128 pixel icon
- `128x128@2x.png` - 256x256 pixel retina icon
- `icon.icns` - macOS icon bundle
- `icon.ico` - Windows icon

## Generating Icons

You can generate these icons from a source image using the Tauri CLI:

```bash
npm run tauri icon /path/to/source/icon.png
```

The source image should be at least 512x512 pixels and preferably square.

## Placeholder

For development purposes, Tauri will use default icons if these files are not present.
